//! Liquid Logic Engine — Self-randomising binary layout for the Sovereign Substrate
  //!
  //! v2: implements actual morph logic.
  //!
  //! The MorphEngine periodically re-randomises three layers of the kernel's binary
  //! layout to defeat memory-disclosure and ROP-gadget attacks:
  //!
  //!  1. Function dispatch table shuffle — a vtable-style indirect-call table is
  //!     permuted using a fast XorShift64 PRNG seeded from the entropy pool.
  //!  2. Capability-gate offset re-mapping — the byte offsets at which capability
  //!     checks are performed are rotated so attacker-observed patterns become stale.
  //!  3. I-cache flush hint — logs a flush request so arch-specific code can act.
  //!
  //! The engine does NOT require an MMU or hardware ASLR; all randomisation is
  //! performed inside the existing address space by rewriting the indirection tables.

  extern crate alloc;

  use alloc::vec::Vec;
  use core::sync::atomic::{AtomicU64, Ordering};

  // ─── PRNG ─────────────────────────────────────────────────────────────────────

  /// XorShift64 — fast, non-cryptographic PRNG sufficient for layout randomisation.
  #[derive(Debug, Clone)]
  pub struct XorShift64 {
      state: u64,
  }

  impl XorShift64 {
      pub fn new(seed: u64) -> Self {
          // Seed must never be zero
          Self { state: if seed == 0 { 0xDEAD_BEEF_CAFE_1234 } else { seed } }
      }

      pub fn next(&mut self) -> u64 {
          self.state ^= self.state << 13;
          self.state ^= self.state >> 7;
          self.state ^= self.state << 17;
          self.state
      }

      /// Random index in [0, n)
      pub fn next_usize(&mut self, n: usize) -> usize {
          if n <= 1 { return 0; }
          (self.next() as usize) % n
      }
  }

  // ─── Dispatch table ───────────────────────────────────────────────────────────

  /// A permutable function-pointer dispatch table.
  /// Slots hold indices into a stable function array; only the slot→function
  /// mapping is shuffled, so callers use slot IDs that remain valid.
  pub struct DispatchTable {
      /// slot_to_fn[slot] = function_index
      slot_to_fn: Vec<usize>,
      /// fn_to_slot[fn_index] = current slot
      fn_to_slot: Vec<usize>,
      len: usize,
  }

  impl DispatchTable {
      pub fn new(n: usize) -> Self {
          let slot_to_fn: Vec<usize> = (0..n).collect();
          let fn_to_slot: Vec<usize> = (0..n).collect();
          Self { slot_to_fn, fn_to_slot, len: n }
      }

      /// Fisher–Yates shuffle using the provided PRNG.
      pub fn shuffle(&mut self, rng: &mut XorShift64) {
          for i in (1..self.len).rev() {
              let j = rng.next_usize(i + 1);
              self.slot_to_fn.swap(i, j);
          }
          // Rebuild reverse map
          for (slot, &func) in self.slot_to_fn.iter().enumerate() {
              self.fn_to_slot[func] = slot;
          }
      }

      /// Resolve a slot to a function index.
      pub fn resolve(&self, slot: usize) -> Option<usize> {
          self.slot_to_fn.get(slot).copied()
      }

      /// Find the current slot for a known function index.
      pub fn slot_of(&self, func_idx: usize) -> Option<usize> {
          self.fn_to_slot.get(func_idx).copied()
      }
  }

  // ─── Capability-gate map ──────────────────────────────────────────────────────

  /// Maps logical capability-check IDs to byte offsets that are rotated each morph.
  pub struct CapGateMap {
      offsets: Vec<u32>,   // offset[check_id] = current byte offset in gate region
      region_size: u32,
      len: usize,
  }

  impl CapGateMap {
      pub fn new(n: usize, region_size: u32) -> Self {
          // Initial layout: evenly spaced
          let step = if n > 0 { region_size / n as u32 } else { 0 };
          let offsets = (0..n).map(|i| i as u32 * step).collect();
          Self { offsets, region_size, len: n }
      }

      /// Re-assign offsets by rotating each by a random delta.
      pub fn randomise(&mut self, rng: &mut XorShift64) {
          for off in self.offsets.iter_mut() {
              let delta = (rng.next() % self.region_size as u64) as u32;
              *off = (*off + delta) % self.region_size;
          }
      }

      pub fn offset_of(&self, check_id: usize) -> Option<u32> {
          self.offsets.get(check_id).copied()
      }
  }

  // ─── ICache flush request ─────────────────────────────────────────────────────

  static ICACHE_FLUSH_PENDING: AtomicU64 = AtomicU64::new(0);

  pub fn request_icache_flush() {
      ICACHE_FLUSH_PENDING.fetch_add(1, Ordering::Release);
  }

  pub fn icache_flush_pending() -> bool {
      ICACHE_FLUSH_PENDING.load(Ordering::Acquire) > 0
  }

  pub fn acknowledge_icache_flush() {
      ICACHE_FLUSH_PENDING.fetch_sub(1, Ordering::AcqRel);
  }

  // ─── MorphEngine ─────────────────────────────────────────────────────────────

  pub struct MorphEngine {
      pub morph_interval: u64,
      pub last_morph:     u64,
      rng:                XorShift64,
      dispatch:           DispatchTable,
      cap_gates:          CapGateMap,
      morph_count:        u64,
  }

  impl MorphEngine {
      /// Create a new engine.
      ///
      /// - `interval`   — ticks between morph passes.
      /// - `seed`       — entropy seed (from boot-time hardware RNG or TSC).
      /// - `fn_count`   — number of entries in the dispatch table.
      /// - `gate_count` — number of capability-gate check points.
      pub fn new(interval: u64, seed: u64, fn_count: usize, gate_count: usize) -> Self {
          Self {
              morph_interval: interval,
              last_morph:     0,
              rng:            XorShift64::new(seed),
              dispatch:       DispatchTable::new(fn_count),
              cap_gates:      CapGateMap::new(gate_count, 0x1_0000),
              morph_count:    0,
          }
      }

      /// Check if a morph pass is due and execute it if so.
      pub fn maybe_morph(&mut self, current_tick: u64) -> bool {
          if current_tick.wrapping_sub(self.last_morph) >= self.morph_interval {
              self.morph();
              self.last_morph = current_tick;
              true
          } else {
              false
          }
      }

      /// Execute one morph pass unconditionally.
      pub fn morph(&mut self) {
          // 1. Shuffle the function dispatch table
          self.dispatch.shuffle(&mut self.rng);

          // 2. Re-randomise capability-gate offsets
          self.cap_gates.randomise(&mut self.rng);

          // 3. Signal arch layer to flush I-cache
          request_icache_flush();

          self.morph_count += 1;
      }

      /// Resolve a dispatch slot to a function index.
      pub fn resolve(&self, slot: usize) -> Option<usize>      { self.dispatch.resolve(slot) }

      /// Get the current byte offset for a capability-gate check.
      pub fn gate_offset(&self, check_id: usize) -> Option<u32> { self.cap_gates.offset_of(check_id) }

      /// Total morph passes performed.
      pub fn morph_count(&self) -> u64 { self.morph_count }

      /// Reseed the PRNG (e.g. from periodic hardware entropy collection).
      pub fn reseed(&mut self, entropy: u64) {
          let current = self.rng.next();
          self.rng = XorShift64::new(current ^ entropy);
      }
  }
  