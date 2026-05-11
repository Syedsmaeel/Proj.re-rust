//! RISC-V (RV64GC) architecture implementation

  use super::Arch;

  pub struct RiscV;

  impl Arch for RiscV {
      fn name() -> &'static str { "riscv64gc-unknown-none-elf" }

      fn early_init() {
          // Disable all interrupts in mie register, configure mtvec
          unsafe {
              // Clear MIE (machine interrupt enable) in mstatus
              core::arch::asm!("csrci mstatus, 8", options(nomem, nostack));
          }
      }

      fn halt() -> ! {
          loop { unsafe { core::arch::asm!("wfi", options(nomem, nostack)); } }
      }

      unsafe fn interrupts_enable() {
          // Set MIE bit in mstatus (bit 3)
          core::arch::asm!("csrsi mstatus, 8", options(nomem, nostack));
      }

      unsafe fn interrupts_disable() {
          // Clear MIE bit in mstatus (bit 3)
          core::arch::asm!("csrci mstatus, 8", options(nomem, nostack));
      }

      fn interrupts_enabled() -> bool {
          let mstatus: usize;
          unsafe {
              core::arch::asm!("csrr {}, mstatus", out(reg) mstatus, options(nomem, nostack));
          }
          // MIE is bit 3 of mstatus
          (mstatus >> 3) & 1 == 1
      }

      unsafe fn flush_tlb() {
          // sfence.vma with no arguments flushes all address spaces
          core::arch::asm!("sfence.vma", options(nomem, nostack));
      }

      unsafe fn flush_tlb_page(vaddr: usize) {
          // sfence.vma rs1, x0 — flush entries for the given virtual address
          core::arch::asm!(
              "sfence.vma {va}, x0",
              va = in(reg) vaddr,
              options(nomem, nostack)
          );
      }

      fn read_tsc() -> u64 {
          let cycles: u64;
          unsafe {
              core::arch::asm!("rdcycle {}", out(reg) cycles, options(nomem, nostack));
          }
          cycles
      }

      fn memory_barrier() {
          unsafe { core::arch::asm!("fence rw, rw", options(nomem, nostack)); }
      }

      fn load_fence() {
          unsafe { core::arch::asm!("fence r, r", options(nomem, nostack)); }
      }

      fn store_fence() {
          unsafe { core::arch::asm!("fence w, w", options(nomem, nostack)); }
      }

      fn nop() {
          unsafe { core::arch::asm!("nop", options(nomem, nostack)); }
      }

      unsafe fn out8(_port: u16, _val: u8)  { /* RISC-V uses MMIO */ }
      unsafe fn in8(_port: u16) -> u8       { 0 }
  }

  // ─── RISC-V UART (NS16550 compatible, common on QEMU virt) ───────────────────

  const UART_BASE: usize = 0x1000_0000; // QEMU virt machine UART

  pub fn uart_putb(b: u8) {
      unsafe {
          // Poll THR empty (LSR bit 5)
          while (core::ptr::read_volatile((UART_BASE + 5) as *const u8) & 0x20) == 0 {}
          core::ptr::write_volatile(UART_BASE as *mut u8, b);
      }
  }

  pub fn uart_puts(s: &str) {
      for b in s.bytes() {
          if b == b'\n' { uart_putb(b'\r'); }
          uart_putb(b);
      }
  }
  