use crate::fd::FileDesc;
use core::{
    cell::{RefCell, RefMut},
    ffi::c_void,
    ptr::{null, null_mut},
};
use embassy_sync::blocking_mutex::{Mutex, raw::ThreadModeRawMutex};
use kernel_abi::display::DisplayOperation;
use kernel_abi::{ExitCode, FileDescriptor};
use nostd::io::Write;

pub use kernel_abi::display::{DisplayMode, DisplayStat};

pub struct DisplayRaw {
    _private: (),
}

pub struct Display {
    inner: Mutex<ThreadModeRawMutex, RefCell<DisplayRaw>>,
}

impl Display {
    pub fn lock<U>(&self, f: impl FnOnce(&mut RefMut<'_, DisplayRaw>) -> U) -> U {
        unsafe { self.inner.lock_mut(|v| f(&mut v.borrow_mut())) }
    }
}

impl DisplayRaw {
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

static DISPLAY: Display = Display {
    inner: Mutex::new(RefCell::new(DisplayRaw { _private: () })),
};

#[must_use]
pub fn display() -> &'static Display {
    &DISPLAY
}

impl Write for DisplayRaw {
    fn write(&mut self, buf: &[u8]) -> nostd::io::Result<usize> {
        FileDesc::from_fd(FileDescriptor::Display).write(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> nostd::io::Result<()> {
        FileDesc::from_fd(FileDescriptor::Display).flush();
        Ok(())
    }
}
