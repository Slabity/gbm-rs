mod ffi;

use std::fs::File;
use std::os::unix::io::AsRawFd;

pub struct Device<'a> {
    file: &'a File,
    raw: ffi::GbmDevice
}

impl<'a> AsRef<File> for Device<'a> {
    fn as_ref(&self) -> &File {
        self.file
    }
}

impl<'a> Device<'a> {
    pub fn from_file(file: &'a File) -> Device<'a> {
        Device {
            file: file,
            raw: ffi::GbmDevice::new(file.as_raw_fd())
        }
    }
}

pub struct Surface<'a> {
    device: &'a Device<'a>,
    raw: ffi::GbmSurface
}

impl<'a> Surface<'a> {
    pub fn from_device(device: &'a Device, width: u32, height: u32, format: u32, flags: u32) -> Surface<'a> {
        Surface {
            device: device,
            raw: ffi::GbmSurface::new(&device.raw, width, height, format, flags)
        }
    }
}
