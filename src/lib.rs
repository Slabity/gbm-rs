#[macro_use]
extern crate bitflags;
extern crate errno;

mod ffi;
pub mod error;
use error::Result;

use std::fs::File;
use std::os::unix::io::AsRawFd;
use std::os::raw::c_void;
use std::rc::Rc;
use std::marker::PhantomData;

///
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
    pub fn from_file(file: &'a File) -> Result<Device<'a>> {
        let dev = Device {
            file: file,
            raw: try!(ffi::GbmDevice::new(file.as_raw_fd()))
        };
        Ok(dev)
    }

    pub fn buffer(&'a self, size: (u32, u32), format: Format, flags: BufferFlags) -> Result<Buffer<'a>> {
        let (width, height) = size;
        let buffer = Buffer {
            device: PhantomData,
            raw: try!(ffi::GbmBufferObject::new(&self.raw, width, height, format as u32, flags.bits())),
            surface: None
        };
        Ok(buffer)
    }

    pub fn surface(&'a self, size: (u32, u32), format: Format, flags: BufferFlags) -> Result<Surface<'a>> {
        let (width, height) = size;
        Surface::from_device(self, width, height, format, flags)
    }

    pub unsafe fn raw(&self) -> *mut c_void {
        self.raw.raw as *mut _
    }
}

pub struct Surface<'a> {
    device: PhantomData<Device<'a>>,
    raw: ffi::GbmSurface,
}

impl<'a> Surface<'a> {
    pub fn from_device(device: &'a Device, width: u32, height: u32, format: Format, flags: BufferFlags) -> Result<Surface<'a>> {
        let surface = Surface {
            device: PhantomData,
            raw: try!(ffi::GbmSurface::new(&device.raw, width, height, format as u32, flags.bits()))
        };
        Ok(surface)
    }

    pub unsafe fn lock_front_buffer(&'a self) -> Result<Buffer<'a>> {
        let buffer = Buffer {
            device: PhantomData,
            raw: try!(self.raw.lock_front_buffer()),
            surface: Some(self)
        };
        Ok(buffer)
    }

    pub unsafe fn raw(&self) -> *mut c_void {
        self.raw.raw as *mut _
    }
}

pub struct Buffer<'a> {
    device: PhantomData<Device<'a>>,
    raw: ffi::GbmBufferObject,
    surface: Option<&'a Surface<'a>>
}

impl<'a> Buffer<'a> {
    pub fn size(&self) -> (u32, u32) {
        (self.raw.width(), self.raw.height())
    }

    pub fn stride(&self) -> u32 {
        self.raw.stride()
    }

    pub fn format(&self) -> u32 {
        self.raw.format()
    }

    pub fn handle(&self) -> *mut c_void {
        self.raw.handle()
    }

    pub fn set_user_data<D>(&self, data: Option<Rc<D>>) {
        self.raw.set_user_data(data);
    }

    pub unsafe fn get_user_data<D>(&self) -> Option<Rc<D>> {
        self.raw.get_user_data()
    }

    pub unsafe fn raw(&self) -> *mut c_void {
        self.raw.raw as *mut _
    }
}

impl<'a> Drop for Buffer<'a> {
    fn drop(&mut self) {
        match self.surface {
            Some(surface) => {
                    surface.raw.release_front_buffer(&self.raw);
            },
            None => self.raw.destroy()
        }
    }
}

bitflags! {
    pub flags BufferFlags: u32 {
        const SCANOUT   = ffi::gbm_bo_flags::GBM_BO_USE_SCANOUT as u32,
        const CURSOR    = ffi::gbm_bo_flags::GBM_BO_USE_CURSOR as u32,
        const RENDERING = ffi::gbm_bo_flags::GBM_BO_USE_RENDERING as u32,
        const WRITE     = ffi::gbm_bo_flags::GBM_BO_USE_WRITE as u32,
        const LINEAR    = ffi::gbm_bo_flags::GBM_BO_USE_LINEAR as u32
    }
}

pub enum Format {
    XRGB8888 = ffi::gbm_bo_format::GBM_BO_FORMAT_XRGB8888 as isize,
    ARGB8888 = ffi::gbm_bo_format::GBM_BO_FORMAT_ARGB8888 as isize
}
