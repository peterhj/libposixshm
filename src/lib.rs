#![feature(libc)]

extern crate libc;

use libc::{c_void, c_int, c_uint, size_t};
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
use std::path::{PathBuf};
use std::ptr;

pub struct PosixShm {
  path: PathBuf,
  fd: c_int,
  data: *mut u8,
  size: usize,
}

impl Drop for PosixShm {
  fn drop(&mut self) {
    if self.fd != -1 {
      let ret1 = unsafe { munmap(self.data as *mut c_void, self.size as size_t) };
      let ret2 = unsafe { close(self.fd) };
      assert!(ret1 != -1, "failed to unmap memory!");
      assert!(ret2 != -1, "failed to close shared memory object!");
    }
  }
}

impl PosixShm {
  pub fn open_mode(path: &PathBuf, mode: c_int, perm: c_uint, prot: c_int) -> PosixShm {
    let path_cstr = CString::new(path.to_str()
        .expect("path is not a valid string!")
        .as_bytes())
      .ok().expect("failed to create C string!");
    let fd = unsafe { shm_open(path_cstr.as_ptr(), mode, perm) };
    assert!(fd != -1, "failed to open shared memory object!");
    let mut st: stat = unsafe { uninitialized() };
    let ret = unsafe { fstat(fd, &mut st as *mut stat) };
    assert!(ret != -1, "failed to query file stat!");
    let size = st.st_size as usize;
    let data = unsafe { mmap(ptr::null_mut(), size as size_t, prot, MAP_SHARED, fd, 0) };
    assert!(data != MAP_FAILED, "failed to map memory!");
    PosixShm{
      path: path.clone(),
      fd: fd,
      data: data as *mut u8,
      size: size,
    }
  }

  pub fn open(path: &PathBuf) -> PosixShm {
    PosixShm::open_mode(path, O_RDWR, 0o600, PROT_READ | PROT_WRITE)
  }

  pub fn open_read_only(path: &PathBuf) -> PosixShm {
    PosixShm::open_mode(path, O_RDONLY, 0o600, PROT_READ)
  }

  pub fn create(path: &PathBuf) -> PosixShm {
    PosixShm::open_mode(path, O_RDWR | O_CREAT, 0o600, PROT_READ | PROT_WRITE)
  }

  pub fn create_shared_group(path: &PathBuf) -> PosixShm {
    PosixShm::open_mode(path, O_RDWR | O_CREAT, 0o660, PROT_READ | PROT_WRITE)
  }

  pub fn create_shared_everyone(path: &PathBuf) -> PosixShm {
    PosixShm::open_mode(path, O_RDWR | O_CREAT, 0o666, PROT_READ | PROT_WRITE)
  }

  pub fn unlink(&self) {
    let path_cstr = CString::new(self.path.to_str()
        .expect("path is not a valid string!")
        .as_bytes())
      .ok().expect("failed to create C string!");
    let ret = unsafe { shm_unlink(path_cstr.as_ptr()) };
    assert!(ret != -1, "failed to unlink shared memory object!");
    // XXX(20150520): note that the shm is still usable after unlinking but
    // before closing.
    /*self.fd = -1;
    self.data = ptr::null_mut();
    self.size = 0;*/
  }

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
