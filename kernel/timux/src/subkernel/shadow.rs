//! Sovereign Shadow Manager
  //!
  //! Keeps a mirror replica of every primary sub-kernel in lock-step.
  //! On health failure, the shadow is promoted to primary in O(1).
  //!
  //! # Sync model
  //!
  //! `sync()` performs a physical memory copy from primary to shadow using
  //! `core::ptr::copy_nonoverlapping`.  Both regions must be:
  //!   - Valid, non-overlapping physical frames allocated at spawn time.
  //!   - Identical in size (enforced by the sub-kernel manager).
  //!
  //! # Health model
  //!
  //! Health is judged by a monotonically increasing heartbeat counter that
  //! each primary sub-kernel increments every scheduler quantum.  The shadow
  //! manager checks on every `tick()` call.  A primary is considered failed
  //! when its heartbeat counter has not advanced for `STALE_THRESHOLD` ticks.
  //!
  //! # Failover
  //!
  //! When `promote_shadow()` is called the shadow instance becomes the new
  //! primary and the old primary is dropped.  The caller receives the shadow's
  //! SubKernelId so it can repoint all capability references.
  extern crate alloc;

  use alloc::boxed::Box;
  use alloc::vec::Vec;
  use crate::subkernel::instance::{SubKernel, SubKernelId, MemoryRange, SubKernelState};

  // ─── Tuning ───────────────────────────────────────────────────────────────────

  /// How many consecutive missed heartbeats trigger automatic failover.
  pub const STALE_THRESHOLD: u64 = 8;

  // ─── SyncStats ───────────────────────────────────────────────────────────────

  #[derive(Debug, Default, Clone, Copy)]
  pub struct SyncStats {
      pub syncs_ok:       u64,
      pub syncs_skipped:  u64,  // size mismatch or zero-size
      pub failovers:      u64,
      pub last_sync_tick: u64,
      pub bytes_mirrored: u64,
  }

  // ─── ShadowInstance ───────────────────────────────────────────────────────────

  /// A paired primary + shadow sub-kernel instance.
  pub struct ShadowInstance {
      pub id:             SubKernelId,
      pub primary:        Box<SubKernel>,
      pub shadow:         Box<SubKernel>,
      pub is_healthy:     bool,
      pub sync_tick:      u64,
      pub last_heartbeat: u64,
      pub stale_count:    u64,
      pub stats:          SyncStats,
  }

  impl ShadowInstance {
      pub fn new(primary: SubKernel, shadow: SubKernel) -> Self {
          let id = primary.id;
          Self {
              id,
              primary:        Box::new(primary),
              shadow:         Box::new(shadow),
              is_healthy:     true,
              sync_tick:      0,
              last_heartbeat: 0,
              stale_count:    0,
              stats:          SyncStats::default(),
          }
      }

      // ── Memory sync ───────────────────────────────────────────────────────────

      /// Mirror the primary's full memory region to the shadow.
      ///
      /// # Safety
      /// Both the primary and shadow memory ranges must point to valid,
      /// non-overlapping physical frames of equal size.  Size equality is
      /// verified here — the copy is skipped (and counted as skipped) if they
      /// differ.
      pub fn sync(&mut self) {
          let primary_range = self.primary.memory_range();
          let shadow_range  = self.shadow.memory_range_mut();

          let overlaps = primary_range.start < shadow_range.end()
              && shadow_range.start < primary_range.end();
          if primary_range.size == 0
              || primary_range.size != shadow_range.size
              || primary_range.start == 0
              || shadow_range.start == 0
              || overlaps
          {
              // Null, mismatched or overlapping ranges: copying would be UB.
              self.stats.syncs_skipped += 1;
              return;
          }

          // SAFETY: Caller (SubKernelManager) guarantees non-overlap and validity.
          unsafe {
              core::ptr::copy_nonoverlapping(
                  primary_range.start as *const u8,
                  shadow_range.start  as *mut   u8,
                  primary_range.size,
              );
          }

          self.sync_tick += 1;
          self.stats.syncs_ok       += 1;
          self.stats.last_sync_tick  = self.sync_tick;
          self.stats.bytes_mirrored += primary_range.size as u64;
      }

      // ── Health monitoring ─────────────────────────────────────────────────────

      /// Advance one scheduler tick; check primary heartbeat.
      /// Returns `true` if the primary is still healthy, `false` if it has
      /// exceeded the stale threshold and needs failover.
      pub fn tick(&mut self, current_tick: u64) -> bool {
          let hb = self.primary.heartbeat();
          if hb > self.last_heartbeat {
              // Primary is alive — reset stale counter and sync
              self.last_heartbeat = hb;
              self.stale_count    = 0;
              self.sync();
          } else {
              self.stale_count += 1;
          }

          if self.stale_count >= STALE_THRESHOLD {
              self.is_healthy = false;
          }
          self.is_healthy
      }

      // ── Failover ──────────────────────────────────────────────────────────────

      /// Promote the shadow to primary.
      ///
      /// The old primary is dropped.  Returns the shadow's SubKernelId so the
      /// caller can update all external references.
      pub fn promote_shadow(&mut self) -> SubKernelId {
          // Swap shadow into primary slot
          core::mem::swap(&mut self.primary, &mut self.shadow);
          self.primary.state  = SubKernelState::Running;
          self.is_healthy     = true;
          self.stale_count    = 0;
          // Re-baseline: the promoted kernel has its own heartbeat counter.
          // Keeping the old primary's value would make a perfectly healthy
          // new primary look stale until it overtook the dead one.
          self.last_heartbeat = self.primary.heartbeat();
          self.stats.failovers += 1;
          self.primary.id
      }

      pub fn primary_id(&self) -> SubKernelId { self.primary.id }
      pub fn shadow_id(&self)  -> SubKernelId { self.shadow.id  }
  }

  // ─── ShadowManager ────────────────────────────────────────────────────────────

  /// Kernel-global shadow registry.
  pub struct ShadowManager {
      instances: Vec<ShadowInstance>,
  }

  impl ShadowManager {
      pub fn new() -> Self { Self { instances: Vec::new() } }

      /// Register a (primary, shadow) pair.
      pub fn register(&mut self, primary: SubKernel, shadow: SubKernel) -> SubKernelId {
          let id = primary.id;
          self.instances.push(ShadowInstance::new(primary, shadow));
          id
      }

      /// Deregister a pair by primary id.
      pub fn deregister(&mut self, id: SubKernelId) -> bool {
          if let Some(pos) = self.instances.iter().position(|s| s.primary_id() == id) {
              self.instances.remove(pos);
              true
          } else {
              false
          }
      }

      /// Advance all instances by one scheduler tick.
      /// Returns a list of primary ids that need failover.
      pub fn tick_all(&mut self, current_tick: u64) -> Vec<SubKernelId> {
          let mut failed = Vec::new();
          for inst in &mut self.instances {
              if !inst.tick(current_tick) {
                  failed.push(inst.primary_id());
              }
          }
          failed
      }

      /// Force a sync of all healthy instances (e.g. before checkpoint).
      pub fn sync_all(&mut self) {
          for inst in &mut self.instances {
              if inst.is_healthy { inst.sync(); }
          }
      }

      /// Promote the shadow of a failed primary.  Returns the new primary id.
      pub fn failover(&mut self, failed_id: SubKernelId) -> Option<SubKernelId> {
          let inst = self.instances.iter_mut().find(|s| s.primary_id() == failed_id)?;
          Some(inst.promote_shadow())
      }

      pub fn count(&self) -> usize { self.instances.len() }

      pub fn aggregate_stats(&self) -> SyncStats {
          let mut agg = SyncStats::default();
          for inst in &self.instances {
              agg.syncs_ok       += inst.stats.syncs_ok;
              agg.syncs_skipped  += inst.stats.syncs_skipped;
              agg.failovers      += inst.stats.failovers;
              agg.bytes_mirrored += inst.stats.bytes_mirrored;
          }
          agg
      }
  }
  

#[cfg(test)]
mod failover_tests {
    use super::*;
    use crate::subkernel::instance::{SubKernelConfig, SubKernelProfile};

    fn sk(name: &'static str) -> SubKernel {
        SubKernel::spawn(SubKernelConfig::new(name, SubKernelProfile::GeneralPurpose), None)
    }

    #[test]
    fn promoted_shadow_is_judged_by_its_own_heartbeat() {
        let mut inst = ShadowInstance::new(sk("p"), sk("s"));
        for _ in 0..5 { inst.primary.tick(); }
        assert!(inst.tick(0));              // baseline = 5
        for _ in 0..3 { inst.shadow.tick(); }

        inst.promote_shadow();              // new primary heartbeat = 3
        inst.primary.tick();                // -> 4, which is progress
        assert!(inst.tick(0));
        assert_eq!(inst.stale_count, 0, "healthy new primary must not look stale");
    }
}
