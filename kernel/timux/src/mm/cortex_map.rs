//! Cortex Memory Mapper
  //!
  //! Maps semantic object handles to physical backing addresses.
  //! v2 adds:
  //!  - LRU eviction when the registry reaches capacity
  //!  - Volatile-flag GC (volatile objects reaped first under pressure)
  //!  - Per-object access counters and last-access timestamps
  //!  - Pinned objects immune to eviction
  //!  - Aggregate statistics
  extern crate alloc;

  use alloc::collections::BTreeMap;
  use alloc::vec::Vec;
  use spin::Mutex;
  use crate::mm::cortex::MemoryContext;

  // ─── Types ────────────────────────────────────────────────────────────────────

  pub type ObjectHandle = u64;

  // ─── CortexEntry ─────────────────────────────────────────────────────────────

  #[derive(Debug, Clone)]
  pub struct CortexEntry {
      pub ctx:          MemoryContext,
      pub phys_addr:    u64,
      /// Reapable under memory pressure before LRU applies.
      pub volatile:     bool,
      /// Immune to both GC and LRU eviction.
      pub pinned:       bool,
      pub access_count: u64,
      /// Monotonic tick at last access (used for LRU ordering).
      pub last_access:  u64,
      pub size_bytes:   u64,
  }

  impl CortexEntry {
      fn new(ctx: MemoryContext, phys_addr: u64) -> Self {
          Self {
              ctx, phys_addr,
              volatile: false, pinned: false,
              access_count: 0, last_access: 0,
              size_bytes: 0,
          }
      }
  }

  // ─── Stats ────────────────────────────────────────────────────────────────────

  #[derive(Debug, Default, Clone, Copy)]
  pub struct CortexStats {
      pub inserts:        u64,
      pub lookups:        u64,
      pub evictions_lru:  u64,
      pub evictions_gc:   u64,
      pub misses:         u64,
      pub pinned_count:   u64,
      pub volatile_count: u64,
      pub total_bytes:    u64,
  }

  // ─── Inner (unsynchronised) ───────────────────────────────────────────────────

  struct Inner {
      registry: BTreeMap<ObjectHandle, CortexEntry>,
      capacity: usize,
      tick:     u64,
      stats:    CortexStats,
  }

  impl Inner {
      fn new(capacity: usize) -> Self {
          Self { registry: BTreeMap::new(), capacity, tick: 0, stats: CortexStats::default() }
      }

      fn next_tick(&mut self) -> u64 { self.tick += 1; self.tick }

      // ── Volatile GC ───────────────────────────────────────────────────────────

      fn gc_volatile(&mut self) -> usize {
          let victims: Vec<ObjectHandle> = self.registry.iter()
              .filter(|(_, e)| e.volatile && !e.pinned)
              .map(|(h, _)| *h)
              .collect();
          let count = victims.len();
          for h in victims {
              if let Some(e) = self.registry.remove(&h) {
                  self.stats.total_bytes  = self.stats.total_bytes.saturating_sub(e.size_bytes);
                  self.stats.evictions_gc += 1;
              }
          }
          count
      }

      // ── LRU eviction ─────────────────────────────────────────────────────────

      fn evict_lru(&mut self) -> bool {
          let victim = self.registry.iter()
              .filter(|(_, e)| !e.pinned)
              .min_by_key(|(_, e)| e.last_access)
              .map(|(h, _)| *h);
          if let Some(h) = victim {
              if let Some(e) = self.registry.remove(&h) {
                  self.stats.total_bytes    = self.stats.total_bytes.saturating_sub(e.size_bytes);
                  self.stats.evictions_lru += 1;
                  return true;
              }
          }
          false
      }

      // ── Ensure capacity ───────────────────────────────────────────────────────

      fn ensure_capacity(&mut self) {
          while self.registry.len() >= self.capacity {
              if self.gc_volatile() > 0 { continue; }
              if !self.evict_lru() { break; }
          }
      }
  }

  // ─── CortexMap ────────────────────────────────────────────────────────────────

  /// Kernel-global semantic object registry.
  ///
  /// Default capacity: 4 096 entries.
  /// Eviction order: volatile GC first, then LRU (pinned entries are immune).
  pub struct CortexMap {
      inner: Mutex<Inner>,
  }

  impl CortexMap {
      pub const DEFAULT_CAPACITY: usize = 4096;

      pub fn new() -> Self { Self::with_capacity(Self::DEFAULT_CAPACITY) }

      pub fn with_capacity(cap: usize) -> Self {
          Self { inner: Mutex::new(Inner::new(cap)) }
      }

      // ── Map ───────────────────────────────────────────────────────────────────

      /// Register a handle with a context and physical address.
      pub fn map(&self, handle: ObjectHandle, ctx: MemoryContext, addr: u64) {
          let mut g = self.inner.lock();
          g.ensure_capacity();
          let tick = g.next_tick();
          let mut e = CortexEntry::new(ctx, addr);
          e.last_access = tick;
          g.registry.insert(handle, e);
          g.stats.inserts += 1;
      }

      /// Register with full metadata.
      pub fn map_opts(
          &self, handle: ObjectHandle, ctx: MemoryContext, addr: u64,
          size_bytes: u64, volatile: bool, pinned: bool,
      ) {
          let mut g = self.inner.lock();
          g.ensure_capacity();
          let tick = g.next_tick();
          g.stats.total_bytes   += size_bytes;
          if pinned   { g.stats.pinned_count   += 1; }
          if volatile { g.stats.volatile_count += 1; }
          g.registry.insert(handle, CortexEntry {
              ctx, phys_addr: addr,
              volatile, pinned,
              access_count: 0, last_access: tick, size_bytes,
          });
          g.stats.inserts += 1;
      }

      // ── Resolve ───────────────────────────────────────────────────────────────

      /// Look up a handle and return (ctx, phys_addr), bumping access stats.
      pub fn resolve(&self, handle: ObjectHandle) -> Option<(MemoryContext, u64)> {
          let mut g = self.inner.lock();
          g.stats.lookups += 1;
          if let Some(e) = g.registry.get_mut(&handle) {
              g.tick += 1;
              e.access_count += 1;
              e.last_access   = g.tick;
              Some((e.ctx.clone(), e.phys_addr))
          } else {
              g.stats.misses += 1;
              None
          }
      }

      // ── Unmap ─────────────────────────────────────────────────────────────────

      pub fn unmap(&self, handle: ObjectHandle) -> bool {
          let mut g = self.inner.lock();
          if let Some(e) = g.registry.remove(&handle) {
              g.stats.total_bytes = g.stats.total_bytes.saturating_sub(e.size_bytes);
              true
          } else { false }
      }

      // ── Pin / unpin ───────────────────────────────────────────────────────────

      pub fn pin(&self, handle: ObjectHandle) -> bool {
          let mut g = self.inner.lock();
          if let Some(e) = g.registry.get_mut(&handle) { e.pinned = true;  true } else { false }
      }

      pub fn unpin(&self, handle: ObjectHandle) -> bool {
          let mut g = self.inner.lock();
          if let Some(e) = g.registry.get_mut(&handle) { e.pinned = false; true } else { false }
      }

      pub fn mark_volatile(&self, handle: ObjectHandle) -> bool {
          let mut g = self.inner.lock();
          if let Some(e) = g.registry.get_mut(&handle) { e.volatile = true; true } else { false }
      }

      // ── Pressure relief ───────────────────────────────────────────────────────

      /// Manually trigger volatile GC; returns number of entries freed.
      pub fn gc(&self) -> usize { self.inner.lock().gc_volatile() }

      /// Manually evict the LRU non-pinned entry.
      pub fn evict_one(&self) -> bool { self.inner.lock().evict_lru() }

      // ── Observability ─────────────────────────────────────────────────────────

      pub fn stats(&self)    -> CortexStats { self.inner.lock().stats }
      pub fn len(&self)      -> usize       { self.inner.lock().registry.len() }
      pub fn is_empty(&self) -> bool        { self.len() == 0 }
      pub fn capacity(&self) -> usize       { self.inner.lock().capacity }
  }
  