use core::ffi::{CStr, c_char, c_int};
use kernel::prelude::*;

module! {
    type: RustExecDriver,
    author: "Sagartaunk",
    description: "Prototype backend for Jokey-Framework",
}

const EXEC_PATH: &CStr = c"/bin/echo";
const EXEC_ARC0: &CStr = c"/bin/echo";
const EXEC_ARG1: &CStr = c"The driver seems to be working as of now";

/// The `Kernel` crate lacks a safe wrapper for system calls, Thus
/// a legacy api has been used directly. The signature is copied from
/// `include/linux/umh.h`.
extern "C" {
    fn call_usermodehelper(
        path: *const c_char,
        argv: *mut *mut c_char,
        envp: *mut *mut c_char,
        wait: c_int,
    ) -> c_int;
}

#[allow(dead_code)]
// fire and forget
const UMH_NO_WAIT: c_int = 0;
#[allow(dead_code)]
// wait for the exec() itself, not the process
const UMH_WAIT_EXEC: c_int = 1;
// wait for the process to finish (used below)
const UMH_WAIT_PROC: c_int = 2;
#[allow(dead_code)]
// OR into the wait mode to make it killable
const UMH_KILLABLE: c_int = 4;

struct RustExecDriver;

impl kernel::Module for RustExecDriver {
    fn init(_module: &'static ThisModule) -> Result<Self> {
        pr_info!("rust_exec_driver: loading\n");

        // argv / envp must be NULL-terminated `char **` arrays.
        let mut argv: [*mut c_char; 3] = [
            EXEC_ARG0.as_ptr() as *mut c_char,
            EXEC_ARG1.as_ptr() as *mut c_char,
            core::ptr::null_mut(),
        ];
        let mut envp: [*mut c_char; 2] = [
            c"PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin".as_ptr()
                as *mut c_char,
            core::ptr::null_mut(),
        ];

        // SAFETY:
        // - EXEC_PATH and every string reachable from argv/envp are
        //   `'static`, NUL-terminated C strings, so the pointers stay valid
        //   for the whole call.
        // - argv/envp are properly NULL-terminated `*mut *mut c_char`
        //   arrays, matching the C signature.
        // - UMH_WAIT_PROC blocks until the child exits, so nothing above is
        //   dropped or reused while the kernel still holds a reference.
        // - call_usermodehelper() only reads these strings; the non-const
        //   `char **` signature is a C-ism, not a sign that it mutates them.
        let ret = unsafe {
            call_usermodehelper(
                EXEC_PATH.as_ptr(),
                argv.as_mut_ptr(),
                envp.as_mut_ptr(),
                UMH_WAIT_PROC,
            )
        };

        if ret != 0 {
            pr_err!("rust_exec_driver: call_usermodehelper failed: {}\n", ret);
        } else {
            pr_info!("rust_exec_driver: user program launched\n");
        }

        Ok(RustExecDriver)
    }
}

impl Drop for RustExecDriver {
    fn drop(&mut self) {
        pr_info!("rust_exec_driver: unloading\n");
    }
}
