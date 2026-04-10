#![no_std]
#![no_main]

use libsys::{ExitCode, nostd::io::Write};

#[unsafe(no_mangle)]
#[unsafe(link_section = ".text._start")]
pub extern "C" fn _start(abi: *const libsys::KernelAbi) -> ExitCode {
    libsys::core::sys_init(abi);
    libsys::display::display().lock(|d| {
        d.set_display_mode(libsys::display::DisplayMode::Text)
            .expect("failed to set mode");
        d.write_all(b"Hello World!")
            .expect("fail to write to display");
        d.flush().expect("fail to flush display buffer");
    });
    ExitCode::Ok
}
