// Every node this HAL touches, keyed by method.
//
// Policy, carried over from the C++ backend and preserved exactly: the ORIGINAL
// OPlus node is always the FIRST candidate, this kernel's MTK/Transsion
// equivalent is the fallback. The first candidate that yields data (reads) or
// accepts the write (writes) wins. That makes the HAL faithful on an OPlus
// kernel and useful on the MT6893.
//
// The tables are static slices rather than per-method `static c[]` arrays so
// they can be enumerated, counted and checked against the AIDL surface in
// tests. `NODE_TABLE` below is the index; `plan_for` is the only way the method
// bodies reach them.

/// What a method does to its nodes.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Op {
    /// Return the first non-empty read.
    ReadString,
    /// Return the first read parsed as an integer, else 0.
    ReadInt,
    /// Return the first integer appearing anywhere in the text.
    ReadIntScan,
    /// Return whether any candidate exists.
    Exists,
    /// Write the string argument to the first node that accepts it.
    Write,
    /// Write the integer argument to the first node that accepts it.
    WriteInt,
    /// Read through `ProcReqHal` (first non-empty read goes in `target`).
    ReadStruct,
    /// Multi-node, multi-step: not expressible as one table entry.
    Special,
    /// No device-visible effect; the return is a constant.
    Constant,
}

/// One method's node plan.
pub struct Plan {
    /// AIDL method name. The transaction code is the index into the AIDL
    /// declaration order plus one, so this is the identity used everywhere.
    pub method: &'static str,
    pub op: Op,
    /// Candidate paths. A `%s` is replaced by the method's string argument.
    pub nodes: &'static [&'static str],
}

/// The index. Declaration order is the transaction-code order 1..=169 and
/// MUST NOT be reordered: AIDL assigns codes by declaration order, so any
/// shuffle breaks wire compatibility with the shipped HAL and every prebuilt
/// client.
pub static NODE_TABLE: &[Plan] = &[
    Plan { method: "addAcmDirName", op: Op::Constant, nodes: &[] },
    Plan { method: "addAcmNomediaDirName", op: Op::Constant, nodes: &[] },
    Plan { method: "addAcmPkgName", op: Op::Constant, nodes: &[] },
    Plan { method: "addTaskTrackPid", op: Op::ReadInt, nodes: &["/proc/jank_info/cpu_jank_info/task_track", "/proc/trans_scheduler/debug"] },
    Plan { method: "clearTaskTrackGroup", op: Op::WriteInt, nodes: &["/proc/jank_info/cpu_jank_info/task_track", "/proc/trans_scheduler/debug"] },
    Plan { method: "delAcmDirName", op: Op::Constant, nodes: &[] },
    Plan { method: "delAcmNomediaDirName", op: Op::Constant, nodes: &[] },
    Plan { method: "delAcmPkgName", op: Op::Constant, nodes: &[] },
    Plan { method: "disableDamonReclaim", op: Op::ReadInt, nodes: &["/sys/module/damon_reclaim/parameters/enabled"] },
    Plan { method: "disableKmallocDebug", op: Op::ReadInt, nodes: &["/proc/oplus_healthinfo/kmalloc_debug_enable", "/proc/slabinfo"] },
    Plan { method: "disableMultiThreadOptimize", op: Op::ReadInt, nodes: &["/proc/sys/kernel/cpu_multi_thread", "/proc/cpu_loading/onoff"] },
    Plan { method: "disableProcessReclaim", op: Op::ReadInt, nodes: &["/proc/oplus_healthinfo/process_reclaim_enable", "/proc/swaps"] },
    Plan { method: "disableTaskCpustats", op: Op::ReadInt, nodes: &["/proc/task_info/task_cpustats/task_cpustats_enable", "/proc/cpu_loading/onoff"] },
    Plan { method: "disableTaskPlacementDecision", op: Op::ReadInt, nodes: &["/proc/task_info/task_sched_info/task_sched_info_enable", "/proc/trans_scheduler/enable"] },
    Plan { method: "disableVmallocDebug", op: Op::ReadInt, nodes: &["/proc/oplus_healthinfo/vmalloc_debug_enable", "/proc/vmallocinfo"] },
    Plan { method: "enableDamonReclaim", op: Op::ReadInt, nodes: &["/sys/module/damon_reclaim/parameters/enabled"] },
    Plan { method: "enableKmallocDebug", op: Op::ReadInt, nodes: &["/proc/oplus_healthinfo/kmalloc_debug_enable", "/proc/slabinfo"] },
    Plan { method: "enableMultiThreadOptimize", op: Op::ReadInt, nodes: &["/proc/sys/kernel/cpu_multi_thread", "/proc/cpu_loading/onoff"] },
    Plan { method: "enableProcessReclaim", op: Op::ReadInt, nodes: &["/proc/oplus_healthinfo/process_reclaim_enable", "/proc/swaps"] },
    Plan { method: "enableTaskCpustats", op: Op::ReadInt, nodes: &["/proc/task_info/task_cpustats/task_cpustats_enable", "/proc/cpu_loading/onoff"] },
    Plan { method: "enableTaskPlacementDecision", op: Op::ReadInt, nodes: &["/proc/task_info/task_sched_info/task_sched_info_enable", "/proc/trans_scheduler/enable"] },
    Plan { method: "enableVmallocDebug", op: Op::ReadInt, nodes: &["/proc/oplus_healthinfo/vmalloc_debug_enable", "/proc/vmallocinfo"] },
    Plan { method: "existMemMonitor", op: Op::Exists, nodes: &["/proc/memory_monitor", "/proc/meminfo"] },
    Plan { method: "getAcmDirFlag", op: Op::Constant, nodes: &[] },
    Plan { method: "getAcmOpstat", op: Op::Constant, nodes: &[] },
    Plan { method: "getAcmPkgFlag", op: Op::Constant, nodes: &[] },
    Plan { method: "getDdrResidency", op: Op::ReadString, nodes: &["/sys/power/ddr/residency", "/proc/zoneinfo"] },
    Plan { method: "getDevinfoUfsInfo", op: Op::ReadString, nodes: &["/proc/devinfo/ufs", "/sys/block/sda/device/model", "/proc/ufs_debug"] },
    Plan { method: "getDevinfoDDRInfo", op: Op::ReadString, nodes: &["/proc/devinfo/ddr", "/proc/zoneinfo"] },
    Plan { method: "getDevinfoUfsVersionInfo", op: Op::ReadString, nodes: &["/proc/devinfo/ufs_version", "/sys/devices/platform/soc/1d84000.ufshc/device_descriptor/specification_version", "/proc/ufs_debug"] },
    Plan { method: "getExt4FragScore", op: Op::ReadString, nodes: &["/proc/fs/ext4/%s/mb_groups", "/proc/fs/ext4/%s/mb_stats"] },
    Plan { method: "getExt4FreefragInfo", op: Op::ReadString, nodes: &["/proc/fs/ext4/%s/mb_stats", "/proc/fs/ext4/%s/mb_groups"] },
    Plan { method: "getF2fsMovedBlks", op: Op::Special, nodes: &["/sys/fs/f2fs/", "/proc/fs/ext4/"] },
    Plan { method: "getHIAllocWait", op: Op::ReadStruct, nodes: &["/proc/oplus_healthinfo/alloc_wait", "/proc/pressure/memory"] },
    Plan { method: "getHICpuInfo", op: Op::ReadString, nodes: &["/data/oplus/cpu/cpu_loading", "/proc/cpu_loading/curr_cpu_loading"] },
    Plan { method: "getHICpuLoading", op: Op::ReadString, nodes: &["/data/oplus/cpu/cpu_loading", "/proc/cpu_loading/curr_cpu_loading"] },
    Plan { method: "getHIDState", op: Op::ReadStruct, nodes: &["/proc/oplus_healthinfo/dstate", "/proc/pressure/cpu"] },
    Plan { method: "getHIEmcdrvIowait", op: Op::ReadStruct, nodes: &["/proc/oplus_healthinfo/emcdrv_iowait", "/proc/pressure/io"] },
    Plan { method: "getHIFsyncWait", op: Op::ReadStruct, nodes: &["/proc/oplus_healthinfo/fsync_wait", "/proc/pressure/io"] },
    Plan { method: "getHIIonWait", op: Op::ReadStruct, nodes: &["/proc/oplus_healthinfo/ion_wait", "/proc/pressure/memory"] },
    Plan { method: "getHIIowait", op: Op::ReadStruct, nodes: &["/proc/oplus_healthinfo/iowait", "/proc/pressure/io"] },
    Plan { method: "getHIIowaitHung", op: Op::ReadStruct, nodes: &["/proc/oplus_healthinfo/iowait_hung", "/proc/pressure/io"] },
    Plan { method: "getHIKswapdLoading", op: Op::ReadStruct, nodes: &["/proc/vmstat", "/proc/vmstat"] },
    Plan { method: "getHISchedLatency", op: Op::ReadStruct, nodes: &["/proc/oplus_healthinfo/sched_latency", "/proc/schedstat"] },
    Plan { method: "getHIScmCall", op: Op::ReadStruct, nodes: &["/proc/oplus_healthinfo/scm_call", "/proc/sched_debug"] },
    Plan { method: "getHIUfsFeature", op: Op::ReadStruct, nodes: &["/proc/oplus_healthinfo/ufs_feature", "/proc/ufs_debug"] },
    Plan { method: "getKernelVersion", op: Op::Special, nodes: &["/proc/version"] },
    Plan { method: "getKmallocDebug", op: Op::ReadString, nodes: &["/proc/oplus_healthinfo/kmalloc_debug", "/proc/slabinfo"] },
    Plan { method: "getKmallocOrigin", op: Op::ReadString, nodes: &["/proc/oplus_healthinfo/kmalloc_origin", "/proc/slabinfo"] },
    Plan { method: "getKmallocUsed", op: Op::ReadString, nodes: &["/proc/oplus_healthinfo/kmalloc_used", "/proc/slabinfo"] },
    Plan { method: "getMemMonitor", op: Op::ReadString, nodes: &["/proc/oplus_mem/memleak_detect/memory_monitor", "/proc/meminfo"] },
    Plan { method: "getOswapVersion", op: Op::Constant, nodes: &[] },
    Plan { method: "getUfsSignalRecordUpload", op: Op::ReadString, nodes: &["/proc/ufs_signalShow/record_upload", "/sys/devices/platform/soc/1d84000.ufshc/device_descriptor/ufs_features", "/proc/ufs_debug"] },
    Plan { method: "getUfsplusHpbStatus", op: Op::ReadIntScan, nodes: &["/proc/devinfo/ufsplus_status", "/sys/devices/platform/soc/1d84000.ufshc/wb_on", "/proc/ufs_debug"] },
    Plan { method: "getUfsplusTwStatus", op: Op::ReadIntScan, nodes: &["/proc/devinfo/ufsplus_status", "/sys/devices/platform/soc/1d84000.ufshc/wb_on", "/proc/ufs_debug"] },
    Plan { method: "getVmallocDebug", op: Op::ReadString, nodes: &["/proc/oplus_healthinfo/vmalloc_debug", "/proc/vmallocinfo"] },
    Plan { method: "getVmallocHashCal", op: Op::ReadString, nodes: &["/proc/oplus_healthinfo/vmalloc_hash_cal", "/proc/vmallocinfo"] },
    Plan { method: "getVmallocUsed", op: Op::ReadString, nodes: &["/proc/oplus_healthinfo/vmalloc_used", "/proc/vmallocinfo"] },
    Plan { method: "getallocwait", op: Op::ReadString, nodes: &["/proc/oplus_healthinfo/alloc_wait", "/proc/pressure/memory"] },
    Plan { method: "getdstate", op: Op::ReadString, nodes: &["/proc/oplus_healthinfo/dstate", "/proc/pressure/cpu"] },
    Plan { method: "getfsyncwait", op: Op::ReadString, nodes: &["/proc/oplus_healthinfo/fsync_wait", "/proc/pressure/io"] },
    Plan { method: "getionwait", op: Op::ReadString, nodes: &["/proc/oplus_healthinfo/iowait", "/proc/pressure/io"] },
    Plan { method: "getiowait", op: Op::ReadString, nodes: &["/proc/oplus_healthinfo/iowait", "/proc/pressure/io"] },
    Plan { method: "getschedlatency", op: Op::ReadString, nodes: &["/proc/oplus_healthinfo/sched_latency", "/proc/schedstat"] },
    Plan { method: "hybridswap_memcg_para_read", op: Op::ReadString, nodes: &["/dev/memcg/apps", "/dev/memcg/memory.usage_in_bytes"] },
    Plan { method: "hybridswap_memcg_para_write", op: Op::ReadInt, nodes: &["/dev/memcg/apps/memory.swappiness"] },
    Plan { method: "hybridswap_zram_para_read", op: Op::ReadString, nodes: &["/sys/block/zram0/hybridswap_enable", "/sys/block/zram0/mm_stat"] },
    Plan { method: "hybridswap_zram_para_write", op: Op::ReadInt, nodes: &["/sys/block/zram0/hybridswap_enable", "/sys/block/zram0/mm_stat"] },
    Plan { method: "isJankTaskTrackEnable", op: Op::Special, nodes: &["/proc/jank_info/cpu_jank_info/task_track_enable", "/proc/trans_scheduler/enable"] },
    Plan { method: "perProcessMemReadahead", op: Op::ReadInt, nodes: &["/proc/process_reclaim", "/proc/swaps"] },
    Plan { method: "perProcessMemReclaim", op: Op::ReadInt, nodes: &["/proc/process_reclaim", "/proc/swaps"] },
    Plan { method: "readCallStack", op: Op::ReadString, nodes: &["/proc/jank_info/cpu_jank_info/callstack", "/proc/self/stack"] },
    Plan { method: "readClmEnable", op: Op::ReadString, nodes: &["/proc/jank_info/cpu_jank_info/clm_enable", "/proc/cpu_loading/onoff"] },
    Plan { method: "readClmHighLoadAll", op: Op::ReadString, nodes: &["/proc/jank_info/cpu_jank_info/clm_highload_all", "/proc/cpu_loading/overThrhld"] },
    Plan { method: "readClmHighLoadGrp", op: Op::ReadString, nodes: &["/proc/jank_info/cpu_jank_info/clm_highload_grp", "/proc/cpu_loading/specify_overThrhld"] },
    Plan { method: "readClmLowLoadGrp", op: Op::ReadString, nodes: &["/proc/jank_info/cpu_jank_info/clm_lowload_grp", "/proc/cpu_loading/underThrhld"] },
    Plan { method: "readCpuTaskstats", op: Op::ReadString, nodes: &["/proc/task_cpustats", "/proc/uid_cputime/show_uid_stat"] },
    Plan { method: "readDBacktrace", op: Op::ReadString, nodes: &["/proc/task_info/task_sched_info/d_backtrace", "/proc/self/stack"] },
    Plan { method: "readDConvert", op: Op::ReadString, nodes: &["/proc/task_info/task_sched_info/d_convert", "/proc/sched_debug"] },
    Plan { method: "readFgFreqsThreshold", op: Op::ReadString, nodes: &["/proc/jank_info/cpu_jank_info/fg_freqs_threshold", "/sys/devices/system/cpu/cpufreq/policy0/scaling_min_freq"] },
    Plan { method: "readIOBacktrace", op: Op::ReadString, nodes: &["/proc/task_info/task_sched_info/io_backtrace", "/proc/self/stack"] },
    Plan { method: "readIomonitorInfo", op: Op::ReadString, nodes: &["/proc/IoMonitor/uid_io_show", "/proc/uid_io/stats"] },
    Plan { method: "readJankCpuIndicator", op: Op::ReadString, nodes: &["/proc/jank_info/cpu_jank_info/load_indicator", "/proc/loadavg"] },
    Plan { method: "readJankCpuInfo", op: Op::ReadString, nodes: &["/proc/jank_info/cpu_jank_info/cpu_info", "/proc/cpuinfo"] },
    Plan { method: "readJankCpuInfoSig", op: Op::ReadString, nodes: &["/proc/jank_info/cpu_jank_info/cpu_info_sig", "/proc/cpuinfo"] },
    Plan { method: "readJankCpuLoad", op: Op::ReadString, nodes: &["/proc/jank_info/cpu_jank_info/cpu_load", "/proc/cpu_loading/curr_cpu_loading"] },
    Plan { method: "readJankCpuLoad32", op: Op::ReadString, nodes: &["/proc/jank_info/cpu_jank_info/cpu_load32", "/proc/cpu_loading/curr_cpu_loading"] },
    Plan { method: "readJankCpuLoad32Scale", op: Op::ReadString, nodes: &["/proc/jank_info/cpu_jank_info/cpu_load32_scale", "/proc/cpu_loading/curr_cpu_loading"] },
    Plan { method: "readJankTaskTrack", op: Op::ReadString, nodes: &["/proc/jank_info/cpu_jank_info/task_track", "/proc/trans_scheduler/debug"] },
    Plan { method: "readJankTaskTrackByPid", op: Op::ReadString, nodes: &["/proc/jank_info/cpu_jank_info/task_track", "/proc/trans_scheduler/debug"] },
    Plan { method: "readJankVersion", op: Op::ReadString, nodes: &["/proc/jank_info/cpu_jank_info/version", "/proc/version"] },
    Plan { method: "readKmallocDebugCreate", op: Op::ReadString, nodes: &["/proc/oplus_healthinfo/kmalloc_debug_create", "/proc/slabinfo"] },
    Plan { method: "readLimitTable", op: Op::ReadString, nodes: &["/proc/task_info/task_sched_info/limit_table", "/proc/sched_debug"] },
    Plan { method: "readMemleakDetectThread", op: Op::ReadString, nodes: &["/proc/oplus_healthinfo/memleak_detect_thread", "/proc/leakage/spower_test"] },
    Plan { method: "readMemoryByPids", op: Op::Special, nodes: &[] },
    Plan { method: "readMemoryByUids", op: Op::Special, nodes: &[] },
    Plan { method: "readNandswapProc", op: Op::ReadString, nodes: &["/proc/nandswap/fn_enable", "/proc/swaps"] },
    Plan { method: "readNormalizeRealTime", op: Op::ReadString, nodes: &["/proc/task_info/task_load_info/normalize_real_time", "/proc/pressure/cpu"] },
    Plan { method: "readNormalizeRunningTime", op: Op::ReadString, nodes: &["/proc/task_info/task_load_info/normalize_running_time", "/proc/pressure/cpu"] },
    Plan { method: "readOplusReserve3", op: Op::ReadString, nodes: &["/dev/block/by-name/oplusreserve3"] },
    Plan { method: "readOsvelteVersion", op: Op::Special, nodes: &["/dev/osvelte"] },
    Plan { method: "readPidsSet", op: Op::ReadString, nodes: &["/proc/task_info/task_sched_info/pids_set", "/proc/uid_procstat/set"] },
    Plan { method: "readRealTime", op: Op::ReadString, nodes: &["/proc/task_info/task_load_info/real_time", "/proc/uptime"] },
    Plan { method: "readRunningTime", op: Op::ReadString, nodes: &["/proc/task_info/task_load_info/running_time", "/proc/uptime"] },
    Plan { method: "readSchedInfoThreshold", op: Op::ReadString, nodes: &["/proc/task_info/task_sched_info/sched_info_threshold", "/proc/sched_debug"] },
    Plan { method: "readSgeFreqInfo", op: Op::ReadString, nodes: &["/proc/sgefreqinfo", "/sys/class/kgsl/kgsl-3d0/clock_mhz", "/sys/class/kgsl/kgsl-3d0/gpuclk", "/sys/class/devfreq/13000000.mali/cur_freq", "/sys/kernel/ged/hal/current_freqency", "/proc/gpufreqv2/gpufreq_status"] },
    Plan { method: "readSgeInfo", op: Op::ReadString, nodes: &["/proc/sgeinfo", "/sys/class/kgsl/kgsl-3d0/gpu_busy_percentage", "/sys/class/kgsl/kgsl-3d0/gpubusy", "/proc/mtk_mali/utilization", "/sys/kernel/ged/hal/gpu_utilization", "/proc/mtk_mali/gpu_memory"] },
    Plan { method: "readStorageFeature", op: Op::ReadString, nodes: &["/proc/fs/ext4/dm-11/mb_groups", "/sys/fs/f2fs/dm-48/lifetime_write_kbytes", "/sys/fs/f2fs/dm-39"] },
    Plan { method: "readTargetProcess", op: Op::ReadString, nodes: &["/proc/task_info/task_load_info/target_process", "/proc/self/comm"] },
    Plan { method: "readTaskCpustatsEnable", op: Op::ReadString, nodes: &["/proc/task_info/task_cpustats/task_cpustats_enable", "/proc/cpu_loading/onoff"] },
    Plan { method: "readTaskSchedInfo", op: Op::ReadString, nodes: &["/proc/task_info/task_sched_info/version", "/proc/sched_debug"] },
    Plan { method: "readTidsSet", op: Op::ReadString, nodes: &["/proc/task_info/task_sched_info/tids_set", "/proc/uid_procstat/set"] },
    Plan { method: "readUxTaskTrack", op: Op::ReadString, nodes: &["/proc/oplus_scheduler/sched_assist/ux_task", "/proc/trans_scheduler/debug"] },
    Plan { method: "readVaFeature", op: Op::ReadString, nodes: &["/proc/oplus_healthinfo/va_feature", "/proc/self/maps"] },
    Plan { method: "readVersion", op: Op::ReadString, nodes: &["/proc/version"] },
    Plan { method: "removeTaskTrackPid", op: Op::WriteInt, nodes: &["/proc/jank_info/cpu_jank_info/task_track", "/proc/trans_scheduler/debug"] },
    Plan { method: "searchAcmNomediaDirName", op: Op::Constant, nodes: &[] },
    Plan { method: "setAcmOpstat", op: Op::Constant, nodes: &[] },
    Plan { method: "setDamonReclaimColdTime", op: Op::ReadInt, nodes: &["/sys/module/damon_reclaim/parameters/min_age"] },
    Plan { method: "setDamonReclaimMonitoring", op: Op::ReadInt, nodes: &["/sys/module/damon_reclaim/parameters/sample_interval"] },
    Plan { method: "setDamonReclaimQuota", op: Op::ReadInt, nodes: &["/sys/module/damon_reclaim/parameters/quota_ms"] },
    Plan { method: "setDamonReclaimWmarks", op: Op::ReadInt, nodes: &["/sys/module/damon_reclaim/parameters/wmarks_low"] },
    Plan { method: "setFgUids", op: Op::Write, nodes: &["/proc/fg_info/fg_uids", "/proc/uid_procstat/set"] },
    Plan { method: "setFrameRate", op: Op::Write, nodes: &["/proc/sys/kernel/frame_rate", "/proc/tp_report_rate_switch"] },
    Plan { method: "setImFlag", op: Op::Write, nodes: &["/proc/oplus_scheduler/sched_assist/im_flag", "/data/vendor/trans_sched/sched_impt_task"] },
    Plan { method: "setProcessReclaim", op: Op::ReadInt, nodes: &["/proc/process_reclaim", "/proc/swaps"] },
    Plan { method: "setSchedAssistImptTask", op: Op::Write, nodes: &["/proc/sys/kernel/sched_impt_tgid", "/data/vendor/trans_sched/sched_impt_task"] },
    Plan { method: "setSchedAssistScene", op: Op::Write, nodes: &["/proc/sys/kernel/sched_assist_scene", "/proc/sys/kernel/sched_util_clamp_min", "/proc/game_mode"] },
    Plan { method: "setSlideboost", op: Op::Write, nodes: &["/proc/sys/kernel/slide_boost_enabled", "/proc/sys/kernel/sched_util_clamp_min", "/proc/game_state"] },
    Plan { method: "setTpdID", op: Op::ReadInt, nodes: &["/sys/module/tpd/parameters/tpd_id"] },
    Plan { method: "setTpdSerialParams", op: Op::ReadInt, nodes: &["/sys/module/tpd/parameters/tpd_cmds"] },
    Plan { method: "writeClmEnable", op: Op::Write, nodes: &["/proc/jank_info/cpu_jank_info/clm_enable", "/proc/cpu_loading/onoff"] },
    Plan { method: "writeClmHighLoadAll", op: Op::Write, nodes: &["/proc/jank_info/cpu_jank_info/clm_highload_all", "/proc/cpu_loading/overThrhld"] },
    Plan { method: "writeClmHighLoadGrp", op: Op::Write, nodes: &["/proc/jank_info/cpu_jank_info/clm_highload_grp", "/proc/cpu_loading/specify_overThrhld"] },
    Plan { method: "writeClmLowLoadGrp", op: Op::Write, nodes: &["/proc/jank_info/cpu_jank_info/clm_lowload_grp", "/proc/cpu_loading/underThrhld"] },
    Plan { method: "writeDBacktrace", op: Op::Constant, nodes: &[] },
    Plan { method: "writeFgFreqsThreshold", op: Op::Write, nodes: &["/proc/jank_info/cpu_jank_info/fg_freqs_threshold", "/sys/devices/system/cpu/cpufreq/policy0/scaling_min_freq"] },
    Plan { method: "writeIOBacktrace", op: Op::Write, nodes: &["/proc/task_info/task_sched_info/io_backtrace", "/proc/self/stack"] },
    Plan { method: "writeJankTaskTrackEnable", op: Op::WriteInt, nodes: &["/proc/jank_info/cpu_jank_info/task_track_enable", "/proc/trans_scheduler/enable"] },
    Plan { method: "writeKmallocDebugCreate", op: Op::ReadInt, nodes: &["/proc/oplus_healthinfo/kmalloc_debug_create", "/proc/slabinfo"] },
    Plan { method: "writeKmallocDebugCreateWithType", op: Op::ReadInt, nodes: &["/proc/oplus_healthinfo/kmalloc_debug_create", "/proc/slabinfo"] },
    Plan { method: "writeMemMonitor", op: Op::ReadInt, nodes: &["/proc/memory_monitor", "/proc/meminfo"] },
    Plan { method: "writeMemleakDetectThread", op: Op::ReadInt, nodes: &["/proc/oplus_healthinfo/memleak_detect_thread", "/proc/leakage/spower_test"] },
    Plan { method: "writeMonitorStatus", op: Op::Constant, nodes: &[] },
    Plan { method: "writeNandswapProc", op: Op::ReadInt, nodes: &["/proc/nandswap/fn_enable", "/proc/swaps"] },
    Plan { method: "writeOplusReserve3", op: Op::ReadInt, nodes: &["/dev/block/by-name/oplusreserve3"] },
    Plan { method: "writePidsSet", op: Op::Write, nodes: &["/proc/task_info/task_sched_info/pids_set", "/proc/uid_procstat/set"] },
    Plan { method: "writeSchedInfoThreshold", op: Op::Write, nodes: &["/proc/task_info/task_sched_info/sched_info_threshold", "/proc/sched_debug"] },
    Plan { method: "writeStorageFeature", op: Op::ReadInt, nodes: &["/proc/fs/ext4/dm-11/mb_groups", "/sys/fs/f2fs/dm-48/lifetime_write_kbytes", "/sys/fs/f2fs/dm-39"] },
    Plan { method: "writeTaskSchedInfo", op: Op::Write, nodes: &["/proc/task_info/task_sched_info/task_sched_info_enable", "/proc/sched_debug"] },
    Plan { method: "writeTidsSet", op: Op::Write, nodes: &["/proc/task_info/task_sched_info/tids_set", "/proc/uid_procstat/set"] },
    Plan { method: "writeUxState", op: Op::ReadInt, nodes: &["/proc/oplus_scheduler/sched_assist/ux_task", "/proc/trans_scheduler/debug"] },
    Plan { method: "writeVaFeature", op: Op::ReadInt, nodes: &["/proc/oplus_healthinfo/va_feature", "/proc/self/maps"] },
    Plan { method: "setFreqGoverner", op: Op::ReadInt, nodes: &["/sys/devices/system/cpu/cpufreq/policy0/scaling_governor"] },
    Plan { method: "enableAudioPerf", op: Op::ReadInt, nodes: &["/proc/oplus_scheduler/sched_assist/audio/enable"] },
    Plan { method: "writeTmemoryMemory", op: Op::ReadInt, nodes: &["/sys/tmemory/"] },
    Plan { method: "writeTmemorySwitch", op: Op::ReadInt, nodes: &["/sys/tmemory/"] },
    Plan { method: "writeTmemoryFlushBusy", op: Op::ReadInt, nodes: &["/sys/tmemory/"] },
    Plan { method: "writeTmemoryFlushIdle", op: Op::ReadInt, nodes: &["/sys/tmemory/"] },
    Plan { method: "writeTmemoryCapacity", op: Op::ReadInt, nodes: &["/sys/tmemory/"] },
    Plan { method: "writeTmemoryHighWaterRatio", op: Op::ReadInt, nodes: &["/sys/tmemory/"] },
    Plan { method: "readTmemorySysdirtypages", op: Op::ReadString, nodes: &["/sys/tmemory/"] },
    Plan { method: "readTmemoryDirtypages", op: Op::ReadString, nodes: &["/sys/tmemory/"] },
    Plan { method: "readTmemoryErrorStat", op: Op::ReadString, nodes: &["/sys/tmemory/"] },
    Plan { method: "readTmemoryIoLatency", op: Op::ReadString, nodes: &["/sys/tmemory/"] },
    Plan { method: "getClmThreshold", op: Op::ReadString, nodes: &["/proc/jank_info/cpu_jank_info/clm_report_threshold", "/proc/cpu_loading/overThrhld"] },
    Plan { method: "getClmMuxSwitch", op: Op::ReadString, nodes: &["/data/vendor/trans_sched/cpu/clm_mux_switch", "/proc/cpu_loading/onoff"] },
    Plan { method: "setClmThreshold", op: Op::Write, nodes: &["/proc/jank_info/cpu_jank_info/clm_report_threshold", "/proc/cpu_loading/overThrhld"] },
    Plan { method: "setClmMuxSwitch", op: Op::Write, nodes: &["/data/vendor/trans_sched/cpu/clm_mux_switch", "/proc/cpu_loading/onoff"] },
];

/// Transaction code -> plan. Codes are 1-based; code 0 is the "no such method"
/// slot AIDL never uses.
pub fn plan_for(code: u32) -> Option<&'static Plan> {
    if code == 0 {
        return None;
    }
    NODE_TABLE.get((code - 1) as usize)
}

/// Substitute the method's string argument into a `%s` template.
pub fn subst(template: &str, arg: &str) -> String {
    match template.find("%s") {
        Some(pos) => {
            let mut out = String::with_capacity(template.len() + arg.len());
            out.push_str(&template[..pos]);
            out.push_str(arg);
            out.push_str(&template[pos + 2..]);
            out
        }
        None => template.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn table_has_one_plan_per_aidl_method() {
        assert_eq!(NODE_TABLE.len(), 169);
    }

    #[test]
    fn every_code_resolves_and_out_of_range_does_not() {
        for code in 1..=169u32 {
            assert!(plan_for(code).is_some(), "code {} missing", code);
            assert_eq!(plan_for(code).unwrap().method, NODE_TABLE[code as usize - 1].method);
        }
        assert!(plan_for(0).is_none());
        assert!(plan_for(170).is_none());
    }

    #[test]
    fn spot_check_the_declaration_order() {
        // The first and last entries plus a few landmarks: a reordered table
        // would still be 169 long, so the anchors matter. Every index here was
        // taken from the AIDL declaration order, not assumed.
        assert_eq!(NODE_TABLE[0].method, "addAcmDirName");
        assert_eq!(NODE_TABLE[3].method, "addTaskTrackPid");
        assert_eq!(NODE_TABLE[94].method, "readMemoryByPids");
        assert_eq!(NODE_TABLE[114].method, "readVersion");
        assert_eq!(NODE_TABLE[168].method, "setClmMuxSwitch");
    }

    #[test]
    fn table_matches_the_aidl_declaration_order_exactly() {
        // The anchors above catch a small slip; this catches any slip. The
        // AIDL file is the contract, so the two orders are compared in full.
        let aidl = include_str!("../../aidl/vendor/oplus/hardware/performance/IPerformance.aidl");
        let mut declared = Vec::new();
        for line in aidl.lines() {
            let trimmed = line.trim_start();
            let name = ["int ", "void ", "String ", "long ", "boolean ",
                        "ProcReqHal ", "ProcMemStatRet ", "OsvelteVersionRet "]
                .iter()
                .find_map(|prefix| trimmed.strip_prefix(prefix))
                .and_then(|rest| rest.split('(').next());
            if let Some(name) = name {
                if name.chars().next().map_or(false, |c| c.is_ascii_lowercase()) {
                    declared.push(name.to_string());
                }
            }
        }
        assert_eq!(declared.len(), 169, "the AIDL parse drifted");
        for (index, name) in declared.iter().enumerate() {
            assert_eq!(
                NODE_TABLE[index].method, *name,
                "index {} (code {}): table says {}, AIDL says {}",
                index, index + 1, NODE_TABLE[index].method, name
            );
        }
    }

    #[test]
    fn subst_replaces_only_the_first_placeholder() {
        assert_eq!(subst("/proc/fs/ext4/%s/mb_groups", "dm-11"), "/proc/fs/ext4/dm-11/mb_groups");
        assert_eq!(subst("/proc/version", "x"), "/proc/version");
        assert_eq!(subst("a%s%s", "b"), "ab%s");
    }

    #[test]
    fn every_string_op_that_reads_has_at_least_one_node() {
        for plan in NODE_TABLE {
            let needs_nodes = !matches!(plan.op, Op::Constant | Op::Special);
            if needs_nodes {
                assert!(!plan.nodes.is_empty(), "{} has no candidates", plan.method);
            }
        }
    }
}