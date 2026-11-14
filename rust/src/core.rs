// The method bodies that are more than a candidate-list walk.
//
// Everything expressible as "walk these nodes in order" lives in nodes.rs and
// is executed generically. What is left is here: the two memory walks, the
// kernel-version parse, the bool-from-scan coercion, the f2fs directory
// listing and the osvelte constant.

use crate::io_shim;
use crate::nodeconv;
use crate::nodes::Plan;
use crate::parcelables::{osvelte_version, stat_ret, ProcMemStat, ProcMemStatRet};

/// Field name -> the `smaps[_rollup]` key it is summed from. The order is the
/// order fields are summed in, which matters only for readability.
const SMAPS_FIELDS: [(&str, &str); 6] = [
    ("pss", "Pss:"),
    ("swap", "Swap:"),
    ("swap_rss", "SwapPss:"),
    ("anon", "Anonymous:"),
    ("file", "FilePmdMapped:"),
    ("shmem", "ShmemPmdMapped:"),
];

/// Sum one field of a smaps dump, in kB. `None` when the field is absent —
/// which the caller turns into 0, matching the C++ which tested `< 0`.
pub fn sum_smaps_field(text: &str, key: &str) -> Option<i64> {
    let mut total: i64 = 0;
    let mut found = false;
    for line in text.lines() {
        let Some(rest) = line.strip_prefix(key) else {
            continue;
        };
        let rest = rest.trim_start_matches([' ', '\t']);
        let digits: String = rest
            .chars()
            .take_while(|c| c.is_ascii_digit() || *c == '-')
            .collect();
        if digits.is_empty() {
            continue;
        }
        if let Ok(value) = digits.parse::<i64>() {
            total += value;
            found = true;
        }
    }
    if found {
        Some(total)
    } else {
        None
    }
}

/// Build a record from a `smaps`/`smaps_rollup` dump. Absent fields read 0.
fn stat_from_smaps(
    pid: i32,
    comm: String,
    oom_score_adj: u16,
    uid: i32,
    ppid: i32,
    dump: &str,
) -> ProcMemStat {
    let mut stat = ProcMemStat {
        pid,
        comm,
        oom_score_adj,
        uid,
        ppid,
        ..Default::default()
    };
    for (field, key) in SMAPS_FIELDS {
        let value = sum_smaps_field(dump, key).unwrap_or(0).max(0) as i32;
        match field {
            "pss" => stat.pss = value,
            "swap" => stat.swap = value,
            "swap_rss" => stat.swap_rss = value,
            "anon" => stat.anon = value,
            "file" => stat.file = value,
            "shmem" => stat.shmem = value,
            _ => {}
        }
    }
    stat
}

fn read_process_comm(pid: i32) -> String {
    io_shim::read(&format!("/proc/{}/comm", pid))
        .trim_end_matches('\n')
        .to_string()
}

fn read_process_oom_score_adj(pid: i32) -> u16 {
    let text = io_shim::read(&format!("/proc/{}/oom_score_adj", pid));
    if text.is_empty() {
        return 0;
    }
    nodeconv::parse_leading_int(&text).clamp(0, u16::MAX as i32) as u16
}

fn read_process_uid_ppid(pid: i32) -> (i32, i32) {
    let status = io_shim::read(&format!("/proc/{}/status", pid));
    (
        parse_status_uid(&status).unwrap_or(0),
        parse_status_ppid(&status).unwrap_or(0),
    )
}

/// `readMemoryByPids` — per-process memory.
///
/// The OPlus node was a per-uid memcg file. This kernel's memcg is per
/// app-group, so the honest per-process source is `/proc/<pid>/smaps_rollup`,
/// falling back to `/proc/<pid>/smaps`, falling back to `/proc/<pid>/statm`
/// (which matters in practice: the HAL runs in a generic `hal_*` domain that is
/// denied the per-process smaps, while statm is world-readable).
pub fn read_memory_by_pids(pids: &[i32]) -> ProcMemStatRet {
    let mut stats: Vec<ProcMemStat> = Vec::new();
    for pid in pids.iter().copied() {
        let rollup = io_shim::read(&format!("/proc/{}/smaps_rollup", pid));
        let dump = if rollup.is_empty() {
            io_shim::read(&format!("/proc/{}/smaps", pid))
        } else {
            rollup
        };
        let comm = read_process_comm(pid);
        let oom_score_adj = read_process_oom_score_adj(pid);
        let (uid, ppid) = read_process_uid_ppid(pid);

        if dump.is_empty() {
            if let Some(stat) = stat_from_statm(pid, comm, oom_score_adj, uid, ppid) {
                stats.push(stat);
            }
            continue;
        }
        stats.push(stat_from_smaps(pid, comm, oom_score_adj, uid, ppid, &dump));
    }
    stat_ret(1, stats)
}

/// statm: size resident shared text lib data dt, in pages. PSS is not in statm,
/// so it stays 0 — reporting an RSS-sized number as PSS would be a lie.
fn stat_from_statm(
    pid: i32,
    comm: String,
    oom_score_adj: u16,
    uid: i32,
    ppid: i32,
) -> Option<ProcMemStat> {
    let text = io_shim::read(&format!("/proc/{}/statm", pid));
    if text.is_empty() {
        return None;
    }
    let fields: Vec<i64> = text
        .split_whitespace()
        .filter_map(|f| f.parse::<i64>().ok())
        .collect();
    if fields.len() < 3 {
        return None;
    }
    let page_kib = io_shim::page_kib();
    Some(ProcMemStat {
        pid,
        comm,
        oom_score_adj,
        uid,
        ppid,
        vss: fields[0].saturating_mul(page_kib) as i32,
        file: fields[2].saturating_mul(page_kib) as i32,
        pss: 0,
        ..Default::default()
    })
}

/// `readMemoryByUids` — per-uid memory, as the sum of every process's
/// smaps_rollup grouped by the process's real uid. One /proc walk serves all
/// requested uids.
pub fn read_memory_by_uids(uids: &[i32]) -> ProcMemStatRet {
    if uids.is_empty() {
        return stat_ret(1, Vec::new());
    }
    let mut totals: Vec<i64> = vec![0; uids.len()];
    let mut psss: Vec<i64> = vec![0; uids.len()];
    let mut procs: Vec<i32> = vec![0; uids.len()];

    for name in io_shim::numeric_dirs("/proc") {
        let status = io_shim::read(&format!("/proc/{}/status", name));
        let Some(uid) = parse_status_uid(&status) else {
            continue;
        };
        let Some(slot) = uids.iter().position(|u| *u == uid) else {
            continue;
        };
        let rollup = io_shim::read(&format!("/proc/{}/smaps_rollup", name));
        let anon = sum_smaps_field(&rollup, "Anonymous:").unwrap_or(0);
        let pss = sum_smaps_field(&rollup, "Pss:").unwrap_or(0);
        if anon > 0 {
            totals[slot] += anon;
        }
        if pss > 0 {
            psss[slot] += pss;
        }
        procs[slot] += 1;
    }

    let mut stats: Vec<ProcMemStat> = Vec::new();
    for (slot, uid) in uids.iter().enumerate() {
        if procs[slot] == 0 {
            continue;
        }
        stats.push(ProcMemStat {
            pid: *uid,
            uid: *uid,
            comm: "uid".to_string(),
            anon: totals[slot] as i32,
            pss: psss[slot] as i32,
            ..Default::default()
        });
    }
    stat_ret(1, stats)
}

/// The first value on a `Uid:` line — real uid, which is the one memcg
/// accounting by uid has always meant.
fn parse_status_uid(status: &str) -> Option<i32> {
    parse_status_int(status, "Uid:")
}

fn parse_status_ppid(status: &str) -> Option<i32> {
    parse_status_int(status, "PPid:")
}

fn parse_status_int(status: &str, prefix: &str) -> Option<i32> {
    for line in status.lines() {
        let Some(rest) = line.strip_prefix(prefix) else {
            continue;
        };
        let first = rest.split_whitespace().next()?;
        return first.parse::<i32>().ok();
    }
    None
}

/// Run a plan that is not a plain candidate walk.
///
/// Returns `None` for plans that the generic executor in hal.rs handles, so a
/// new special case has to be added here deliberately rather than silently
/// falling through to a zero.
pub fn run_special(plan: &Plan) -> Option<SpecialResult> {
    match plan.method {
        "getKernelVersion" => Some(SpecialResult::Int(nodeconv::kernel_version(plan.nodes))),
        "isJankTaskTrackEnable" => {
            // The MTK node holds "enable=1", so the value has to be scanned
            // for rather than parsed from the front.
            Some(SpecialResult::Bool(
                nodeconv::read_int_scan_first(plan.nodes) != 0,
            ))
        }
        "getF2fsMovedBlks" => Some(SpecialResult::String(nodeconv::read_dir_first(plan.nodes))),
        "readOsvelteVersion" => {
            // Athena's memory releaser branches on major/minor to decide which
            // release path it can take; 1.0 is the value the stock service
            // reports on a kernel with no osvelte node.
            Some(SpecialResult::Osvelte(osvelte_version(0, 1, 0)))
        }
        _ => None,
    }
}

pub enum SpecialResult {
    Int(i32),
    Bool(bool),
    String(String),
    Osvelte(crate::parcelables::OsvelteVersionRet),
}

#[cfg(test)]
mod tests {
    use super::*;

    const ROLLUP: &str = "\
00400000-004a2000 r--p 00000000 fd:01 1234  /system/bin/app
Rss:                 120 kB
Pss:                 100 kB
Pss_Anon:             80 kB
Shared_Clean:         10 kB
Shared_Dirty:          5 kB
Private_Clean:        20 kB
Private_Dirty:        65 kB
Swap:                 16 kB
SwapPss:               8 kB
Anonymous:            80 kB
FilePmdMapped:        24 kB
ShmemPmdMapped:        4 kB
";

    #[test]
    fn smaps_fields_sum_and_the_anon_prefix_does_not_hijack_pss() {
        // "Pss_Anon:" starts with "Pss:"? No — it starts with "Pss_". A naive
        // prefix test on "Pss" would double count; the key includes the colon.
        assert_eq!(sum_smaps_field(ROLLUP, "Pss:"), Some(100));
        assert_eq!(sum_smaps_field(ROLLUP, "Rss:"), Some(120));
        assert_eq!(sum_smaps_field(ROLLUP, "Anonymous:"), Some(80));
        assert_eq!(sum_smaps_field(ROLLUP, "FilePmdMapped:"), Some(24));
    }

    #[test]
    fn a_multi_segment_dump_sums_every_occurrence() {
        let dump = "Rss: 10 kB\nRss: 20 kB\nRss: 3 kB\n";
        assert_eq!(sum_smaps_field(dump, "Rss:"), Some(33));
    }

    #[test]
    fn an_absent_field_is_none() {
        assert_eq!(sum_smaps_field(ROLLUP, "KernelStack:"), None);
    }

    #[test]
    fn stat_from_smaps_maps_every_field() {
        let stat = stat_from_smaps(99, "app".into(), 5, 1000, 1, ROLLUP);
        assert_eq!(stat.oom_score_adj, 5);
        assert_eq!(stat.uid, 1000);
        assert_eq!(stat.ppid, 1);
        assert_eq!(stat.pss, 100);
        assert_eq!(stat.swap, 16);
        assert_eq!(stat.swap_rss, 8);
        assert_eq!(stat.anon, 80);
        assert_eq!(stat.file, 24);
        assert_eq!(stat.shmem, 4);
    }

    #[test]
    fn status_uid_reads_the_real_uid() {
        let status = "Name:\tcom.example\nUid:\t10123\t10123\t10123\t10123\nGid:\t10123\t10123\t10123\t10123\n";
        assert_eq!(parse_status_uid(status), Some(10123));
    }

    #[test]
    fn status_without_a_uid_line_is_none() {
        assert_eq!(parse_status_uid("Name:\tkernel\n"), None);
    }

    #[test]
    fn read_memory_by_pids_walks_real_proc_files() {
        let _g = io_shim::test_lock();
        let root = concat!(env!("CARGO_MANIFEST_DIR"), "/target/test_tmp/perf_core_pids");
        let _ = std::fs::remove_dir_all(root);
        std::fs::create_dir_all(format!("{}/proc/4242", root)).unwrap();
        io_shim::set_test_root(root);
        std::fs::write(format!("{}/proc/4242/smaps_rollup", root), ROLLUP).unwrap();
        std::fs::write(format!("{}/proc/4242/comm", root), "stress-ng\n").unwrap();
        std::fs::write(format!("{}/proc/4242/oom_score_adj", root), "-600\n").unwrap();
        std::fs::write(format!("{}/proc/4242/status", root), "Uid:\t1000\nPPid:\t1\n").unwrap();
        let ret = read_memory_by_pids(&[4242]);
        assert_eq!(ret.status, 1);
        assert_eq!(ret.arr_ms.len(), 1);
        assert_eq!(ret.arr_ms[0].pid, 4242);
        assert_eq!(ret.arr_ms[0].comm, "stress-ng");
        assert_eq!(ret.arr_ms[0].oom_score_adj, 0);
        assert_eq!(ret.arr_ms[0].uid, 1000);
        assert_eq!(ret.arr_ms[0].ppid, 1);
        assert_eq!(ret.arr_ms[0].pss, 100);
        io_shim::clear_test_root();
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn an_unreadable_pid_is_skipped_not_fatal() {
        let _g = io_shim::test_lock();
        let root = concat!(env!("CARGO_MANIFEST_DIR"), "/target/test_tmp/perf_core_pids_missing");
        let _ = std::fs::remove_dir_all(root);
        std::fs::create_dir_all(format!("{}/proc", root)).unwrap();
        io_shim::set_test_root(root);
        let ret = read_memory_by_pids(&[1]);
        assert!(ret.arr_ms.is_empty());
        io_shim::clear_test_root();
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn statm_fallback_reports_vss_and_no_pss() {
        let _g = io_shim::test_lock();
        let root = concat!(env!("CARGO_MANIFEST_DIR"), "/target/test_tmp/perf_core_statm");
        let _ = std::fs::remove_dir_all(root);
        std::fs::create_dir_all(format!("{}/proc/777", root)).unwrap();
        io_shim::set_test_root(root);
        // size resident shared text lib data dt — 1000 pages vss, 10 shared
        std::fs::write(format!("{}/proc/777/statm", root), "1000 100 10 5 0 20 0\n").unwrap();
        let ret = read_memory_by_pids(&[777]);
        assert_eq!(ret.arr_ms.len(), 1);
        let page_kib = io_shim::page_kib();
        assert_eq!(ret.arr_ms[0].vss, (1000 * page_kib) as i32);
        assert_eq!(ret.arr_ms[0].file, (10 * page_kib) as i32);
        assert_eq!(ret.arr_ms[0].pss, 0);
        io_shim::clear_test_root();
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn read_memory_by_uids_groups_processes_by_uid() {
        let _g = io_shim::test_lock();
        let root = concat!(env!("CARGO_MANIFEST_DIR"), "/target/test_tmp/perf_core_uids");
        let _ = std::fs::remove_dir_all(root);
        for pid in ["100", "101", "200"] {
            std::fs::create_dir_all(format!("{}/proc/{}", root, pid)).unwrap();
        }
        io_shim::set_test_root(root);
        std::fs::write(
            format!("{}/proc/100/status", root),
            "Name:\ta\nUid:\t10123\t10123\t10123\t10123\n",
        )
        .unwrap();
        std::fs::write(
            format!("{}/proc/101/status", root),
            "Name:\tb\nUid:\t10123\t10123\t10123\t10123\n",
        )
        .unwrap();
        std::fs::write(
            format!("{}/proc/200/status", root),
            "Name:\tc\nUid:\t10999\t10999\t10999\t10999\n",
        )
        .unwrap();
        std::fs::write(
            format!("{}/proc/100/smaps_rollup", root),
            "Anonymous: 10 kB\nPss: 8 kB\n",
        )
        .unwrap();
        std::fs::write(
            format!("{}/proc/101/smaps_rollup", root),
            "Anonymous: 20 kB\nPss: 16 kB\n",
        )
        .unwrap();
        std::fs::write(
            format!("{}/proc/200/smaps_rollup", root),
            "Anonymous: 40 kB\nPss: 32 kB\n",
        )
        .unwrap();
        // a non-numeric /proc entry must be ignored
        std::fs::create_dir_all(format!("{}/proc/sys", root)).unwrap();

        let ret = read_memory_by_uids(&[10123, 10999, 55555]);
        assert_eq!(ret.arr_ms.len(), 2, "the absent uid must not produce a record");
        assert_eq!(ret.arr_ms[0].pid, 10123);
        assert_eq!(ret.arr_ms[0].comm, "uid");
        assert_eq!(ret.arr_ms[0].anon, 30);
        assert_eq!(ret.arr_ms[0].pss, 24);
        assert_eq!(ret.arr_ms[1].pid, 10999);
        assert_eq!(ret.arr_ms[1].anon, 40);

        io_shim::clear_test_root();
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn read_memory_by_uids_of_nothing_returns_nothing() {
        let ret = read_memory_by_uids(&[]);
        assert!(ret.arr_ms.is_empty());
    }
}