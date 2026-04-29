pub mod arm64;
pub mod riscv;
pub mod x86_64;
pub trait Arch {
    fn name() -> &'static str;
    fn early_init();
    fn halt() -> !;
}
