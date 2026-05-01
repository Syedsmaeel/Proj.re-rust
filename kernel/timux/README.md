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

Timux does not use Intel's Ring 0-3. It defines its own:

| Ring | Name | Who lives here | Default rights |
|---|---|---|---|
| Ring 0 | Kernel Core | Timux kernel | Unrestricted |
| Ring 1 | Kernel Extension | Drivers, filesystems | Supervised — capability-gated |
| Ring 2 | System Service | init, IPC daemons, Bash++ | Capability-isolated |
| Ring 3 | User | Applications | Fully capability-isolated |
| Ring 4 | Sandbox | Untrusted code | Maximum restriction |

Every cross-ring operation requires a valid `CapabilityToken`. No capability = no access, regardless of ring level. Capabilities are unforgeable, delegatable, and revocable.

### Capability Rights (21 total)

```
READ · WRITE · EXEC · MAP · SEND · RECV · DELEGATE · REVOKE
RING_TRANSITION · PRIVILEGE_ESCALATION · IRQ_BIND · DMA_ACCESS
MMIO_ACCESS · PROCESS_SPAWN · PROCESS_KILL · FS_READ · FS_WRITE
NET_SEND · NET_RECV · CLOCK_READ · CLOCK_SET
```

---

## Sub-Kernel System

Each sub-kernel is a complete isolated OS instance. Not a container. Not a VM. A full sovereign OS with its own scheduler, memory manager, driver registry, and capability table.

### Pre-built OS profiles

| Profile | Description | Pre-loaded rights |
|---|---|---|
| `GeneralPurpose` | Default full-featured OS | All standard rights |
| `RealTime` | Strict scheduling guarantees | CLOCK_SET, IRQ_BIND |
| `Networking` | Network OS | NET_SEND, NET_RECV, DMA |
| `Storage` | Filesystem OS | FS_READ, FS_WRITE, DMA |
| `Graphics` | GPU OS | DMA, MMIO, IRQ_BIND |
| `Enclave` | Maximum isolation | READ, EXEC only |
| `Custom` | Define everything | You decide |

### Lifecycle operations

```rust
manager.spawn(&cap, config, parent)      // create a new OS instance
manager.pause(&cap, id)                  // freeze — state preserved
manager.resume(&cap, id)                 // unfreeze and continue
manager.hot_swap(&cap, id, new_config)   // replace running image, no kill
manager.clone_sk(&cap, id)              // duplicate a sub-kernel
manager.migrate(&cap, id)               // freeze + transport to new location
manager.terminate(&cap, id)             // clean shutdown
manager.bridge_ipc(&cap, from, to)      // IPC channel between sub-kernels
manager.bridge_shm(&cap, from, to, ..)  // shared memory region
manager.snapshot(&cap, id)              // capture full state
```

Sub-kernels can spawn other sub-kernels — recursively nested OS instances, all capability-gated.

---

## Heap Allocator

Timux runs on bare metal. No OS beneath it. No `malloc`. No libc.

The heap allocator (`mm/allocator.rs`) is a linked-list allocator written from scratch:

- `ALLOCATOR.init(heap_start, heap_size)` — called once at boot
- `alloc()` — walks free list, finds fitting node, splits, returns aligned pointer
- `dealloc()` — returns block to free list
- No external crates. No GC. No runtime.

Heap locations per arch:

| Arch | Heap base | Notes |
|---|---|---|
| x86_64 | `0x10_0000` | 1MB physical |
| RISC-V | `0x8020_0000` | After OpenSBI handoff |
| ARM64 | `0x4000_0000` | Typical DRAM start |

---

## Bash++ — Sovereign TUI Shell

Bash++ is a ring-2 sub-kernel service running inside Timux as a `SystemService`.

### Two languages, one shell

**Bash++ native syntax:**
```bash
let name := "Timux"
fn greet(who) {
    render "hello $who" color=cyan
}
greet $name
icon "🦀"
theme dracula

for item in $list { echo $item }

match $status {
    "ok"  => render "all good" color=green
    "err" => render "failed"   color=red
}
```

**Bash-compatible** — `.sh` scripts run unmodified:
```bash
#!/bin/bash
function deploy() {
    local env="production"
    echo "Deploying to $env"
}
export VERSION=1.0
deploy
```

### TUI capabilities

```bash
pane split=vertical ratio=60 { widget filetree /timux }
tab "shell" "🐚" { ... }
tab "files" "📁" { ... }
widget clock
widget process
widget editor myfile.rs
widget palette
theme dracula | nord | gruvbox | catppuccin | solarized
render "🦀 Timux" color=cyan
border style=rounded
```

### Built-in widgets

| Widget | Description |
|---|---|
| `clock` | Live clock display |
| `filetree <path>` | File tree navigator |
| `process` | Process viewer (PID, CPU, MEM) |
| `editor <file>` | Text editor |
| `palette` | Command palette |
| `status` | Status bar |

---

## Architecture


## Architecture Diagram

```
                    master kernel — ring 0
         CapAuthority · SubKernelManager · IPC broker · shared memory arbiter
                              spawn(cap)
              |                    |                    |
              ▼                    ▼                    ▼
┌─────────────────────┐  ┌─────────────────────┐  ┌─────────────────────┐
│    sub-kernel 1     │  │    sub-kernel 2     │  │    sub-kernel 3     │
│     net-kernel      │  │     fs-kernel       │  │     gpu-kernel      │
│─────────────────────│  │─────────────────────│  │─────────────────────│
│  ring 0 — sk-core   │  │  ring 0 — sk-core   │  │  ring 0 — sk-core   │
│  ring 1 — sk-ext    │  │  ring 1 — sk-ext    │  │  ring 1 — sk-ext    │
│  ring 2 — sk-svc    │◄─►  ring 2 — sk-svc    │◄─►  ring 2 — sk-svc    │
│  ring 3 — user      │IPC│  ring 3 — user      │IPC│  ring 3 — user      │
│  ring 4 — sandbox   │  │  ring 4 — sandbox   │  │  ring 4 — sandbox   │
│─────────────────────│  │─────────────────────│  │─────────────────────│
│     cap table       │  │     cap table       │  │     cap table       │
│  NET_SEND·NET_RECV  │  │  FS_READ·FS_WRITE   │  │  DMA·MMIO_ACCESS    │
└──────────┬──────────┘  └──────────┬──────────┘  └──────────┬──────────┘
           ╎                        ╎                        ╎
           ╎  (MAP cap)             ╎  (MAP cap)             ╎  (MAP cap)
           ▼                        ▼                        ▼
┌──────────────────────────────────────────────────────────────────────────┐
│                        shared memory region                              │
│  MAP cap required · page-table entries controlled by master kernel       │
│              read-only or read-write per capability token                │
└──────────────────────────────────────────────────────────────────────────┘

Legend:
  ◄─► ─────  IPC channel          (SEND + RECV cap required)
  ╎ - - -    shared memory        (MAP cap required)
  spawn(cap) master kernel or sub-kernel creates a new OS instance
             each sub-kernel has its own fully isolated ring 0–4
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
