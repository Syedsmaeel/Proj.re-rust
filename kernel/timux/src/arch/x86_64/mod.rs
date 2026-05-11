//! x86_64 architecture implementation

  use super::Arch;

  pub struct X86_64;

  impl Arch for X86_64 {
      fn name() -> &'static str { "x86_64-unknown-none" }

      fn early_init() {
          // Disable legacy PIC (8259) — we use APIC
          unsafe {
              // Mask all PIC1 and PIC2 interrupts
              Self::out8(0xA1, 0xFF);
              Self::out8(0x21, 0xFF);
          }
      }

      fn halt() -> ! {
          loop { unsafe { core::arch::asm!("hlt", options(nomem, nostack)); } }
      }

      unsafe fn interrupts_enable() {
          core::arch::asm!("sti", options(nomem, nostack));
      }

      unsafe fn interrupts_disable() {
          core::arch::asm!("cli", options(nomem, nostack));
      }

      fn interrupts_enabled() -> bool {
          let flags: u64;
          unsafe {
              core::arch::asm!(
                  "pushfq",
                  "pop {flags}",
                  flags = out(reg) flags,
                  options(nomem, preserves_flags)
              );
          }
          // Bit 9 of RFLAGS = IF (Interrupt Flag)
          (flags >> 9) & 1 == 1
      }

      unsafe fn flush_tlb() {
          // Reload CR3 with itself — flushes all non-global TLB entries
          let cr3: u64;
          core::arch::asm!("mov {}, cr3", out(reg) cr3, options(nomem, nostack));
          core::arch::asm!("mov cr3, {}", in(reg) cr3, options(nomem, nostack));
      }

      unsafe fn flush_tlb_page(vaddr: usize) {
          core::arch::asm!("invlpg [{addr}]", addr = in(reg) vaddr, options(nostack));
      }

      fn read_tsc() -> u64 {
          let lo: u32;
          let hi: u32;
          unsafe {
              core::arch::asm!(
                  "lfence",       // serialise before RDTSC
                  "rdtsc",
                  out("eax") lo,
                  out("edx") hi,
                  options(nomem, nostack),
              );
          }
          ((hi as u64) << 32) | lo as u64
      }

      fn memory_barrier() {
          unsafe { core::arch::asm!("mfence", options(nomem, nostack)); }
      }

      fn load_fence() {
          unsafe { core::arch::asm!("lfence", options(nomem, nostack)); }
      }

      fn store_fence() {
          unsafe { core::arch::asm!("sfence", options(nomem, nostack)); }
      }

      fn nop() {
          unsafe { core::arch::asm!("nop", options(nomem, nostack)); }
      }

      unsafe fn out8(port: u16, val: u8) {
          core::arch::asm!(
              "out dx, al",
              in("dx") port,
              in("al") val,
              options(nomem, nostack)
          );
      }

      unsafe fn in8(port: u16) -> u8 {
          let val: u8;
          core::arch::asm!(
              "in al, dx",
              out("al") val,
              in("dx") port,
              options(nomem, nostack)
          );
          val
      }
  }

  // ─── x86_64 serial port (COM1) for early debugging ───────────────────────────

  const COM1: u16 = 0x3F8;

  pub fn serial_init() {
      unsafe {
          X86_64::out8(COM1 + 1, 0x00); // disable interrupts
          X86_64::out8(COM1 + 3, 0x80); // enable DLAB
          X86_64::out8(COM1 + 0, 0x03); // baud divisor low  (38400)
          X86_64::out8(COM1 + 1, 0x00); // baud divisor high
          X86_64::out8(COM1 + 3, 0x03); // 8 bits, no parity, 1 stop
          X86_64::out8(COM1 + 2, 0xC7); // enable+clear FIFO, 14-byte threshold
          X86_64::out8(COM1 + 4, 0x0B); // RTS/DSR set
      }
  }

  fn serial_wait() {
      // Wait until transmit-holding-register is empty
      while unsafe { X86_64::in8(COM1 + 5) } & 0x20 == 0 {}
  }

  pub fn serial_putb(b: u8) {
      serial_wait();
      unsafe { X86_64::out8(COM1, b); }
      if b == b'\n' {
          serial_wait();
          unsafe { X86_64::out8(COM1, b'\r'); }
      }
  }

  pub fn serial_puts(s: &str) {
      for b in s.bytes() { serial_putb(b); }
  }
  