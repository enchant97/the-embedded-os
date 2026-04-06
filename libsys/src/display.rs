use crate::core::abi;
use core::{
    ffi::c_void,
    ptr::{null, null_mut},
};
use kernel_abi::display::DisplayOperation;
use kernel_abi::{ExitCode, FileDescriptor};

pub use kernel_abi::display::{DisplayMode, DisplayStat};

pub fn get_display_mode() -> Result<DisplayMode, ExitCode> {
    let mut display_mode = DisplayMode::Text;
    let code = (abi().ioctl)(
        FileDescriptor::Display,
        DisplayOperation::GetMode as usize,
        null(),
        &mut display_mode as *mut _ as *mut c_void,
    );
    if code == ExitCode::Ok {
        Ok(display_mode)
    } else {
        Err(code)
    }
}

pub fn set_display_mode(display_mode: DisplayMode) -> Result<(), ExitCode> {
    let code = (abi().ioctl)(
        FileDescriptor::Display,
        DisplayOperation::SetMode as usize,
        &display_mode as *const _ as *mut c_void,
        null_mut(),
    );
    if code == ExitCode::Ok {
        Ok(())
    } else {
        Err(code)
    }
}

pub fn get_display_stat() -> Result<DisplayStat, ExitCode> {
    let mut display_stat = DisplayStat {
        width: 0,
        height: 0,
    };
    let code = (abi().ioctl)(
        FileDescriptor::Display,
        DisplayOperation::GetStat as usize,
        null(),
        &mut display_stat as *mut _ as *mut c_void,
    );
    if code == ExitCode::Ok {
        Ok(display_stat)
    } else {
        Err(code)
    }
}
