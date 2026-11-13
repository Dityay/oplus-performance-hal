// OPlus Performance HAL service entry point (Rust port).
//
// Startup recovered from the shipped binary and its init rc
// (erofs/vendor/etc/init/vendor-oplus-hardware-performance-V1-service.rc):
//
//   service oplus.performance.hal.service-1-0
//       /vendor/bin/hw/vendor-oplus-hardware-performance-V1-service
//       class hal
//       capabilities SYS_PTRACE
//       user root
//       group system readproc
//       interface aidl vendor.oplus.hardware.performance.IPerformance/default
//
// The rc grants SYS_PTRACE + readproc because the diagnostic read path makes
// many small /proc reads per transaction, so the pool gets a few threads.

#![allow(dead_code)]

#[path = "../wire.rs"]
mod wire;
#[path = "../hal.rs"]
mod hal;

use std::ffi::CString;
use std::os::raw::c_char;

use hal::{on_create, on_destroy, on_transact, DESCRIPTOR, LOG_TAG};
use wire::*;

// The default instance is a single literal so the exact fqname is statically
// visible in the binary, as it is in the shipped C++ build.
const DEFAULT_INSTANCE: &str = "vendor.oplus.hardware.performance.IPerformance/default";

fn main() {
    // The diagnostic read path is many small /proc reads per transaction, so
    // give the pool the same four threads the shipped service uses.
    let rc = unsafe { ABinderProcess_setThreadPoolMaxThreadCount(4) };
    if !rc {
        log(LOG_LEVEL_ERROR, LOG_TAG, "setThreadPoolMaxThreadCount(4) failed");
    }
    // SAFETY: no arguments.
    unsafe { ABinderProcess_startThreadPool() };

    let descriptor = CString::new(DESCRIPTOR).unwrap();
    // SAFETY: descriptor is NUL-terminated; the handlers match the typedefs.
    let class = unsafe {
        AIBinder_Class_define(
            descriptor.as_ptr() as *const c_char,
            on_create,
            on_destroy,
            on_transact,
        )
    };
    if class.is_null() {
        log(LOG_LEVEL_ERROR, LOG_TAG, "AIBinder_Class_define failed");
        std::process::exit(1);
    }

    // SAFETY: class was just defined; on_create allocates the marker itself,
    // so no args are needed.
    let binder = unsafe { AIBinder_new(class, std::ptr::null_mut()) };
    if binder.is_null() {
        log(LOG_LEVEL_ERROR, LOG_TAG, "AIBinder_new failed");
        std::process::exit(1);
    }

    let name = match std::env::var("OPLUS_PERF_INSTANCE") {
        Ok(suffix) if !suffix.is_empty() => format!("{}/{}", DESCRIPTOR, suffix),
        _ => DEFAULT_INSTANCE.to_string(),
    };
    let instance = CString::new(name.clone()).unwrap();
    // SAFETY: binder is a live AIBinder we own; instance is NUL-terminated.
    let status = unsafe { AServiceManager_addService(binder, instance.as_ptr() as *const c_char) };
    if status != 0 {
        log(
            LOG_LEVEL_ERROR,
            LOG_TAG,
            &format!("addService({}) -> status {}", name, status),
        );
        std::process::exit(1);
    }
    log(LOG_LEVEL_INFO, LOG_TAG, &format!("registered {}", name));

    // SAFETY: no arguments.
    unsafe { ABinderProcess_joinThreadPool() };
}