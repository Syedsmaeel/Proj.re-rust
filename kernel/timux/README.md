# Timux

**A sovereign kernel with a hybrid ring + capability privilege model.**

> The OS you run is not neutral. Linux inherits Unix's 1970s privilege model. Windows is a corporate black box. Timux owns the model — from ring definition to capability minting.

**Author:** Syed Ismaeel — Lucknow, Est. 2019  
**License:** AGPL-3.0  
**Targets:** x86_64 · RISC-V · ARM64

---

## Privilege Model

Timux does not use Intel's Ring 0-3. It defines its own:

| Ring | Name | Who lives here | Default rights |
|---|---|---|---|
| Ring 0 | Kernel Core | Timux kernel | Unrestricted |
| Ring 1 | Kernel Extension | Drivers, filesystems | Supervised — capability-gated |
| Ring 2 | System Service | init, IPC daemons | Capability-isolated |
| Ring 3 | User | Applications | Fully capability-isolated |
| Ring 4 | Sandbox | Untrusted code | Maximum restriction |

Every cross-ring operation requires a valid `CapabilityToken`. No capability = no access, regardless of ring level. Capabilities are unforgeable, delegatable, and revocable.

## Capability Rights

```
READ · WRITE · EXEC · MAP · SEND · RECV · DELEGATE · REVOKE
RING_TRANSITION · PRIVILEGE_ESCALATION · IRQ_BIND · DMA_ACCESS
MMIO_ACCESS · PROCESS_SPAWN · PROCESS_KILL · FS_READ · FS_WRITE
NET_SEND · NET_RECV · CLOCK_READ · CLOCK_SET
```

## Architecture

```
kernel/timux/
├── src/
│   ├── priv_model.rs   — Ring hierarchy + CapabilityToken + CapabilityTable
│   ├── cap.rs          — CapAuthority: minting, verification, presets
│   ├── mm.rs           — Virtual memory, capability-gated mapping
│   ├── sched.rs        — Ring-aware preemptive scheduler
│   ├── ipc.rs          — Capability-gated message passing
│   ├── boot.rs         — Arch-independent early init + KernelState
│   └── arch/
│       ├── x86_64/     — GDT, IDT, paging
│       ├── riscv/      — trap handling, Sv39 paging
│       └── arm64/      — MMU, exception vectors
```

## Build

```bash
# Requires nightly Rust + target toolchain
rustup target add x86_64-unknown-none riscv64gc-unknown-none-elf aarch64-unknown-none

cargo build -p timux
```

AGPL-3.0 — Syed Ismaeel, Lucknow Est. 2019
