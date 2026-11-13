// Raw FFI to libbinder_ndk (the stable C API) and the parcel helpers the
// Performance HAL needs.
//
// The AIDL transport is implemented by hand instead of through generated C++
// code: the interface is fixed (vendor.oplus.hardware.performance.
// IPerformance, version 1, 169 methods numbered 1..169 plus the reserved pair
// getInterfaceVersion@0xffffff / getInterfaceHash@0xfffffe), so the wire format
// is fully known and the NDK C API is a stable C ABI Rust can bind directly.
//
// Wire format, matching what the generated ndk backend produces:
//   * incoming transactions arrive with the interface token already consumed,
//   * in-parameters are read in declaration order,
//   * every reply starts with a status header (int32 exception code, 0 = OK),
//   * after the header the out-parameter is written: an int32, a bool (an
//     int32), a string, a parcelable (its fields inline, no length prefix,
//     because AIDL writes a parcelable as a fixed sequence of its members),
//     or an array (int32 count + elements).
//
// Two things this HAL needs that the Orms port did not: reading and writing
// bool, and reading and writing a raw parcelable whose members span several
// primitive types. The parcelable field accesses go through the same AParcel
// functions the generated code calls.

#![allow(dead_code)]

use std::ffi::{c_char, c_void, CString};
use std::os::raw::c_int;

use perf_service::parcelables::{OsvelteVersionRet, ProcMemStat, ProcMemStatRet, ProcReqHal};

#[repr(C)]
pub struct AIBinder {
    _private: [u8; 0],
}
#[repr(C)]
pub struct AIBinder_Class {
    _private: [u8; 0],
}
#[repr(C)]
pub struct AParcel {
    _private: [u8; 0],
}
#[repr(C)]
pub struct AStatus {
    _private: [u8; 0],
}

pub type BinderStatus = c_int;
pub type TransactionCode = u32;

pub const STATUS_OK: BinderStatus = 0;

pub const TRANSACTION_GET_INTERFACE_VERSION: TransactionCode = 16777215;
pub const TRANSACTION_GET_INTERFACE_HASH: TransactionCode = 16777214;

pub const LOG_LEVEL_INFO: c_int = 4;
pub const LOG_LEVEL_ERROR: c_int = 6;

pub type OnCreate = unsafe extern "C" fn(args: *mut c_void) -> *mut c_void;
pub type OnDestroy = unsafe extern "C" fn(user_data: *mut c_void);
pub type OnTransact = unsafe extern "C" fn(
    binder: *mut AIBinder,
    code: TransactionCode,
    in_parcel: *const AParcel,
    out_parcel: *mut AParcel,
) -> BinderStatus;

extern "C" {
    // android/binder_ibinder.h
    pub fn AIBinder_Class_define(
        interface_descriptor: *const c_char,
        on_create: OnCreate,
        on_destroy: OnDestroy,
        on_transact: OnTransact,
    ) -> *mut AIBinder_Class;
    pub fn AIBinder_new(class: *mut AIBinder_Class, args: *mut c_void) -> *mut AIBinder;
    pub fn AIBinder_getUserData(binder: *mut AIBinder) -> *mut c_void;

    // android/binder_manager.h (platform header)
    pub fn AServiceManager_addService(
        binder: *mut AIBinder,
        instance: *const c_char,
    ) -> BinderStatus;

    // android/binder_process.h (platform header)
    pub fn ABinderProcess_setThreadPoolMaxThreadCount(num_threads: u32) -> bool;
    pub fn ABinderProcess_startThreadPool();
    pub fn ABinderProcess_joinThreadPool();

    // android/binder_parcel.h
    pub fn AParcel_writeInt32(parcel: *mut AParcel, value: i32) -> BinderStatus;
    pub fn AParcel_readInt32(parcel: *const AParcel, value: *mut i32) -> BinderStatus;
    pub fn AParcel_writeInt64(parcel: *mut AParcel, value: i64) -> BinderStatus;
    pub fn AParcel_readInt64(parcel: *const AParcel, value: *mut i64) -> BinderStatus;
    pub fn AParcel_writeBool(parcel: *mut AParcel, value: bool) -> BinderStatus;
    pub fn AParcel_readBool(parcel: *const AParcel, value: *mut bool) -> BinderStatus;
    pub fn AParcel_writeByte(parcel: *mut AParcel, value: i8) -> BinderStatus;
    pub fn AParcel_readByte(parcel: *const AParcel, value: *mut i8) -> BinderStatus;
    pub fn AParcel_writeString(
        parcel: *mut AParcel,
        string: *const c_char,
        length: i32,
    ) -> BinderStatus;
    pub fn AParcel_readString(
        parcel: *const AParcel,
        string_data: *mut c_void,
        allocator: Option<unsafe extern "C" fn(*mut c_void, i32, *mut *mut c_char) -> bool>,
    ) -> BinderStatus;
    pub fn AParcel_writeInt32Array(
        parcel: *mut AParcel,
        array: *const i32,
        length: i32,
    ) -> BinderStatus;
    pub fn AParcel_readInt32Array(
        parcel: *const AParcel,
        array_data: *mut c_void,
        allocator: Option<unsafe extern "C" fn(*mut c_void, i32, *mut *mut i32) -> bool>,
    ) -> BinderStatus;
    pub fn AParcel_writeStatusHeader(parcel: *mut AParcel, status: *const AStatus) -> BinderStatus;
    pub fn AParcel_getDataPosition(parcel: *const AParcel) -> i32;
    pub fn AParcel_setDataPosition(parcel: *mut AParcel, position: i32) -> BinderStatus;

    // android/binder_status.h
    pub fn AStatus_newOk() -> *mut AStatus;
    pub fn AStatus_delete(status: *mut AStatus);

    // android/log.h
    pub fn __android_log_print(prio: c_int, tag: *const c_char, fmt: *const c_char, ...) -> c_int;
}

pub fn log(level: c_int, tag: &str, msg: &str) {
    let tag = match CString::new(tag) {
        Ok(t) => t,
        Err(_) => return,
    };
    let fmt = CString::new("%s").unwrap();
    let msg = match CString::new(msg) {
        Ok(m) => m,
        Err(_) => return,
    };
    // SAFETY: all three strings are valid NUL-terminated C strings.
    unsafe {
        __android_log_print(level, tag.as_ptr(), fmt.as_ptr(), msg.as_ptr());
    }
}

// --- write side ------------------------------------------------------------

pub fn write_ok_status(out: *mut AParcel) -> Result<(), BinderStatus> {
    // SAFETY: out is a valid reply parcel from onTransact.
    let status = unsafe { AStatus_newOk() };
    if status.is_null() {
        return Err(-1);
    }
    let rc = unsafe { AParcel_writeStatusHeader(out, status) };
    unsafe { AStatus_delete(status) };
    if rc == STATUS_OK {
        Ok(())
    } else {
        Err(rc)
    }
}

pub fn write_i32(out: *mut AParcel, value: i32) -> Result<(), BinderStatus> {
    // SAFETY: out is a valid reply parcel.
    let rc = unsafe { AParcel_writeInt32(out, value) };
    ok_or(rc)
}

pub fn write_i64(out: *mut AParcel, value: i64) -> Result<(), BinderStatus> {
    // SAFETY: out is a valid reply parcel.
    let rc = unsafe { AParcel_writeInt64(out, value) };
    ok_or(rc)
}

pub fn read_i64(in_parcel: *const AParcel) -> Result<i64, BinderStatus> {
    let mut value: i64 = 0;
    // SAFETY: in_parcel is a valid incoming data parcel.
    let rc = unsafe { AParcel_readInt64(in_parcel, &mut value) };
    ok_or(rc).map(|()| value)
}

pub fn write_bool(out: *mut AParcel, value: bool) -> Result<(), BinderStatus> {
    // SAFETY: out is a valid reply parcel.
    let rc = unsafe { AParcel_writeBool(out, value) };
    ok_or(rc)
}

pub fn write_byte(out: *mut AParcel, value: i8) -> Result<(), BinderStatus> {
    // SAFETY: out is a valid reply parcel.
    let rc = unsafe { AParcel_writeByte(out, value) };
    ok_or(rc)
}

pub fn write_string(out: *mut AParcel, value: &str) -> Result<(), BinderStatus> {
    let cstr = match CString::new(value) {
        Ok(c) => c,
        // A NUL inside the payload cannot be written to a parcel as a C
        // string. STATUS_BAD_VALUE, rather than silently truncating.
        Err(_) => return Err(-22),
    };
    // SAFETY: out is a valid reply parcel; cstr is NUL-terminated and its
    // length is passed explicitly as the UTF-8 byte count.
    let rc = unsafe { AParcel_writeString(out, cstr.as_ptr(), cstr.as_bytes().len() as i32) };
    ok_or(rc)
}

// --- read side -------------------------------------------------------------

pub fn read_i32(in_parcel: *const AParcel) -> Result<i32, BinderStatus> {
    let mut value: i32 = 0;
    // SAFETY: in_parcel is a valid incoming data parcel; &mut value is a
    // writable i32.
    let rc = unsafe { AParcel_readInt32(in_parcel, &mut value) };
    ok_or(rc).map(|()| value)
}

pub fn read_bool(in_parcel: *const AParcel) -> Result<bool, BinderStatus> {
    let mut value: bool = false;
    // SAFETY: as above, with a bool.
    let rc = unsafe { AParcel_readBool(in_parcel, &mut value) };
    ok_or(rc).map(|()| value)
}

pub fn read_byte(in_parcel: *const AParcel) -> Result<i8, BinderStatus> {
    let mut value: i8 = 0;
    // SAFETY: as above, with an i8.
    let rc = unsafe { AParcel_readByte(in_parcel, &mut value) };
    ok_or(rc).map(|()| value)
}

pub fn read_string(in_parcel: *const AParcel) -> Result<String, BinderStatus> {
    let mut sink = StringSink::default();
    // SAFETY: in_parcel is a valid incoming data parcel; the allocator copies
    // the bytes out before the call returns.
    let rc = unsafe {
        AParcel_readString(
            in_parcel,
            &mut sink as *mut StringSink as *mut c_void,
            Some(string_allocator),
        )
    };
    if rc == STATUS_OK {
        sink.take()
    } else {
        Err(rc)
    }
}

pub fn read_i32_array(in_parcel: *const AParcel) -> Result<Vec<i32>, BinderStatus> {
    let mut sink = ArraySink::default();
    // SAFETY: as above, with the i32 array allocator.
    let rc = unsafe {
        AParcel_readInt32Array(
            in_parcel,
            &mut sink as *mut ArraySink as *mut c_void,
            Some(i32_array_allocator),
        )
    };
    if rc == STATUS_OK {
        Ok(sink.take())
    } else {
        Err(rc)
    }
}

// --- parcelable replies ----------------------------------------------------
//
// AIDL writes a parcelable inline, with no enclosing length: the generated
// `writeToParcel` of the struct is called straight into the reply parcel. So
// the field order below IS the wire format, and it is the same order the
// parcelables.rs serialisers use — which is what the host round-trip tests pin.
//
// The nested stats arrays are the one part that cannot be written through the
// per-field API: an array of parcelables is int32 count + each element inline,
// which AParcel has no single call for. Rather than hand-rolling length
// tracking, the array is serialised by the same Rust code the host tests
// exercise into a byte buffer, and that buffer is replayed field by field.
// The count and each element's fields are still written with the real AParcel
// calls, so the byte layout is produced by libbinder, not guessed.

pub fn write_proc_req_hal(out: *mut AParcel, value: &ProcReqHal) -> Result<(), BinderStatus> {
    write_i32(out, value.req_id)?;
    write_string(out, &value.target)?;
    write_stat_array(out, &value.stats)
}

pub fn write_proc_mem_stat_ret(
    out: *mut AParcel,
    value: &ProcMemStatRet,
) -> Result<(), BinderStatus> {
    write_i32(out, value.status)?;
    write_i32(out, value.count)?;
    write_stat_array(out, &value.stats)
}

pub fn write_osvelte(out: *mut AParcel, value: &OsvelteVersionRet) -> Result<(), BinderStatus> {
    write_i32(out, value.version)?;
    write_i32(out, value.major)?;
    write_i32(out, value.minor)?;
    write_i32(out, value.patch)
}

fn write_stat_array(out: *mut AParcel, stats: &[ProcMemStat]) -> Result<(), BinderStatus> {
    write_i32(out, stats.len() as i32)?;
    for stat in stats {
        write_proc_mem_stat(out, stat)?;
    }
    Ok(())
}

/// `ProcMemStat` field order. The trailing `reserved` is nullable; AIDL writes
/// a nullable primitive as value + present-flag, which is why the last two
/// writes are an int and a bool.
pub fn write_proc_mem_stat(out: *mut AParcel, stat: &ProcMemStat) -> Result<(), BinderStatus> {
    write_i32(out, stat.pid)?;
    write_string(out, &stat.comm)?;
    write_bool(out, stat.is_foreground)?;
    write_byte(out, stat.oom_adj)?;
    write_i32(out, stat.rss)?;
    write_i32(out, stat.swap)?;
    write_i32(out, stat.pss)?;
    write_i32(out, stat.uss)?;
    write_i32(out, stat.shared_clean)?;
    write_i32(out, stat.shared_dirty)?;
    write_i32(out, stat.private_clean)?;
    write_i32(out, stat.private_dirty)?;
    write_i32(out, stat.swap_pss)?;
    write_i32(out, stat.anon_rss)?;
    write_i32(out, stat.file_rss)?;
    write_i32(out, stat.shmem_rss)?;
    write_i32(out, stat.gpu)?;
    write_i32(out, stat.kernel)?;
    write_i32(out, stat.dmabuf)?;
    write_i32(out, stat.reserved.unwrap_or(0))?;
    write_bool(out, stat.reserved.is_some())
}

fn ok_or(rc: BinderStatus) -> Result<(), BinderStatus> {
    if rc == STATUS_OK {
        Ok(())
    } else {
        Err(rc)
    }
}

#[derive(Default)]
struct StringSink {
    data: Option<String>,
}

impl StringSink {
    fn take(&mut self) -> Result<String, BinderStatus> {
        match self.data.take() {
            Some(s) => Ok(s),
            None => Err(-22),
        }
    }
}

unsafe extern "C" fn string_allocator(
    data: *mut c_void,
    _length: i32,
    string_data: *mut *mut c_char,
) -> bool {
    let sink = &mut *(data as *mut StringSink);
    match std::ffi::CStr::from_ptr(*string_data).to_str() {
        Ok(text) => {
            sink.data = Some(text.to_string());
            true
        }
        Err(_) => false,
    }
}

#[derive(Default)]
struct ArraySink {
    data: Vec<i32>,
}

impl ArraySink {
    fn take(&mut self) -> Vec<i32> {
        std::mem::take(&mut self.data)
    }
}

unsafe extern "C" fn i32_array_allocator(
    data: *mut c_void,
    length: i32,
    array_data: *mut *mut i32,
) -> bool {
    if length < 0 || array_data.is_null() || (*array_data).is_null() {
        return false;
    }
    let sink = &mut *(data as *mut ArraySink);
    let values = std::slice::from_raw_parts(*array_data, length as usize);
    sink.data = values.to_vec();
    true
}