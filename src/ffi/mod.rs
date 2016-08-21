mod gbm_shim;

pub use self::gbm_shim::*;
use std::os::unix::io::RawFd;

#[derive(Debug)]
pub struct GbmDevice {
    pub raw: *mut gbm_device
}

impl GbmDevice {
    pub fn new(fd: RawFd) -> GbmDevice {
        let ptr = unsafe {
            gbm_create_device(fd)
        };

        GbmDevice {
            raw: ptr
        }
    }
}

impl Drop for GbmDevice {
    fn drop(&mut self) {
        unsafe {
            gbm_device_destroy(self.raw);
        }
    }
}

#[derive(Debug)]
pub struct GbmSurface {
    pub raw: *mut gbm_surface
}

impl GbmSurface {
    pub fn new(device: &GbmDevice, width: u32, height: u32, format: u32, flags: u32) -> GbmSurface {
        let ptr = unsafe {
            gbm_surface_create(device.raw, width, height, format, flags)
        };

        GbmSurface {
            raw: ptr
        }
    }
}

impl Drop for GbmSurface {
    fn drop(&mut self) {
        unsafe {
            gbm_surface_destroy(self.raw);
        }
    }
}

