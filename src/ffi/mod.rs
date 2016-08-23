mod gbm_shim;

pub use self::gbm_shim::*;
use std::os::unix::io::RawFd;
use std::mem::transmute;
use std::os::raw::c_void;

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

    pub fn lock_front_buffer(&self) -> GbmBufferObject {
        let ptr = unsafe {
            gbm_surface_lock_front_buffer(self.raw)
        };

        GbmBufferObject {
            raw: ptr
        }
    }

    pub fn release_front_buffer(&self, buffer: GbmBufferObject) {
        unsafe {
            gbm_surface_release_buffer(self.raw, buffer.raw);
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

#[derive(Debug)]
pub struct GbmBufferObject {
    pub raw: *mut gbm_bo
}

impl GbmBufferObject {
    pub fn new(device: &GbmDevice, width: u32, height: u32, format: u32, flags: u32) -> GbmBufferObject {
        let ptr = unsafe {
            gbm_bo_create(device.raw, width, height, format, flags)
        };

        GbmBufferObject {
            raw: ptr
        }
    }

    pub fn width(&self) -> u32 {
        unsafe {
            gbm_bo_get_width(self.raw)
        }
    }

    pub fn height(&self) -> u32 {
        unsafe {
            gbm_bo_get_height(self.raw)
        }
    }

    pub fn stride(&self) -> u32 {
        unsafe {
            gbm_bo_get_stride(self.raw)
        }
    }

    pub fn format(&self) -> u32 {
        unsafe {
            gbm_bo_get_format(self.raw)
        }
    }

    pub fn handle(&self) -> u64 {
        unsafe {
            transmute(gbm_bo_get_handle(self.raw))
        }
    }

    pub unsafe fn set_user_data(&self, data: *mut c_void) {
        gbm_bo_set_user_data(self.raw, data, None);
    }
}

impl Drop for GbmBufferObject {
    fn drop(&mut self) {
        unsafe {
            gbm_bo_destroy(self.raw);
        }
    }
}

