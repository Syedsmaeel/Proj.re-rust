; Minimal x86_64 Sovereign Bootstrap
; This is the "Seed" that Hoffman grafts onto foreign kernels.

[bits 64]
global _start

_start:
    ; 1. Setup minimal stack
    ; 2. Initialize .tmx-disk storage
    ; 3. Jump to the grafted kernel entry point
    
    ; Placeholder: Jump to entry point (will be patched by Hoffman Builder)
    jmp 0x0
