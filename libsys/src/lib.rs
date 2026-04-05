#![no_std]

pub mod alloc;
pub mod core;
pub mod process;

pub use process::ExitCode;

#[macro_export]
macro_rules! main_entrypoint {
    ($main:path) => {
        const _: fn() -> $crate::process::ExitCode = $main;

        #[unsafe(no_mangle)]
        #[unsafe(link_section = ".text._start")]
        pub extern "C" fn _start(abi: *const ::kernel_abi::KernelAbi) -> ! {
            $crate::core::sys_init(abi);
            $crate::process::exit($main())
        }
    };
}
