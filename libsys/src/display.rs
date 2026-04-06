use crate::fd::FileDesc;
use core::{
    cell::RefCell,
    ffi::c_void,
    ptr::{null, null_mut},
};
use embassy_sync::blocking_mutex::{Mutex, raw::ThreadModeRawMutex};
use kernel_abi::display::DisplayOperation;
use kernel_abi::{ExitCode, FileDescriptor};
use nostd::io::Write;

pub use kernel_abi::display::{DisplayMode, DisplayStat};

pub struct Display {
    _private: (),
}

impl Display {
    pub fn get_display_mode(&self) -> Result<DisplayMode, ExitCode> {
        let mut display_mode = DisplayMode::Text;
        FileDesc::from_fd(FileDescriptor::Display)
            .ioctl(
                DisplayOperation::GetMode as usize,
                null(),
                &mut display_mode as *mut _ as *mut c_void,
            )
            .map(|_| display_mode)
    }

    pub fn set_display_mode(&mut self, display_mode: DisplayMode) -> Result<(), ExitCode> {
        FileDesc::from_fd(FileDescriptor::Display).ioctl(
            DisplayOperation::SetMode as usize,
            &display_mode as *const _ as *mut c_void,
            null_mut(),
        )
    }

    pub fn get_display_stat(&self) -> Result<DisplayStat, ExitCode> {
        let mut display_stat = DisplayStat {
            width: 0,
            height: 0,
        };
        FileDesc::from_fd(FileDescriptor::Display)
            .ioctl(
                DisplayOperation::GetStat as usize,
                null(),
                &mut display_stat as *mut _ as *mut c_void,
            )
            .map(|_| display_stat)
    }
}

static DISPLAY: Mutex<ThreadModeRawMutex, RefCell<Display>> =
    Mutex::new(RefCell::new(Display { _private: () }));

#[must_use]
pub fn display() -> &'static Mutex<ThreadModeRawMutex, RefCell<Display>> {
    &DISPLAY
}

impl Write for Display {
    fn write(&mut self, buf: &[u8]) -> nostd::io::Result<usize> {
        FileDesc::from_fd(FileDescriptor::Display).write(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> nostd::io::Result<()> {
        FileDesc::from_fd(FileDescriptor::Display).flush();
        Ok(())
    }
}
