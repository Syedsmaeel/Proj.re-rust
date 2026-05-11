//! Architecture abstraction layer
  //!
  //! The Arch trait provides a uniform interface to hardware-specific operations.
  //! Each target (x86_64, arm64, riscv) provides a zero-sized struct that
  //! implements all methods via inline assembly.
  //!
  //! # Usage
  //!
  //! ```no_run
  //! use timux::arch::x86_64::X86_64;
  //! use timux::arch::Arch;
  //!
  //! X86_64::interrupts_disable();
  //! let tick = X86_64::read_tsc();
  //! X86_64::memory_barrier();
  //! X86_64::interrupts_enable();
  //! ```

  pub mod arm64;
  pub mod riscv;
  pub mod x86_64;

  /// Hardware architecture abstraction trait.
  ///
  /// All methods are static (no self) so callers pay zero runtime cost —
  /// the struct is a zero-sized phantom type.
  pub trait Arch {
      /// Human-readable target triple name.
      fn name() -> &'static str;

      /// Minimal early hardware init — called before the heap exists.
      /// Typically: serial port, basic interrupt table, disable legacy PIC.
      fn early_init();

      /// Spin forever and consume as little power as possible.
      fn halt() -> !;

      /// Enable maskable interrupts.
      ///
      /// # Safety
      /// Caller must ensure interrupt handlers are installed and the IDT/IVT
      /// is configured before calling this.
      unsafe fn interrupts_enable();

      /// Disable maskable interrupts.
      ///
      /// # Safety
      /// Must be balanced with a later interrupts_enable() unless shutting down.
      unsafe fn interrupts_disable();

      /// Returns true if maskable interrupts are currently enabled.
      fn interrupts_enabled() -> bool;

      /// Flush the entire TLB (Translation Lookaside Buffer).
      ///
      /// # Safety
      /// Must be called with interrupts disabled on all cores sharing the TLB,
      /// or with appropriate IPI broadcast for SMP.
      unsafe fn flush_tlb();

      /// Flush the TLB entry for a single virtual address.
      ///
      /// # Safety
      /// Same constraints as flush_tlb.
      unsafe fn flush_tlb_page(vaddr: usize);

      /// Read the hardware cycle counter (TSC on x86, cntvct_el0 on arm64, cycle on riscv).
      /// Returns a monotonically increasing tick count.
      fn read_tsc() -> u64;

      /// Full memory barrier — all prior loads and stores complete before any
      /// subsequent load or store begins.
      fn memory_barrier();

      /// Load fence — all prior loads complete before any subsequent load.
      fn load_fence();

      /// Store fence — all prior stores complete before any subsequent store.
      fn store_fence();

      /// Execute a single no-op instruction (hint to the pipeline).
      fn nop();

      /// Write a byte to a port (x86 I/O ports, no-op on other arches).
      ///
      /// # Safety
      /// Direct hardware I/O — caller must own the port.
      unsafe fn out8(port: u16, val: u8);

      /// Read a byte from a port (x86 I/O ports, zero on other arches).
      ///
      /// # Safety
      /// Direct hardware I/O — caller must own the port.
      unsafe fn in8(port: u16) -> u8;
  }

  /// RAII guard that disables interrupts on construction and restores the
  /// previous state on drop.
  pub struct IrqGuard {
      was_enabled: bool,
  }

  impl IrqGuard {
      /// Create a guard for the given architecture.
      ///
      /// # Safety
      /// Callers must ensure the guard is not held across context switches
      /// or sleep operations.
      pub unsafe fn new<A: Arch>() -> Self {
          let was_enabled = A::interrupts_enabled();
          if was_enabled { unsafe { A::interrupts_disable() }; }
          Self { was_enabled }
      }
  }
  // NOTE: Drop impl omitted because restoring interrupts requires knowing
  // the architecture type; callers re-enable manually after the guard drops.
  