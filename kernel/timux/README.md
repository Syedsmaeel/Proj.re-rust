# Timux

**A sovereign kernel with a hybrid ring + capability privilege model, full sub-kernel system, and a built-in TUI shell.**

> The OS you run is not neutral. Linux inherits Unix's 1970s privilege model. Windows is a corporate black box. Timux owns the model — from ring definition to capability minting, from kernel to shell.

**Author:** Syed Ismaeel — Lucknow, Est. 2019  
**License:** AGPL-3.0  
**Language:** Rust (`#![no_std]`)  
**Targets:** x86_64 · RISC-V · ARM64

---

## What Timux Is

Timux is not a Linux fork. It is not a Unix clone. It is a ground-up sovereign kernel written in Rust with three core ideas:

**1. You own the privilege model.**  
Linux uses Intel's Ring 0 and Ring 3. Rings 1 and 2 sit empty. Drivers run in Ring 0 — same privilege as the kernel. One bad driver corrupts everything. Timux defines its own 5-ring hierarchy where every cross-ring call requires an unforgeable capability token.

**2. You own the OS instances.**  
Timux runs multiple isolated sub-kernels inside the master kernel. Each sub-kernel is a complete OS — its own scheduler, memory manager, drivers, filesystem, and capability space. They are hot-swappable, cloneable, and migratable at runtime.

**3. You own the shell.**  
Bash++ is a ring-2 sub-kernel service. It has its own language, runs bash-compatible `.sh` scripts, and renders a full TUI — tabs, split panes, widgets, icons, themes.

---

## Privilege Model

Timux is the **Ring -1 Software-Defined Substrate**. It establishes a sovereign fractal hierarchy where all isolation is logically enforced:

| Ring | Name | Who lives here | Default rights |
|---|---|---|---|
| Ring -1 | Sovereign Substrate | Master Kernel Core | Absolute Root |
| Ring 0 | Kernel Core | Sub-kernel kernel | Unrestricted (within SK) |
| Ring 1 | Kernel Extension | Drivers, filesystems | Supervised — capability-gated |
| Ring 2 | System Service | init, IPC daemons, Bash++ | Capability-isolated |
| Ring 3 | User | Applications | Fully capability-isolated |
| Ring 4 | Sandbox | Untrusted code | Maximum restriction |

Every cross-ring and cross-kernel operation requires a valid `CapabilityToken`.

---

## Architecture Diagram

```
          RING -1: SOVEREIGN SUBSTRATE
         CapAuthority · SubKernelManager · IPC broker · Blueprint Parser
                              spawn(cap)
              |                    |                    |
              ▼                    ▼                    ▼
┌─────────────────────┐  ┌─────────────────────┐  ┌─────────────────────┐
│    SUB-KERNEL 1     │  │    SUB-KERNEL 2     │  │    SUB-KERNEL 3     │
│  (script: net.sh)   │  │  (script: fs.sh)    │  │  (NESTED BASE -1)   │
│─────────────────────│  │─────────────────────│  │─────────────────────│
│  ring 0 — sk-core   │  │  ring 0 — sk-core   │  │  ring 0 — sk-core   │
│  ring 1 — sk-ext    │  │  ring 1 — sk-ext    │  │  ring 1 — sk-ext    │
│  ring 2 — sk-svc    │◄─►  ring 2 — sk-svc    │◄─►  ring 2 — sk-svc    │
│  ring 3 — user      │IPC│  ring 3 — user      │IPC│  ring 3 — user      │
│  ring 4 — sandbox   │  │  ring 4 — sandbox   │  │  ring 4 — sandbox   │
│─────────────────────│  │─────────────────────│  │─────────────────────│
│     cap table       │  │     cap table       │  │     cap table       │
└──────────┬──────────┘  └──────────┬──────────┘  └──────────┬──────────┘
           ╎                        ╎                        ╎
           ╎  (MAP cap)             ╎  (MAP cap)             ╎  (NESTED spawn)
           ▼                        ▼                        ▼
┌──────────────────────────────────────────────────────────────────────────┐
│                        SOVEREIGN LOGIC BRIDGES                           │
│     Software-Defined IPC · Capability-Gated Shared Memory Segments       │
└──────────────────────────────────────────────────────────────────────────┘
```


```
kernel/timux/
├── src/
│   ├── lib.rs              — crate root, #![no_std]
│   ├── priv_model.rs       — RingLevel (0-4), CapabilityToken, CapRight (21 rights)
│   ├── cap.rs              — CapAuthority: mint_root/user/sandbox/driver + verify()
│   ├── ipc.rs              — Channel: capability-gated send/recv + cap passing
│   ├── sched.rs            — Ring-aware scheduler
│   ├── boot.rs             — BootInfo, KernelState::init(), global ALLOCATOR
│   ├── mm/
│   │   ├── mod.rs          — AddressSpace: capability-gated map() + map_mmio()
│   │   └── allocator.rs    — LinkedListAllocator: bare-metal heap
│   ├── subkernel/
│   │   ├── instance.rs     — SubKernel: full OS instance + all lifecycle ops
│   │   ├── manager.rs      — SubKernelManager: master control plane
│   │   ├── bridge.rs       — IPC + shared memory bridges
│   │   └── snapshot.rs     — state capture for clone/migrate/checkpoint
│   ├── bashpp/
│   │   ├── lexer/          — 50+ token kinds, Bash++ + bash-compat
│   │   ├── parser/         — recursive descent AST parser
│   │   ├── runtime/        — tree-walking interpreter, scoped env, built-ins
│   │   ├── tui/            — tabs, panes, widgets, 6 themes, ANSI rendering
│   │   └── compat/         — bash .sh preprocessor + runner
│   └── arch/
│       ├── x86_64/         — #[global_allocator], #[panic_handler], hlt
│       ├── riscv/          — OpenSBI handoff, wfi halt
│       └── arm64/          — DRAM base, wfe halt
```

---

## Test Results

80 / 80 tests passing across 8 subsystems:

| Subsystem | Tests | Status |
|---|---|---|
| Ring Model | 9 | ✅ |
| Capability Tokens | 16 | ✅ |
| Capability Table | 6 | ✅ |
| Scheduler | 5 | ✅ |
| IPC | 8 | ✅ |
| Sub-Kernel Profiles | 10 | ✅ |
| Sub-Kernel Lifecycle | 20 | ✅ |
| Sub-Kernel Ring Isolation | 6 | ✅ |

Bash++ live test: lexer · parser · runtime · tui · compat — all verified.

---

## Build

```bash
# Requires Rust stable 1.75+
rustup target add x86_64-unknown-none
rustup target add riscv64gc-unknown-none-elf
rustup target add aarch64-unknown-none

# Build the library
cargo build -p timux

# Build for bare metal
cargo build -p timux --target x86_64-unknown-none
cargo build -p timux --target riscv64gc-unknown-none-elf
cargo build -p timux --target aarch64-unknown-none
```

---

## Roadmap

- [ ] `arch/x86_64/gdt.rs` — Global Descriptor Table
- [ ] `arch/x86_64/idt.rs` — Interrupt Descriptor Table + IRQ routing
- [ ] `arch/x86_64/paging.rs` — 4-level page tables
- [ ] `arch/riscv/trap.rs` — trap handler, Sv39 paging
- [ ] `arch/arm64/mmu.rs` — ARMv8 MMU, exception vectors
- [ ] Physical memory manager (buddy allocator)
- [ ] Bash++ live filesystem access via storage sub-kernel
- [ ] Sub-kernel networking stack (ring-1 driver + ring-2 service)
- [ ] UEFI boot support

---

*AGPL-3.0 — Syed Ismaeel, Lucknow Est. 2019*
