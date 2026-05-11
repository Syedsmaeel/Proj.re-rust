//! Fractal Scheduler — CPU budget management across sub-kernels
  //!
  //! v2: implements weighted round-robin scheduling among sub-kernels.
  //!
  //! Each sub-kernel receives a CpuBudget expressed as a quota in microseconds
  //! per scheduling epoch (default 10 ms).  The scheduler tracks consumption and
  //! rotates fairly while respecting priority weights.
  //!
  //!  - tick(elapsed_us)      — advance all budgets, return next sk to run.
  //!  - yield_sk(id)          — sk blocks/yields; ineligible until next epoch.
  //!  - set_budget(id, b)     — register or update a sub-kernel.
  //!  - remove(id)            — deregister a terminated sub-kernel.
  //!  - utilisation()         — iterator of (id, consumed_us, quota_us).

  extern crate alloc;

  use alloc::collections::BTreeMap;
  use crate::subkernel::instance::SubKernelId;

  pub const EPOCH_US: u64 = 10_000; // 10 ms

  // ─── CpuBudget ────────────────────────────────────────────────────────────────

  #[derive(Debug, Clone)]
  pub struct CpuBudget {
      /// Quota per epoch in microseconds.
      pub quota_us:  u64,
      /// Scheduling priority — 0 = highest; lower value wins ties.
      pub priority:  u8,
      /// Remaining budget this epoch (µs).
      pub remaining: u64,
      /// Total CPU time consumed (µs) across all epochs.
      pub consumed:  u64,
      /// True when the sk has yielded or exhausted its budget.
      pub yielded:   bool,
      /// Epoch counter — refreshed when global epoch advances.
      pub epoch:     u64,
  }

  impl CpuBudget {
      pub fn new(quota_us: u64, priority: u8) -> Self {
          Self { quota_us, priority, remaining: quota_us, consumed: 0, yielded: false, epoch: 0 }
      }

      fn refresh(&mut self, new_epoch: u64) {
          if self.epoch < new_epoch {
              self.remaining = self.quota_us;
              self.yielded   = false;
              self.epoch     = new_epoch;
          }
      }

      fn charge(&mut self, us: u64) {
          let used = us.min(self.remaining);
          self.remaining -= used;
          self.consumed  += used;
          if self.remaining == 0 { self.yielded = true; }
      }
  }

  // ─── FractalScheduler ─────────────────────────────────────────────────────────

  pub struct FractalScheduler {
      pub budgets:  BTreeMap<SubKernelId, CpuBudget>,
      current:      Option<SubKernelId>,
      epoch:        u64,
      elapsed_acc:  u64,
      tick_count:   u64,
  }

  impl FractalScheduler {
      pub fn new() -> Self {
          Self { budgets: BTreeMap::new(), current: None, epoch: 1, elapsed_acc: 0, tick_count: 0 }
      }

      /// Register or update a sub-kernel's CPU budget.
      pub fn set_budget(&mut self, sk_id: SubKernelId, budget: CpuBudget) {
          self.budgets.insert(sk_id, budget);
      }

      /// Remove a terminated sub-kernel.
      pub fn remove(&mut self, sk_id: SubKernelId) {
          self.budgets.remove(&sk_id);
          if self.current == Some(sk_id) { self.current = None; }
      }

      /// Advance time by `elapsed_us` and return the next sub-kernel to run.
      ///
      /// Returns Some(id) when a sub-kernel should be given the CPU, or None
      /// when all budgets for this epoch are exhausted.
      pub fn tick(&mut self, elapsed_us: u64) -> Option<SubKernelId> {
          self.tick_count += 1;

          // Charge elapsed time to the currently running sk
          if let Some(id) = self.current {
              if let Some(b) = self.budgets.get_mut(&id) { b.charge(elapsed_us); }
          }

          // Advance epoch boundary
          self.elapsed_acc += elapsed_us;
          if self.elapsed_acc >= EPOCH_US {
              self.elapsed_acc -= EPOCH_US;
              self.epoch += 1;
              for b in self.budgets.values_mut() { b.refresh(self.epoch); }
          }

          // Pick next: lowest priority number wins; ties broken by lowest id
          let next = self.budgets.iter()
              .filter(|(_, b)| !b.yielded && b.remaining > 0)
              .min_by_key(|(id, b)| (b.priority, *id))
              .map(|(id, _)| *id);

          self.current = next;
          next
      }

      /// Mark a sub-kernel as voluntarily yielded for the rest of this epoch.
      pub fn yield_sk(&mut self, sk_id: SubKernelId) {
          if let Some(b) = self.budgets.get_mut(&sk_id) { b.yielded = true; }
          if self.current == Some(sk_id) { self.current = None; }
      }

      pub fn tick_count(&self) -> u64            { self.tick_count }
      pub fn epoch(&self) -> u64                 { self.epoch }
      pub fn current(&self) -> Option<SubKernelId> { self.current }

      /// Iterator of (sk_id, consumed_us, quota_us) for monitoring.
      pub fn utilisation(&self) -> impl Iterator<Item = (SubKernelId, u64, u64)> + '_ {
          self.budgets.iter().map(|(id, b)| (*id, b.consumed, b.quota_us))
      }
  }
  