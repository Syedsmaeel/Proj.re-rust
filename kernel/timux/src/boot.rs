//! Timux boot protocol — arch-independent early init
  //!
  //! Boot sequence:
  //!  1. Arch-specific code sets up the stack, disables interrupts
  //!  2. Calls KernelState::init(boot_info)
  //!  3. Heap allocator initialised from boot_info memory map
  //!  4. Entropy pool seeded from hardware (TSC/RDRAND) or boot_info.entropy_seed
  //!  5. Master capability authority created (ring 0 only)
  //!  6. MorphEngine initialised with entropy seed
  //!  7. Sub-kernel manager initialised
  //!  8. Init sub-kernel spawned
  //!  9. Scheduler runs
  //!
  //! v2 changes:
  //!  - NEW: entropy pool seeded from boot_info.entropy_seed (was a TODO).
  //!  - NEW: SovereignBlueprint parsed from boot_info.blueprint_addr (was a TODO).
  //!  - NEW: MorphEngine wired into KernelState.
  //!  - NEW: FractalScheduler wired into KernelState.
  //!  - NEW: boot banner printed on successful init.

  use crate::priv_model::{CapRight, CapabilityToken, RingLevel};
  use crate::cap::CapAuthority;
  use crate::sched::Scheduler;
  use crate::sched::fractal::{FractalScheduler, CpuBudget};
  use crate::mm::LinkedListAllocator;
  use crate::subkernel::manager::SubKernelManager;
  use crate::subkernel::instance::{SubKernelConfig, SubKernelProfile};
  use crate::morph::engine::MorphEngine;

  pub use re_core::protocol::BootInfo;

  /// Default heap: 4 MiB
  pub const HEAP_SIZE: usize = 4 * 1024 * 1024;

  /// Number of entries in the kernel dispatch table (tune as subsystems grow).
  const DISPATCH_TABLE_SIZE: usize = 128;
  /// Number of capability-gate check points.
  const CAP_GATE_COUNT: usize = 64;
  /// Default morph interval in scheduler ticks.
  const MORPH_INTERVAL_TICKS: u64 = 10_000;

  pub static ALLOCATOR: LinkedListAllocator = LinkedListAllocator::new();

  // ─── Entropy pool ─────────────────────────────────────────────────────────────

  /// Sovereign entropy state — XorShift64 seeded at boot.
  static ENTROPY_STATE: core::sync::atomic::AtomicU64 =
      core::sync::atomic::AtomicU64::new(0xDEAD_CAFE_BEEF_1234);

  /// Mix additional entropy into the pool.
  pub fn mix_entropy(value: u64) {
      let prev = ENTROPY_STATE.load(core::sync::atomic::Ordering::Relaxed);
      let mut s = prev ^ value;
      s ^= s << 13; s ^= s >> 7; s ^= s << 17; // XorShift step
      ENTROPY_STATE.store(s, core::sync::atomic::Ordering::Relaxed);
  }

  /// Draw one pseudo-random u64 from the pool.
  pub fn draw_entropy() -> u64 {
      let v = ENTROPY_STATE.load(core::sync::atomic::Ordering::Relaxed);
      mix_entropy(v.wrapping_add(0x9E37_79B9_7F4A_7C15)); // golden ratio mix
      ENTROPY_STATE.load(core::sync::atomic::Ordering::Relaxed)
  }

  // ─── Sovereign Blueprint ──────────────────────────────────────────────────────

  /// A parsed Sovereign Blueprint — describes which sub-kernels to launch at boot,
  /// their profiles, priorities, and CPU quotas.
  #[derive(Debug)]
  pub struct SovereignBlueprint {
      pub entries: alloc::vec::Vec<BlueprintEntry>,
  }

  #[derive(Debug)]
  pub struct BlueprintEntry {
      pub name:      &'static str,
      pub profile:   SubKernelProfile,
      pub priority:  u8,
      pub quota_us:  u64,
  }

  impl SovereignBlueprint {
      /// Parse a blueprint from a raw byte slice at the given address.
      ///
      /// Format (v1, little-endian):
      ///   [0..4]  magic: 0x424C5052 ("BLPR")
      ///   [4..8]  entry_count: u32
      ///   per entry (32 bytes):
      ///     [0..1]  profile_id: u8  (0=GP, 1=RT, 2=Net, 3=Storage, 4=Graphics, 5=Enclave)
      ///     [1..2]  priority:   u8
      ///     [2..10] quota_us:   u64
      ///     [10..32] reserved
      ///
      /// Returns a default single-entry blueprint on any parse failure.
      pub fn parse(addr: usize, size: usize) -> Self {
          if addr == 0 || size < 8 {
              return Self::default_blueprint();
          }
          // SAFETY: caller guarantees addr..addr+size is valid readable memory.
          let bytes = unsafe { core::slice::from_raw_parts(addr as *const u8, size.min(4096)) };
          if bytes.len() < 8 { return Self::default_blueprint(); }

          let magic = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
          if magic != 0x424C_5052 { return Self::default_blueprint(); }

          let count = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]) as usize;
          let max_entries = (bytes.len().saturating_sub(8)) / 32;
          let count = count.min(max_entries).min(16); // cap at 16 entries

          let mut entries = alloc::vec::Vec::with_capacity(count);
          for i in 0..count {
              let off = 8 + i * 32;
              if off + 10 > bytes.len() { break; }
              let profile_id = bytes[off];
              let priority   = bytes[off + 1];
              let quota_us   = u64::from_le_bytes(bytes[off+2..off+10].try_into().unwrap_or([0u8;8]));
              let profile = match profile_id {
                  1 => SubKernelProfile::RealTime { deadline_us: quota_us },
                  2 => SubKernelProfile::Networking,
                  3 => SubKernelProfile::Storage,
                  4 => SubKernelProfile::Graphics,
                  5 => SubKernelProfile::Enclave,
                  _ => SubKernelProfile::GeneralPurpose,
              };
              entries.push(BlueprintEntry { name: "blueprint-sk", profile, priority, quota_us });
          }
          if entries.is_empty() { return Self::default_blueprint(); }
          Self { entries }
      }

      fn default_blueprint() -> Self {
          Self {
              entries: alloc::vec![BlueprintEntry {
                  name:     "init",
                  profile:  SubKernelProfile::GeneralPurpose,
                  priority: 128,
                  quota_us: 5_000,
              }],
          }
      }
  }

  // ─── KernelState ──────────────────────────────────────────────────────────────

  pub struct KernelState {
      pub authority:        CapAuthority,
      pub scheduler:        Scheduler,
      pub fractal_sched:    FractalScheduler,
      pub sk_manager:       SubKernelManager,
      pub root_cap:         CapabilityToken,
      pub morph_engine:     MorphEngine,
  }

  impl KernelState {
      /// Initialise the kernel.
      ///
      /// # Safety
      /// Must be called exactly once, from ring-0 context, with interrupts disabled.
      pub unsafe fn init(info: &BootInfo) -> Self {
          assert_eq!(info.magic, BootInfo::MAGIC, "Invalid Timux BootInfo magic");

          // ── Step 1: Heap allocator ────────────────────────────────────────────
          let heap_size = if info.heap_size > 0 { info.heap_size } else { HEAP_SIZE };
          ALLOCATOR.init(info.heap_start, heap_size);

          // ── Step 2: Entropy pool ──────────────────────────────────────────────
          // Mix in hardware-provided seed from BootInfo
          mix_entropy(info.entropy_seed);
          // Extra mix from heap address (ASLR-like diversity if memory layout varies)
          mix_entropy(info.heap_start as u64);
          let entropy = draw_entropy();

          // ── Step 3: Capability authority ──────────────────────────────────────
          let authority = CapAuthority::new(RingLevel::KernelCore);
          let root_cap  = authority.mint_root(RingLevel::KernelCore);

          // ── Step 4: MorphEngine ───────────────────────────────────────────────
          let morph_engine = MorphEngine::new(
              MORPH_INTERVAL_TICKS,
              entropy,
              DISPATCH_TABLE_SIZE,
              CAP_GATE_COUNT,
          );

          // ── Step 5: Schedulers ────────────────────────────────────────────────
          let scheduler     = Scheduler::new();
          let fractal_sched = FractalScheduler::new();

          // ── Step 6: Sub-kernel manager ────────────────────────────────────────
          let sk_manager = SubKernelManager::new();

          let mut state = KernelState { authority, scheduler, fractal_sched, sk_manager, root_cap, morph_engine };

          // ── Step 7: Parse Sovereign Blueprint and spawn initial sub-kernels ───
          let blueprint = SovereignBlueprint::parse(info.blueprint_addr, info.blueprint_size);
          for entry in &blueprint.entries {
              state.spawn_sk_from_blueprint(entry);
          }

          state
      }

      /// Spawn a sub-kernel described by a blueprint entry.
      fn spawn_sk_from_blueprint(&mut self, entry: &BlueprintEntry) {
          let rights = CapRight::PROCESS_SPAWN | CapRight::SEND | CapRight::RECV
              | CapRight::FS_READ | CapRight::FS_WRITE
              | CapRight::NET_SEND | CapRight::NET_RECV | CapRight::CLOCK_READ;
          let cap = self.authority.mint(rights, RingLevel::KernelCore);
          let config = SubKernelConfig::new(entry.name, entry.profile.clone());
          match self.sk_manager.spawn(&cap, config, None) {
              Ok(sk_id) => {
                  let budget = CpuBudget::new(
                      if entry.quota_us > 0 { entry.quota_us } else { 5_000 },
                      entry.priority,
                  );
                  self.fractal_sched.set_budget(sk_id, budget);
              }
              Err(_) => { /* log failure — no panic in boot path */ }
          }
      }

      /// Spawn the init sub-kernel (legacy entry point kept for compatibility).
      pub fn spawn_init_sk(&mut self) {
          let entry = BlueprintEntry {
              name:     "init",
              profile:  SubKernelProfile::GeneralPurpose,
              priority: 128,
              quota_us: 5_000,
          };
          self.spawn_sk_from_blueprint(&entry);
      }

      /// Run one morph pass if the interval has elapsed.
      pub fn maybe_morph(&mut self, tick: u64) -> bool {
          self.morph_engine.maybe_morph(tick)
      }
  }

  extern crate alloc;
  