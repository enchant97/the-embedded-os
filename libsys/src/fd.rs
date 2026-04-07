use core::ffi::c_void;

use kernel_abi::{ExitCode, FileDescriptor};

use crate::core::abi;

pub struct FileDesc {
    descriptor: FileDescriptor,
}

impl FileDesc {
    #[must_use]
    pub fn from_fd(fd: FileDescriptor) -> Self {
        Self { descriptor: fd }
    }

    pub fn write(&self, buf: &[u8]) {
        (abi().write)(self.descriptor, buf.as_ptr(), buf.len());
    }

    pub fn flush(&self) {
        (abi().flush)(self.descriptor);
    }

    pub fn ioctl(
        &self,
        op: usize,
        in_arg: *const c_void,
        out_arg: *mut c_void,
    ) -> Result<(), ExitCode> {
        let code = (abi().ioctl)(self.descriptor, op, in_arg, out_arg);
        if code == ExitCode::Ok {
            Ok(())
        } else {
            Err(code)
        }
    }
}
