//! ARM64 (AArch64) architecture implementation

  use super::Arch;

  pub struct Arm64;

  impl Arch for Arm64 {
      fn name() -> &'static str { "aarch64-unknown-none-softfloat" }

      fn early_init() {
          // Disable FIQ/IRQ masking from EL1 startup default
          // (done via DAIF — handled by interrupts_enable)
      }

      fn halt() -> ! {
          loop { unsafe { core::arch::asm!("wfi", options(nomem, nostack)); } }
      }

      unsafe fn interrupts_enable() {
          // Clear DAIF.I bit to unmask IRQ
          core::arch::asm!("msr daifclr, #2", options(nomem, nostack));
      }

      unsafe fn interrupts_disable() {
          // Set DAIF.I bit to mask IRQ
          core::arch::asm!("msr daifset, #2", options(nomem, nostack));
      }

      fn interrupts_enabled() -> bool {
          let daif: u64;
          unsafe {
              core::arch::asm!("mrs {}, daif", out(reg) daif, options(nomem, nostack));
          }
          // Bit 7 (I) of DAIF: 0 = IRQ unmasked (enabled), 1 = masked
          (daif >> 7) & 1 == 0
      }

      unsafe fn flush_tlb() {
          // Invalidate all TLB entries for EL1 across the Inner Shareable domain
          core::arch::asm!(
              "dsb ishst",         // ensure stores are visible
              "tlbi vmalle1is",    // invalidate all EL1 entries IS
              "dsb ish",           // ensure TLB invalidation is complete
              "isb",               // sync instruction stream
              options(nomem, nostack)
          );
      }

      unsafe fn flush_tlb_page(vaddr: usize) {
          core::arch::asm!(
              "dsb ishst",
              "tlbi vaae1is, {va}",
              "dsb ish",
              "isb",
              va = in(reg) (vaddr >> 12), // VA[55:12]
              options(nomem, nostack)
          );
      }

      fn read_tsc() -> u64 {
          let tsc: u64;
          unsafe {
              core::arch::asm!(
                  "isb",
                  "mrs {}, cntvct_el0",
                  out(reg) tsc,
                  options(nomem, nostack)
              );
          }
          tsc
      }

      fn memory_barrier() {
          unsafe { core::arch::asm!("dmb sy", options(nomem, nostack)); }
      }

      fn load_fence() {
          unsafe { core::arch::asm!("dmb ld", options(nomem, nostack)); }
      }

      fn store_fence() {
          unsafe { core::arch::asm!("dmb st", options(nomem, nostack)); }
      }

      fn nop() {
          unsafe { core::arch::asm!("nop", options(nomem, nostack)); }
      }

      unsafe fn out8(_port: u16, _val: u8) {
          // ARM64 has no I/O port space; MMIO used instead
      }

      unsafe fn in8(_port: u16) -> u8 { 0 }
  }
  