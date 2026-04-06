#![no_std]
#![no_main]

use libsys::ExitCode;

#[unsafe(no_mangle)]
#[unsafe(link_section = ".text._start")]
pub extern "C" fn _start(abi: *const libsys::KernelAbi) -> ExitCode {
    libsys::core::sys_init(abi);
    // write better libsys to make it easier
    unsafe {
        libsys::display::display().lock_mut(|d| {
            d.borrow_mut()
                .set_display_mode(libsys::display::DisplayMode::Text);
        });
    }
    ExitCode::Ok
}
