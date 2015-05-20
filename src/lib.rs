#![feature(libc)]

extern crate libc;

use libc::{c_void, c_int, c_uint, off_t, size_t};
use libc::consts::os::posix88::{
  O_CREAT, O_RDONLY, O_RDWR,
  MAP_FAILED, MAP_SHARED, PROT_READ, PROT_WRITE,
};
use libc::funcs::posix88::mman::{mmap, munmap, shm_open, shm_unlink};
use libc::funcs::posix88::stat_::{fchmod, fstat};
use libc::funcs::posix88::unistd::{close};
use libc::funcs::posix01::unistd::{ftruncate};
use libc::types::os::arch::posix01::{stat};
use std::ffi::{CString};
use std::mem::{uninitialized};
use std::ptr;

pub struct PosixShmMap {
  data: *mut u8,
  size: usize,
}

impl Drop for PosixShmMap {
  fn drop(&mut self) {
    let ret1 = unsafe { munmap(self.data as *mut c_void, self.size as size_t) };
    assert!(ret1 != -1, "failed to unmap memory!");
  }
}

impl PosixShmMap {
  pub unsafe fn as_ptr<T>(&self) -> *const T {
    self.data as *const T
  }

  pub unsafe fn as_mut_ptr<T>(&self) -> *mut T {
    self.data as *mut T
  }

  pub fn size(&self) -> usize {
    self.size
  }
}

pub struct PosixShm {
  name: String,
  fd: c_int,
  prot: c_int,
}

impl Drop for PosixShm {
  fn drop(&mut self) {
    if self.fd != -1 {
      let ret2 = unsafe { close(self.fd) };
      assert!(ret2 != -1, "failed to close shared memory object!");
    }
  }
}

impl PosixShm {
  pub fn open_mode(name: &str, mode: c_int, perm: c_uint, prot: c_int) -> PosixShm {
    // FIXME: check name format based on man page recommendations.
    let name_cstr = CString::new(name)
      .ok().expect("failed to create C string!");
    let fd = unsafe { shm_open(name_cstr.as_ptr(), mode, perm) };
    assert!(fd != -1, "failed to open shared memory object!");
    PosixShm{
      name: name.to_string(),
      fd: fd,
      prot: prot,
    }
  }

  pub fn open(name: &str) -> PosixShm {
    PosixShm::open_mode(name, O_RDWR, 0o600, PROT_READ | PROT_WRITE)
  }

  pub fn open_read_only(name: &str) -> PosixShm {
    PosixShm::open_mode(name, O_RDONLY, 0o600, PROT_READ)
  }

  pub fn create(name: &str) -> PosixShm {
    PosixShm::open_mode(name, O_RDWR | O_CREAT, 0o600, PROT_READ | PROT_WRITE)
  }

  pub fn create_shared_group(name: &str) -> PosixShm {
    PosixShm::open_mode(name, O_RDWR | O_CREAT, 0o660, PROT_READ | PROT_WRITE)
  }

  pub fn create_shared_everyone(name: &str) -> PosixShm {
    PosixShm::open_mode(name, O_RDWR | O_CREAT, 0o666, PROT_READ | PROT_WRITE)
  }

  pub fn truncate(&mut self, size: isize) {
    let ret = unsafe { ftruncate(self.fd, size as off_t) };
    assert!(ret != -1, "failed to resize shared memory object!");
  }

  pub fn map_all(&self) -> PosixShmMap {
    let mut st: stat = unsafe { uninitialized() };
    let ret = unsafe { fstat(self.fd, &mut st as *mut stat) };
    assert!(ret != -1, "failed to query file stat!");
    let size = st.st_size as usize;
    self.map(size, 0)
  }

  pub fn map(&self, size: usize, offset: isize) -> PosixShmMap {
    let data = unsafe { mmap(ptr::null_mut(), size as size_t, self.prot, MAP_SHARED, self.fd, offset as off_t) };
    assert!(data != MAP_FAILED, "failed to map memory!");
    PosixShmMap{
      data: data as *mut u8,
      size: size,
    }
  }

  pub fn unlink(&self) {
    let name: &str = self.name.as_ref();
    let name_cstr = CString::new(name)
      .ok().expect("failed to create C string!");
    let ret = unsafe { shm_unlink(name_cstr.as_ptr()) };
    assert!(ret != -1, "failed to unlink shared memory object!");
    // XXX(20150520): note that the shm is still usable after unlinking but
    // before closing.
    /*self.fd = -1;
    self.data = ptr::null_mut();
    self.size = 0;*/
  }
}
