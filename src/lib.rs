mod ffi;

use std::fs::File;
use std::os::unix::io::AsRawFd;
use std::sync::{Mutex, MutexGuard};
use std::os::raw::c_void;


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

    pub fn buffer(&'a self, size: (u32, u32), format: u32, flags: u32) -> Buffer<'a> {
        let (width, height) = size;
        Buffer {
            device: self,
            raw: ffi::GbmBufferObject::new(&self.raw, width, height, format, flags)
        }
    }

    pub fn surface(&'a self, size: (u32, u32), format: u32, flags: u32) -> Surface<'a> {
        let (width, height) = size;
        Surface::from_device(self, width, height, format, flags)
    }
}

pub struct Surface<'a> {
    device: &'a Device<'a>,
    raw: ffi::GbmSurface,
    front_lock: Mutex<()>
}

impl<'a> Surface<'a> {
    pub fn from_device(device: &'a Device, width: u32, height: u32, format: u32, flags: u32) -> Surface<'a> {
        Surface {
            device: device,
            raw: ffi::GbmSurface::new(&device.raw, width, height, format, flags),
            front_lock: Mutex::new(())
        }
    }

    pub fn lock_front_buffer(&'a self) -> FrontBuffer<'a> {
        let guard = self.front_lock.lock();
        FrontBuffer {
            surface: self,
            raw: ffi::GbmFrontBufferObject::lock(&self.raw),
            _guard: guard.unwrap()
        }
    }

}

pub struct Buffer<'a> {
    device: &'a Device<'a>,
    raw: ffi::GbmBufferObject
}

impl<'a> Buffer<'a> {
    pub unsafe fn set_user_data(&self, data: *mut c_void) {
        self.raw.set_user_data(data);
    }
}

pub struct FrontBuffer<'a> {
    surface: &'a Surface<'a>,
    raw: ffi::GbmFrontBufferObject,
    _guard: MutexGuard<'a, ()>
}

impl<'a> FrontBuffer<'a> {
    pub unsafe fn set_user_data(&self, data: *mut c_void) {
        self.raw.set_user_data(data);
    }
}

impl<'a> Drop for FrontBuffer<'a> {
    fn drop(&mut self) {
        self.raw.release(&self.surface.raw);
    }
}
