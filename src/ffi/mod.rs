mod gbm_shim;

use errno::{Errno, errno, set_errno};
use super::error::{Result, Error};

pub use self::gbm_shim::*;
use std::os::unix::io::RawFd;
use std::mem::transmute;
use std::os::raw::c_void;
use std::ptr::null_mut;
use std::rc::Rc;

macro_rules! gbm_cmd {
    ( $func:expr ) => ( unsafe {
        set_errno(Errno(0));
        let ptr = $func;
        if ptr.is_null() {
            let err = errno();
            return Err(Error::Ioctl(err));
        }
        ptr
    })
}

#[derive(Debug)]
pub struct GbmDevice {
    pub raw: *mut gbm_device
}

impl GbmDevice {
    pub fn new(fd: RawFd) -> Result<GbmDevice> {
        let ptr = gbm_cmd!(gbm_create_device(fd));
        let dev = GbmDevice {
            raw: ptr
        };

        Ok(dev)
    }
}

impl Drop for GbmDevice {
    fn drop(&mut self) {
        unsafe { gbm_device_destroy(self.raw) };
    }
}

// Must derive clone to access pointer in Drop for Buffer<'a>
#[derive(Debug, Clone)]
pub struct GbmSurface {
    pub raw: *mut gbm_surface
}

impl GbmSurface {
    pub fn new(device: &GbmDevice, width: u32, height: u32, format: u32, flags: u32) -> Result<GbmSurface> {
        let ptr = gbm_cmd!(gbm_surface_create(device.raw, width, height, format, flags));
        let surface = GbmSurface {
            raw: ptr
        };

        Ok(surface)
    }

    pub fn lock_front_buffer(&self) -> Result<GbmBufferObject> {
        let ptr = gbm_cmd!(gbm_surface_lock_front_buffer(self.raw));
        let buffer = GbmBufferObject {
            raw: ptr
        };

        Ok(buffer)
    }

    pub fn release_front_buffer(&self, buffer: &GbmBufferObject) {
        unsafe { gbm_surface_release_buffer(self.raw, buffer.raw) };
    }
}

impl Drop for GbmSurface {
    fn drop(&mut self) {
        unsafe { gbm_surface_destroy(self.raw) };
    }
}

// Must derive clone to access pointer in Drop for Buffer<'a>
#[derive(Debug, Clone)]
pub struct GbmBufferObject {
    pub raw: *mut gbm_bo
}

impl GbmBufferObject {
    pub fn new(device: &GbmDevice, width: u32, height: u32, format: u32, flags: u32) -> Result<GbmBufferObject> {
        let ptr = gbm_cmd!(gbm_bo_create(device.raw, width, height, format, flags));
        let buffer = GbmBufferObject {
            raw: ptr
        };

        Ok(buffer)
    }

    pub fn width(&self) -> u32 {
        unsafe { gbm_bo_get_width(self.raw) }
    }

    pub fn height(&self) -> u32 {
        unsafe { gbm_bo_get_height(self.raw) }
    }

    pub fn stride(&self) -> u32 {
        unsafe { gbm_bo_get_stride(self.raw) }
    }

    pub fn format(&self) -> u32 {
        unsafe { gbm_bo_get_format(self.raw) }
    }

    pub fn handle(&self) -> *mut c_void {
        unsafe { transmute(gbm_bo_get_handle(self.raw)) }
    }

    pub fn set_user_data<T: Sized>(&self, data: Option<Rc<T>>) {
        unsafe {
            // If we have user data already, destroy it.
            let fields: *mut _gbm_bo = self.raw as *mut _;
            if !(*fields).user_data.is_null() {
                let func = (*fields).destroy_user_data as unsafe extern "C" fn(*mut gbm_bo, *mut c_void);
                func(self.raw, (*fields).user_data);
            }
        }

        match data {
            Some(d) => {
                let ptr = Box::into_raw(Box::new(d)) as *mut _;
                unsafe {
                    gbm_bo_set_user_data(self.raw, ptr, Some(destroy::<T>));
                }
            },
            None => unsafe {
                gbm_bo_set_user_data(self.raw, null_mut(), None);
            }
        }
    }

    pub unsafe fn get_user_data<T: Sized>(&self) -> Option<Rc<T>> {
        let ptr = gbm_bo_get_user_data(self.raw) as *mut _;
        if ptr.is_null() {
            return None;
        }
        let rc: Rc<T> = *Box::from_raw(ptr);
        Some(rc.clone())
    }

    pub fn destroy(&self) {
        unsafe { gbm_bo_destroy(self.raw) };
    }
}

/// Used to remove the reference count from user data.
unsafe extern fn destroy<T: Sized>(_: *mut gbm_bo, data: *mut c_void) {
    let _: Box<Rc<T>> = Box::from_raw(data as *mut _);
}

/// Needed to access user data and manually call destroy
#[repr(C)]
struct _gbm_bo {
    device: *mut gbm_device,
    width: u32,
    height: u32,
    stride: u32,
    format: u32,
    handle: *mut c_void,
    user_data: *mut c_void,
    destroy_user_data: unsafe extern "C" fn(*mut gbm_bo, *mut c_void)
}
