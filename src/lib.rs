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

/// A `Device` is a handle to the character device file that provides libgbm
/// access.
///
/// A `Device` does not own the handle to the file used. It is the responsibility
/// of the program to open and close the file.
pub struct Device<F> where F: AsRef<File> {
    file: F,
    raw: ffi::GbmDevice
}

impl<F> AsRef<File> for Device<F> where F: AsRef<File> {
    fn as_ref(&self) -> &File {
        self.file.as_ref()
    }
}

impl<'a, F> Device<F> where F: AsRef<File> {
    /// Creates a `Device` from a file reference.
    pub fn from_file(file: F) -> Result<Device<F>> {
        let fd = file.as_ref().as_raw_fd();
        let dev = Device {
            file: file,
            raw: try!(ffi::GbmDevice::new(fd))
        };
        Ok(dev)
    }

    /// Creates a `Buffer` using the given size and parameters.
    pub fn buffer(&'a self, size: (u32, u32), format: Format, flags: BufferFlags) -> Result<Buffer<F>> {
        let (width, height) = size;
        let buffer = Buffer {
            device: PhantomData,
            raw: try!(ffi::GbmBufferObject::new(&self.raw, width, height, format as u32, flags.bits())),
            surface: None
        };
        Ok(buffer)
    }

    /// Creates a `Surface` using the given size and parameters.
    pub fn surface(&'a self, size: (u32, u32), format: Format, flags: BufferFlags) -> Result<Surface<F>> {
        Surface::from_device(self, size, format, flags)
    }

    /// Returns a pointer to the underlying `gbm_device`
    pub unsafe fn raw(&self) -> *mut c_void {
        self.raw.raw as *mut _
    }
}

/// A `Surface` is a handle to the buffers used for primary rendering.
///
/// A `Surface` cannot outlive the `Device` it was created from.
pub struct Surface<F> where F: AsRef<File> {
    device: PhantomData<Device<F>>,
    raw: ffi::GbmSurface,
}

impl<'a, F> Surface<F> where F: AsRef<File> {
    /// Creates a surface from a `Device` and the given parameters.
    pub fn from_device(device: &'a Device<F>, size: (u32, u32), format: Format, flags: BufferFlags) -> Result<Surface<F>> {
        let (width, height) = size;
        let surface = Surface {
            device: PhantomData,
            raw: try!(ffi::GbmSurface::new(&device.raw, width, height, format as u32, flags.bits()))
        };
        Ok(surface)
    }

    /// Locks the front buffer to be used for display.
    ///
    /// # Safety
    /// This method should be called once, and only once per buffer swap.
    /// Calling, this method before a buffer swap, or multiple times between
    /// swaps will result in undefined behavior. Likely crashes.
    pub unsafe fn lock_front_buffer(&'a self) -> Result<Buffer<'a, F>> {
        let buffer = Buffer {
            device: PhantomData,
            raw: try!(self.raw.lock_front_buffer()),
            surface: Some(self)
        };
        Ok(buffer)
    }

    /// Returns a pointer to the underlying `gbm_surface`
    pub unsafe fn raw(&self) -> *mut c_void {
        self.raw.raw as *mut _
    }
}

pub struct Buffer<'a, F> where F: 'a + AsRef<File> {
    device: PhantomData<Device<F>>,
    raw: ffi::GbmBufferObject,
    surface: Option<&'a Surface<F>>
}

impl<'a, F> Buffer<'a, F> where F: AsRef<File> {
    /// Returns the width and height of the buffer.
    pub fn size(&self) -> (u32, u32) {
        (self.raw.width(), self.raw.height())
    }

    /// Returns the stride of the buffer.
    pub fn stride(&self) -> u32 {
        self.raw.stride()
    }

    /// Returns the format of the buffer.
    pub fn format(&self) -> u32 {
        self.raw.format()
    }

    /// Returns the raw handle to the buffer.
    pub fn handle(&self) -> *mut c_void {
        self.raw.handle()
    }

    /// Attach a reference counted object to the buffer. This can be
    /// retrieved again using `get_user_data`
    ///
    /// When this buffer is destroyed, it will automatically destroy the
    /// reference.
    pub fn set_user_data<D>(&self, data: Option<Rc<D>>) {
        self.raw.set_user_data(data);
    }

    /// Retrieves the reference counted data set using `set_user_data`.
    ///
    /// # Safety
    /// Using the wrong type will result in the data being interpretted
    /// incorrectly.
    pub unsafe fn get_user_data<D>(&self) -> Option<Rc<D>> {
        self.raw.get_user_data()
    }

    /// Returns a pointer to the underlying `gbm_buffer`
    pub unsafe fn raw(&self) -> *mut c_void {
        self.raw.raw as *mut _
    }
}

impl<'a, F> Drop for Buffer<'a, F> where F: AsRef<File> {
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
