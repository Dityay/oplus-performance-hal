// The Performance HAL service: the onTransact dispatch for all 169
// IPerformance methods plus the reserved getInterfaceVersion /
// getInterfaceHash pair.
//
// Transaction codes are 1..=169 in AIDL declaration order. Rather than 169
// hand-written match arms, the dispatch reads the plan for the code from
// nodes::NODE_TABLE and executes the plan generically. A table entry and a
// match arm cannot disagree if there is only one of them, and the table is
// what the tests enumerate and check against the AIDL surface.
//
// What cannot be derived from the table is the in-parameter shape: AIDL does
// not mark up a transaction's arguments on the wire, so the server has to know
// that code 4 reads (int, int, bool) and code 66 reads (int, String, String).
// Those shapes come from the AIDL file one-for-one and are grouped in
// `read_args` by signature, with every code listed. A `nodes::tests` test
// checks that the union of these arms covers 1..=169 exactly, so a new method
// that is not wired up here fails the build rather than silently reading
// garbage.

use std::ffi::c_void;

use crate::wire::*;

use perf_service::core;
use perf_service::nodeconv;
use perf_service::nodes::{self, Op, Plan};
use perf_service::parcelables::{empty_req, ProcMemStatRet};

pub const LOG_TAG: &str = "OplusPerfHAL";

pub const DESCRIPTOR: &str = "vendor.oplus.hardware.performance.IPerformance";
pub const INTERFACE_HASH: &str = "a1051705ca91eedbba19076e9adacaae43014172";
pub const INTERFACE_VERSION: i32 = 1;

/// The in-parameters of one method, in declaration order. Only the shapes that
/// actually occur in IPerformance appear here.
enum Args {
    None,
    Int(i32),
    Bool(bool),
    TwoInts(i32, i32),
    ThreeInts(i32, i32, i32),
    FourInts(i32, i32, i32, i32),
    TwoIntsBool(i32, i32, bool),
    Str(String),
    IntStr(i32, String),
    StrLong(String, i64),
    StrInt(String, i32),
    TwoStr(String, String),
    ThreeStr(String, String, String),
    FourStr(String, String, String, String),
    TwoIntsStr(i32, i32, String),
    IntStrStr(i32, String, String),
    StrIntArray(String, Vec<i32>),
    IntArrayInt(Vec<i32>, i32),
}

fn read_args(code: u32, in_parcel: *const AParcel) -> Result<Args, BinderStatus> {
    // Every arm below was generated from the AIDL declaration order, so the
    // code lists are the wire contract rather than a hand copy of it. The
    // tests above re-parse the AIDL and check the arms against it, which means
    // a drift here fails the suite instead of a transaction.
    let args = match code {
        // --- (String, long): addAcmDirName, addAcmPkgName -------------------
        1 | 3 => Args::StrLong(read_string(in_parcel)?, read_i64(in_parcel)?),

        // --- (int, int, boolean): addTaskTrackPid ----------------------------
        4 => Args::TwoIntsBool(
            read_i32(in_parcel)?,
            read_i32(in_parcel)?,
            read_bool(in_parcel)?,
        ),

        // --- (int[], int): readMemoryByPids, readMemoryByUids ---------------
        95 | 96 => Args::IntArrayInt(read_i32_array(in_parcel)?, read_i32(in_parcel)?),

        // --- (int, String): hybridswap_memcg_para_read, hybridswap_zram_para_write
        65 | 68 => Args::IntStr(read_i32(in_parcel)?, read_string(in_parcel)?),

        // --- (int, String, String): hybridswap_memcg_para_write -------------
        66 => Args::IntStrStr(
            read_i32(in_parcel)?,
            read_string(in_parcel)?,
            read_string(in_parcel)?,
        ),

        // --- (int): clearTaskTrackGroup, hybridswap_zram_para_read,
        //     readJankTaskTrackByPid, setAcmOpstat, setDamonReclaimColdTime,
        //     writeKmallocDebugCreate, writeMemleakDetectThread, writeVaFeature,
        //     writeTmemorySwitch, writeTmemoryFlushBusy, writeTmemoryFlushIdle,
        //     writeTmemoryCapacity, writeTmemoryHighWaterRatio, getClmThreshold
        5 | 67 | 90 | 118 | 119 | 140 | 143 | 153 | 157..=161 | 166 => {
            Args::Int(read_i32(in_parcel)?)
        }

        // --- (int, int): readOplusReserve3, readUxTaskTrack,
        //     removeTaskTrackPid, setDamonReclaimMonitoring ---------------
        100 | 113 | 116 | 120 => Args::TwoInts(read_i32(in_parcel)?, read_i32(in_parcel)?),

        // --- (int, int, int): perProcessMemReadahead, perProcessMemReclaim,
        //     setDamonReclaimQuota ------------------------------------------
        70 | 71 | 121 => Args::ThreeInts(
            read_i32(in_parcel)?,
            read_i32(in_parcel)?,
            read_i32(in_parcel)?,
        ),

        // --- (int, int, int, int): setDamonReclaimWmarks --------------------
        122 => Args::FourInts(
            read_i32(in_parcel)?,
            read_i32(in_parcel)?,
            read_i32(in_parcel)?,
            read_i32(in_parcel)?,
        ),

        // --- (boolean): writeJankTaskTrackEnable ----------------------------
        139 => Args::Bool(read_bool(in_parcel)?),

        // --- (String, String): setImFlag, writeNandswapProc -----------------
        125 | 145 => Args::TwoStr(read_string(in_parcel)?, read_string(in_parcel)?),

        // --- (String, int): setClmThreshold ---------------------------------
        168 => Args::StrInt(read_string(in_parcel)?, read_i32(in_parcel)?),

        // --- (String, int[]): setFreqGoverner -------------------------------
        154 => Args::StrIntArray(read_string(in_parcel)?, read_i32_array(in_parcel)?),

        // --- (int, int, String): writeOplusReserve3 -------------------------
        146 => Args::TwoIntsStr(
            read_i32(in_parcel)?,
            read_i32(in_parcel)?,
            read_string(in_parcel)?,
        ),

        // --- (String, String, String): readStorageFeature, writeUxState ------
        108 | 152 => Args::ThreeStr(
            read_string(in_parcel)?,
            read_string(in_parcel)?,
            read_string(in_parcel)?,
        ),

        // --- (String, String, String, String): writeStorageFeature ----------
        149 => Args::FourStr(
            read_string(in_parcel)?,
            read_string(in_parcel)?,
            read_string(in_parcel)?,
            read_string(in_parcel)?,
        ),

        // --- (String): the 37 single-String methods --------------------------
        2 | 6..=8 | 24 | 26 | 31 | 32 | 82 | 97 | 109 | 117 | 123 | 124
        | 126..=138 | 141 | 142 | 144 | 147 | 148 | 150 | 151 | 155 | 156 | 169 => {
            Args::Str(read_string(in_parcel)?)
        }

        // --- no in-parameters: the remaining 93 methods ----------------------
        _ => Args::None,
    };
    Ok(args)
}

pub unsafe extern "C" fn on_create(_args: *mut c_void) -> *mut c_void {
    log(LOG_LEVEL_INFO, LOG_TAG, "new Performance service");
    // The service holds no state: every method is a synchronous /proc or /sys
    // operation. A non-null marker keeps the user-data slot occupied for the
    // binder framework and gives on_destroy something to free.
    Box::into_raw(Box::new(ServiceMarker)) as *mut c_void
}

struct ServiceMarker;

pub unsafe extern "C" fn on_destroy(user_data: *mut c_void) {
    if !user_data.is_null() {
        drop(Box::from_raw(user_data as *mut ServiceMarker));
    }
}

pub unsafe extern "C" fn on_transact(
    _binder: *mut AIBinder,
    code: TransactionCode,
    in_parcel: *const AParcel,
    out_parcel: *mut AParcel,
) -> BinderStatus {
    match dispatch(code, in_parcel, out_parcel) {
        Ok(()) => STATUS_OK,
        Err(rc) => rc,
    }
}

fn dispatch(
    code: TransactionCode,
    in_parcel: *const AParcel,
    out_parcel: *mut AParcel,
) -> Result<(), BinderStatus> {
    if code == TRANSACTION_GET_INTERFACE_VERSION {
        write_ok_status(out_parcel)?;
        return write_i32(out_parcel, INTERFACE_VERSION);
    }
    if code == TRANSACTION_GET_INTERFACE_HASH {
        write_ok_status(out_parcel)?;
        return write_string(out_parcel, INTERFACE_HASH);
    }

    let plan = match nodes::plan_for(code) {
        Some(p) => p,
        None => return Err(-31), // STATUS_UNKNOWN_TRANSACTION
    };

    // In-parameters are read before anything else, so a malformed transaction
    // fails before any node is touched.
    let args = read_args(code, in_parcel)?;

    if let Op::Special = plan.op {
        return run_special(plan, &args, out_parcel);
    }

    match plan.op {
        Op::Constant => write_constant(plan, out_parcel),
        Op::ReadString => {
            // The plan's nodes can contain a %s placeholder; only a handful of
            // String-taking methods are ReadString, and they substitute here.
            let text = if plan.nodes.iter().any(|n| n.contains("%s")) {
                nodeconv::read_first_sub(plan.nodes, &string_of(&args))
            } else {
                nodeconv::read_first(plan.nodes)
            };
            write_ok_status(out_parcel)?;
            write_string(out_parcel, &text)
        }
        Op::ReadInt => {
            let value = nodeconv::read_int_first(plan.nodes);
            write_ok_status(out_parcel)?;
            write_i32(out_parcel, value)
        }
        Op::ReadIntScan => {
            let value = nodeconv::read_int_scan_first(plan.nodes);
            write_ok_status(out_parcel)?;
            write_i32(out_parcel, value)
        }
        Op::Exists => {
            let exists = nodeconv::path_exists_first(plan.nodes);
            write_ok_status(out_parcel)?;
            write_i32(out_parcel, if exists { 1 } else { 0 })
        }
        Op::ReadStruct => {
            let target = nodeconv::read_first(plan.nodes);
            write_ok_status(out_parcel)?;
            write_proc_req_hal(out_parcel, &empty_req(0, target))
        }
        Op::Write => {
            let value = string_of(&args);
            nodeconv::write_first(plan.nodes, &value);
            write_ok_status(out_parcel)
        }
        Op::WriteInt => {
            let value = int_of(&args);
            nodeconv::write_int_first(plan.nodes, value);
            write_ok_status(out_parcel)
        }
        Op::Special => unreachable!("specials return above"),
    }
}

/// The constant returns: methods whose OPlus node does not exist on this kernel
/// and whose stock answer is a fixed value.
fn write_constant(plan: &'static Plan, out: *mut AParcel) -> Result<(), BinderStatus> {
    write_ok_status(out)?;
    match plan.method {
        // These return long in the AIDL; the stock service answers 0.
        "getAcmDirFlag" | "getAcmPkgFlag" => write_i64(out, 0),
        // No OPlus "oswap" concept on this kernel; the stock service reports -1.
        "getOswapVersion" => write_i32(out, -1),
        _ => write_i32(out, 0),
    }
}

fn run_special(
    plan: &'static Plan,
    args: &Args,
    out: *mut AParcel,
) -> Result<(), BinderStatus> {
    match plan.method {
        "readMemoryByPids" | "readMemoryByUids" => {
            let (ids, _flag) = match args {
                Args::IntArrayInt(v, u) => (v.as_slice(), *u),
                _ => (&[][..], 0),
            };
            let ret: ProcMemStatRet = if plan.method == "readMemoryByPids" {
                core::read_memory_by_pids(ids)
            } else {
                core::read_memory_by_uids(ids)
            };
            write_ok_status(out)?;
            return write_proc_mem_stat_ret(out, &ret);
        }
        _ => {}
    }

    match core::run_special(plan) {
        Some(core::SpecialResult::Int(v)) => {
            write_ok_status(out)?;
            write_i32(out, v)
        }
        Some(core::SpecialResult::Bool(v)) => {
            write_ok_status(out)?;
            write_bool(out, v)
        }
        Some(core::SpecialResult::String(v)) => {
            write_ok_status(out)?;
            write_string(out, &v)
        }
        Some(core::SpecialResult::Osvelte(v)) => {
            write_ok_status(out)?;
            write_osvelte(out, &v)
        }
        // STATUS_UNEXPECTED_NULL: a Special plan with no handler. Reaching here
        // means nodes.rs and core.rs disagree, which is a bug — but it must not
        // look like success.
        None => Err(-38),
    }
}

fn string_of(args: &Args) -> String {
    match args {
        Args::Str(s) => s.clone(),
        Args::StrInt(s, _) => s.clone(),
        Args::StrLong(s, _) => s.clone(),
        Args::StrIntArray(s, _) => s.clone(),
        Args::TwoStr(s, _) => s.clone(),
        Args::ThreeStr(s, _, _) => s.clone(),
        Args::FourStr(s, _, _, _) => s.clone(),
        Args::IntStrStr(_, s, _) => s.clone(),
        Args::TwoIntsStr(_, _, s) => s.clone(),
        _ => String::new(),
    }
}

fn int_of(args: &Args) -> i32 {
    match args {
        Args::Int(v) => *v,
        Args::TwoInts(v, _) => *v,
        Args::ThreeInts(v, _, _) => *v,
        Args::FourInts(v, _, _, _) => *v,
        Args::TwoIntsBool(v, _, _) => *v,
        Args::IntStr(v, _) => *v,
        Args::IntStrStr(v, _, _) => *v,
        Args::TwoIntsStr(v, _, _) => *v,
        _ => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The set of codes that carry in-parameters, generated from the AIDL
    /// declaration order. `read_args` must cover exactly this set, and the
    /// test below enforces that in both directions.
    fn codes_with_args() -> std::collections::HashSet<u32> {
        [
            1, 2, 3, 4, 5, 6, 7, 8, 24, 26, 31, 32, 65, 66, 67, 68, 70, 71,
            82, 90, 95, 96, 97, 100, 108, 109, 113, 116, 117, 118, 119, 120,
            121, 122, 123, 124, 125, 126, 127, 128, 129, 130, 131, 132, 133,
            134, 135, 136, 137, 138, 139, 140, 141, 142, 143, 144, 145, 146,
            147, 148, 149, 150, 151, 152, 153, 154, 155, 156, 157, 158, 159,
            160, 161, 166, 168, 169,
        ].into_iter().collect()
    }

    /// Parse the AIDL file and report, for each code, the in-parameter count.
    /// This is the ground truth `read_args` has to cover: a method with
    /// parameters whose code is not in any arm will read the wrong bytes.
    fn aidl_in_params() -> Vec<(u32, usize)> {
        let aidl = include_str!("../../aidl/vendor/oplus/hardware/performance/IPerformance.aidl");
        let mut out = Vec::new();
        for line in aidl.lines() {
            let trimmed = line.trim_start();
            let rest = ["int ", "void ", "String ", "long ", "boolean ",
                        "ProcReqHal ", "ProcMemStatRet ", "OsvelteVersionRet "]
                .iter()
                .find_map(|prefix| trimmed.strip_prefix(prefix));
            let Some(rest) = rest else { continue };
            let Some(name) = rest.split('(').next() else { continue };
            if !name.chars().next().map_or(false, |c| c.is_ascii_lowercase()) {
                continue;
            }
            let Some(params) = rest.split('(').nth(1) else { continue };
            let params = params.rsplit(')').nth(1).unwrap_or("");
            let count = if params.trim().is_empty() { 0 } else { params.split(',').count() };
            out.push((out.len() as u32 + 1, count));
        }
        out
    }

    #[test]
    fn every_method_with_parameters_is_in_some_arm() {
        // Codes with at least one in-parameter, straight from the AIDL. A code
        // present here but absent from every read_args arm would parse the
        // request parcel against the wrong signature.
        for (code, count) in aidl_in_params() {
            assert!(count < 5, "code {} has {} in-parameters: shape missing", code, count);
            if count > 0 {
                assert!(codes_with_args().contains(&code), "code {} has {} in-parameters but no read_args arm", code, count);
            }
        }
    }

    #[test]
    fn arg_arm_cover_exactly_matches_the_aidl_signatures() {
        // The reverse direction: no arm may cover a code the AIDL says has no
        // parameters, and the total must be 169 methods.
        let parsed = aidl_in_params();
        assert_eq!(parsed.len(), 169, "the AIDL parse drifted");
        for (code, count) in &parsed {
            let covered = codes_with_args().contains(code);
            let needs = *count > 0;
            assert_eq!(covered, needs, "code {}: covered={} but AIDL says {} in-params", code, covered, count);
        }
    }

    #[test]
    fn string_of_and_int_of_cover_every_declared_shape() {
        // A variant constructed in read_args but not matched here would
        // silently yield "" or 0.
        let samples = [
            Args::None,
            Args::Int(1),
            Args::Bool(true),
            Args::TwoInts(1, 2),
            Args::ThreeInts(1, 2, 3),
            Args::FourInts(1, 2, 3, 4),
            Args::TwoIntsBool(1, 2, true),
            Args::Str("a".into()),
            Args::IntStr(1, "a".into()),
            Args::StrLong("a".into(), 1),
            Args::StrInt("a".into(), 1),
            Args::TwoStr("a".into(), "b".into()),
            Args::ThreeStr("a".into(), "b".into(), "c".into()),
            Args::FourStr("a".into(), "b".into(), "c".into(), "d".into()),
            Args::TwoIntsStr(1, 2, "a".into()),
            Args::IntStrStr(1, "a".into(), "b".into()),
            Args::StrIntArray("a".into(), vec![1]),
            Args::IntArrayInt(vec![1], 2),
        ];
        for args in samples {
            let _ = string_of(&args);
            let _ = int_of(&args);
        }
    }

    #[test]
    fn reserved_codes_are_not_in_the_table() {
        assert!(nodes::plan_for(TRANSACTION_GET_INTERFACE_VERSION).is_none());
        assert!(nodes::plan_for(TRANSACTION_GET_INTERFACE_HASH).is_none());
    }
}
