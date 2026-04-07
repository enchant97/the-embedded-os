#![no_std]
#![no_main]

use libsys::ExitCode;

#[unsafe(no_mangle)]
#[unsafe(link_section = ".text._start")]
pub extern "C" fn _start(abi: *const libsys::KernelAbi) -> ExitCode {
    libsys::core::sys_init(abi);
    libsys::display::display().lock(|d| {
        d.set_display_mode(libsys::display::DisplayMode::Text);
    });
    ExitCode::Ok
}
