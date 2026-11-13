# OPlus node -> this device's equivalent

Generated from the remap table in scripts/gen_service_remap.py — do not edit by hand.

Every MTK/Transsion path below was confirmed to exist on device
102562537K104189 (Infinix GT 10 Pro / X6739, 5.10.270-StargazerKernel-CelestialSphere).
The original OPlus node stays the PRIMARY candidate; the equivalent is the
fallback, so the HAL is faithful on an OPlus kernel and returns real data here.

| # | method | op | original OPlus node | this device's equivalent |
|---|--------|----|---------------------|------------------------|
| 1 | `addTaskTrackPid` | file | `/proc/jank_info/cpu_jank_info/task_track` | /proc/trans_scheduler/debug |
| 2 | `clearTaskTrackGroup` | file | `/proc/jank_info/cpu_jank_info/task_track` | /proc/trans_scheduler/debug |
| 3 | `disableDamonReclaim` | file | `/sys/module/damon_reclaim/parameters/enabled` | _(none: graceful default)_ |
| 4 | `disableKmallocDebug` | file | `/proc/oplus_healthinfo/kmalloc_debug_enable` | /proc/slabinfo |
| 5 | `disableMultiThreadOptimize` | file | `/proc/sys/kernel/cpu_multi_thread` | /proc/cpu_loading/onoff |
| 6 | `disableProcessReclaim` | file | `/proc/oplus_healthinfo/process_reclaim_enable` | /proc/swaps |
| 7 | `disableTaskCpustats` | file | `/proc/task_info/task_cpustats/task_cpustats_enable` | _(none: graceful default)_ |
| 8 | `disableTaskPlacementDecision` | file | `/proc/task_info/task_sched_info/task_sched_info_enable` | _(none: graceful default)_ |
| 9 | `disableVmallocDebug` | file | `/proc/oplus_healthinfo/vmalloc_debug_enable` | /proc/vmallocinfo |
| 10 | `enableDamonReclaim` | file | `/sys/module/damon_reclaim/parameters/enabled` | _(none: graceful default)_ |
| 11 | `enableKmallocDebug` | file | `/proc/oplus_healthinfo/kmalloc_debug_enable` | /proc/slabinfo |
| 12 | `enableMultiThreadOptimize` | file | `/proc/sys/kernel/cpu_multi_thread` | /proc/cpu_loading/onoff |
| 13 | `enableProcessReclaim` | file | `/proc/oplus_healthinfo/process_reclaim_enable` | /proc/swaps |
| 14 | `enableTaskCpustats` | file | `/proc/task_info/task_cpustats/task_cpustats_enable` | _(none: graceful default)_ |
| 15 | `enableTaskPlacementDecision` | file | `/proc/task_info/task_sched_info/task_sched_info_enable` | _(none: graceful default)_ |
| 16 | `enableVmallocDebug` | file | `/proc/oplus_healthinfo/vmalloc_debug_enable` | /proc/vmallocinfo |
| 17 | `existMemMonitor` | file | `/proc/memory_monitor` | /proc/meminfo |
| 18 | `getDdrResidency` | file | `/sys/power/ddr/residency` | /proc/zoneinfo |
| 19 | `getDevinfoUfsInfo` | file | `/proc/devinfo/ufs` | /proc/ufs_debug |
| 20 | `getDevinfoDDRInfo` | file | `/proc/devinfo/ddr` | /proc/zoneinfo |
| 21 | `getDevinfoUfsVersionInfo` | file | `/proc/devinfo/ufs_version` | /proc/ufs_debug |
| 22 | `getExt4FragScore` | file (arg template) | `/proc/fs/ext4/%s/mb_groups` | /proc/fs/ext4/%s/mb_stats |
| 23 | `getExt4FreefragInfo` | file (arg template) | `/proc/fs/ext4/%s/mb_stats` | /proc/fs/ext4/%s/mb_groups |
| 24 | `getF2fsMovedBlks` | dir | `/sys/fs/f2fs/` | /proc/fs/ext4/ |
| 25 | `getHIAllocWait` | file | `/proc/oplus_healthinfo/alloc_wait` | /proc/pressure/memory |
| 26 | `getHICpuInfo` | file | `/data/oplus/cpu/cpu_loading` | /proc/cpu_loading/curr_cpu_loading |
| 27 | `getHICpuLoading` | file | `/data/oplus/cpu/cpu_loading` | /proc/cpu_loading/curr_cpu_loading |
| 28 | `getHIDState` | file | `/proc/oplus_healthinfo/dstate` | /proc/pressure/cpu |
| 29 | `getHIEmcdrvIowait` | file | `/proc/oplus_healthinfo/emcdrv_iowait` | /proc/pressure/io |
| 30 | `getHIFsyncWait` | file | `/proc/oplus_healthinfo/fsync_wait` | /proc/pressure/io |
| 31 | `getHIIonWait` | file | `/proc/oplus_healthinfo/ion_wait` | /proc/pressure/memory |
| 32 | `getHIIowait` | file | `/proc/oplus_healthinfo/iowait` | /proc/pressure/io |
| 33 | `getHIIowaitHung` | file | `/proc/oplus_healthinfo/iowait_hung` | /proc/pressure/io |
| 34 | `getHIKswapdLoading` | file | `/proc/vmstat` | /proc/vmstat |
| 35 | `getHISchedLatency` | file | `/proc/oplus_healthinfo/sched_latency` | /proc/schedstat |
| 36 | `getHIScmCall` | file | `/proc/oplus_healthinfo/scm_call` | /proc/sched_debug |
| 37 | `getHIUfsFeature` | file | `/proc/oplus_healthinfo/ufs_feature` | /proc/ufs_debug |
| 38 | `getKernelVersion` | file | `/proc/version` | _(none: graceful default)_ |
| 39 | `getKmallocDebug` | file | `/proc/oplus_healthinfo/kmalloc_debug` | /proc/slabinfo |
| 40 | `getKmallocOrigin` | file | `/proc/oplus_healthinfo/kmalloc_origin` | /proc/slabinfo |
| 41 | `getKmallocUsed` | file | `/proc/oplus_healthinfo/kmalloc_used` | /proc/slabinfo |
| 42 | `getMemMonitor` | file | `/proc/oplus_mem/memleak_detect/memory_monitor` | /proc/meminfo |
| 43 | `getOswapVersion` | file | `/proc/version` | _(none: graceful default)_ |
| 44 | `getUfsSignalRecordUpload` | file | `/proc/ufs_signalShow/record_upload` | /proc/ufs_debug |
| 45 | `getUfsplusHpbStatus` | file | `/proc/devinfo/ufsplus_status` | /proc/ufs_debug |
| 46 | `getUfsplusTwStatus` | file | `/proc/devinfo/ufsplus_status` | /proc/ufs_debug |
| 47 | `getVmallocDebug` | file | `/proc/oplus_healthinfo/vmalloc_debug` | /proc/vmallocinfo |
| 48 | `getVmallocHashCal` | file | `/proc/oplus_healthinfo/vmalloc_hash_cal` | /proc/vmallocinfo |
| 49 | `getVmallocUsed` | file | `/proc/oplus_healthinfo/vmalloc_used` | /proc/vmallocinfo |
| 50 | `getallocwait` | file | `/proc/oplus_healthinfo/alloc_wait` | /proc/pressure/memory |
| 51 | `getdstate` | file | `/proc/oplus_healthinfo/dstate` | /proc/pressure/cpu |
| 52 | `getfsyncwait` | file | `/proc/oplus_healthinfo/fsync_wait` | /proc/pressure/io |
| 53 | `getionwait` | file | `/proc/oplus_healthinfo/iowait` | /proc/pressure/io |
| 54 | `getiowait` | file | `/proc/oplus_healthinfo/iowait` | /proc/pressure/io |
| 55 | `getschedlatency` | file | `/proc/oplus_healthinfo/sched_latency` | /proc/schedstat |
| 56 | `hybridswap_memcg_para_read` | file | `/dev/memcg/apps` | /dev/memcg/memory.usage_in_bytes |
| 57 | `hybridswap_memcg_para_write` | file | `/dev/memcg/apps` | _(none: graceful default)_ |
| 58 | `hybridswap_zram_para_read` | file | `/sys/block/zram0/hybridswap_enable` | /sys/block/zram0/mm_stat |
| 59 | `hybridswap_zram_para_write` | file | `/sys/block/zram0/hybridswap_enable` | /sys/block/zram0/mm_stat |
| 60 | `isJankTaskTrackEnable` | file | `/proc/jank_info/cpu_jank_info/task_track_enable` | /proc/trans_scheduler/enable |
| 61 | `perProcessMemReadahead` | file | `/proc/process_reclaim` | /proc/swaps |
| 62 | `perProcessMemReclaim` | file | `/proc/process_reclaim` | /proc/swaps |
| 63 | `readCallStack` | file | `/proc/jank_info/cpu_jank_info/callstack` | /proc/self/stack |
| 64 | `readClmEnable` | file | `/proc/jank_info/cpu_jank_info/clm_enable` | /proc/cpu_loading/onoff |
| 65 | `readClmHighLoadAll` | file | `/proc/jank_info/cpu_jank_info/clm_highload_all` | /proc/cpu_loading/overThrhld |
| 66 | `readClmHighLoadGrp` | file | `/proc/jank_info/cpu_jank_info/clm_highload_grp` | /proc/cpu_loading/specify_overThrhld |
| 67 | `readClmLowLoadGrp` | file | `/proc/jank_info/cpu_jank_info/clm_lowload_grp` | /proc/cpu_loading/underThrhld |
| 68 | `readCpuTaskstats` | file | `/proc/task_cpustats` | /proc/uid_cputime/show_uid_stat |
| 69 | `readDBacktrace` | file | `/proc/task_info/task_sched_info/d_backtrace` | /proc/self/stack |
| 70 | `readDConvert` | file | `/proc/task_info/task_sched_info/d_convert` | /proc/sched_debug |
| 71 | `readFgFreqsThreshold` | file | `/proc/jank_info/cpu_jank_info/fg_freqs_threshold` | /sys/devices/system/cpu/cpufreq/policy0/scaling_min_freq |
| 72 | `readIOBacktrace` | file | `/proc/task_info/task_sched_info/io_backtrace` | /proc/self/stack |
| 73 | `readIomonitorInfo` | file | `/proc/IoMonitor/uid_io_show` | /proc/uid_io/stats |
| 74 | `readJankCpuIndicator` | file | `/proc/jank_info/cpu_jank_info/load_indicator` | /proc/loadavg |
| 75 | `readJankCpuInfo` | file | `/proc/jank_info/cpu_jank_info/cpu_info` | /proc/cpuinfo |
| 76 | `readJankCpuInfoSig` | file | `/proc/jank_info/cpu_jank_info/cpu_info_sig` | /proc/cpuinfo |
| 77 | `readJankCpuLoad` | file | `/proc/jank_info/cpu_jank_info/cpu_load` | /proc/cpu_loading/curr_cpu_loading |
| 78 | `readJankCpuLoad32` | file | `/proc/jank_info/cpu_jank_info/cpu_load32` | /proc/cpu_loading/curr_cpu_loading |
| 79 | `readJankCpuLoad32Scale` | file | `/proc/jank_info/cpu_jank_info/cpu_load32_scale` | /proc/cpu_loading/curr_cpu_loading |
| 80 | `readJankTaskTrack` | file | `/proc/jank_info/cpu_jank_info/task_track` | /proc/trans_scheduler/debug |
| 81 | `readJankTaskTrackByPid` | file | `/proc/jank_info/cpu_jank_info/task_track` | /proc/trans_scheduler/debug |
| 82 | `readJankVersion` | file | `/proc/jank_info/cpu_jank_info/version` | /proc/version |
| 83 | `readKmallocDebugCreate` | file | `/proc/oplus_healthinfo/kmalloc_debug_create` | /proc/slabinfo |
| 84 | `readLimitTable` | file | `/proc/task_info/task_sched_info/limit_table` | /proc/sched_debug |
| 85 | `readMemleakDetectThread` | file | `/proc/oplus_healthinfo/memleak_detect_thread` | /proc/leakage/spower_test |
| 86 | `readMemoryByPids` | file | `/proc/%d/smaps` | _(none: graceful default)_ |
| 87 | `readMemoryByUids` | file | `/dev/memcg/apps` | _(none: graceful default)_ |
| 88 | `readNandswapProc` | file | `/proc/nandswap/fn_enable` | /proc/swaps |
| 89 | `readNormalizeRealTime` | file | `/proc/task_info/task_load_info/normalize_real_time` | /proc/pressure/cpu |
| 90 | `readNormalizeRunningTime` | file | `/proc/task_info/task_load_info/normalize_running_time` | /proc/pressure/cpu |
| 91 | `readOplusReserve3` | file | `/dev/block/by-name/oplusreserve3` | _(none: graceful default)_ |
| 92 | `readOsvelteVersion` | file | `/dev/osvelte` | _(none: graceful default)_ |
| 93 | `readPidsSet` | file | `/proc/task_info/task_sched_info/pids_set` | /proc/uid_procstat/set |
| 94 | `readRealTime` | file | `/proc/task_info/task_load_info/real_time` | /proc/uptime |
| 95 | `readRunningTime` | file | `/proc/task_info/task_load_info/running_time` | /proc/uptime |
| 96 | `readSchedInfoThreshold` | file | `/proc/task_info/task_sched_info/sched_info_threshold` | /proc/sched_debug |
| 97 | `readSgeFreqInfo` | file | `/proc/sgefreqinfo` | /sys/class/devfreq/13000000.mali/cur_freq<br>/sys/kernel/ged/hal/current_freqency<br>/proc/gpufreqv2/gpufreq_status |
| 98 | `readSgeInfo` | file | `/proc/sgeinfo` | /proc/mtk_mali/utilization<br>/sys/kernel/ged/hal/gpu_utilization<br>/proc/mtk_mali/gpu_memory |
| 99 | `readStorageFeature` | file | `/proc/fs/ext4/dm-11/mb_groups` | /sys/fs/f2fs/dm-39 |
| 100 | `readTargetProcess` | file | `/proc/task_info/task_load_info/target_process` | /proc/self/comm |
| 101 | `readTaskCpustatsEnable` | file | `/proc/task_info/task_cpustats/task_cpustats_enable` | _(none: graceful default)_ |
| 102 | `readTaskSchedInfo` | file | `/proc/task_info/task_sched_info/version` | /proc/sched_debug |
| 103 | `readTidsSet` | file | `/proc/task_info/task_sched_info/tids_set` | /proc/uid_procstat/set |
| 104 | `readUxTaskTrack` | file | `/proc/oplus_scheduler/sched_assist/ux_task` | /proc/trans_scheduler/debug |
| 105 | `readVaFeature` | file | `/proc/oplus_healthinfo/va_feature` | /proc/self/maps |
| 106 | `readVersion` | file | `/proc/version` | _(none: graceful default)_ |
| 107 | `removeTaskTrackPid` | file | `/proc/jank_info/cpu_jank_info/task_track` | /proc/trans_scheduler/debug |
| 108 | `setDamonReclaimColdTime` | file | `/sys/module/damon_reclaim/parameters/min_age` | _(none: graceful default)_ |
| 109 | `setDamonReclaimMonitoring` | file | `/sys/module/damon_reclaim/parameters/sample_interval` | _(none: graceful default)_ |
| 110 | `setDamonReclaimQuota` | file | `/sys/module/damon_reclaim/parameters/quota_ms` | _(none: graceful default)_ |
| 111 | `setDamonReclaimWmarks` | file | `/sys/module/damon_reclaim/parameters/wmarks_low` | _(none: graceful default)_ |
| 112 | `setFgUids` | file | `/proc/fg_info/fg_uids` | /proc/uid_procstat/set |
| 113 | `setFrameRate` | file | `/proc/sys/kernel/frame_rate` | /proc/tp_report_rate_switch |
| 114 | `setImFlag` | file | `/proc/oplus_scheduler/sched_assist/im_flag` | /data/vendor/trans_sched/sched_impt_task |
| 115 | `setProcessReclaim` | file | `/proc/process_reclaim` | /proc/swaps |
| 116 | `setSchedAssistImptTask` | file | `/proc/sys/kernel/sched_impt_tgid` | /data/vendor/trans_sched/sched_impt_task |
| 117 | `setSchedAssistScene` | file | `/proc/sys/kernel/sched_assist_scene` | /proc/game_mode |
| 118 | `setSlideboost` | file | `/proc/sys/kernel/slide_boost_enabled` | /proc/game_state |
| 119 | `setTpdID` | file | `/sys/module/tpd/parameters/tpd_id` | _(none: graceful default)_ |
| 120 | `setTpdSerialParams` | file | `/sys/module/tpd/parameters/tpd_cmds` | _(none: graceful default)_ |
| 121 | `writeClmEnable` | file | `/proc/jank_info/cpu_jank_info/clm_enable` | /proc/cpu_loading/onoff |
| 122 | `writeClmHighLoadAll` | file | `/proc/jank_info/cpu_jank_info/clm_highload_all` | /proc/cpu_loading/overThrhld |
| 123 | `writeClmHighLoadGrp` | file | `/proc/jank_info/cpu_jank_info/clm_highload_grp` | /proc/cpu_loading/specify_overThrhld |
| 124 | `writeClmLowLoadGrp` | file | `/proc/jank_info/cpu_jank_info/clm_lowload_grp` | /proc/cpu_loading/underThrhld |
| 125 | `writeFgFreqsThreshold` | file | `/proc/jank_info/cpu_jank_info/fg_freqs_threshold` | /sys/devices/system/cpu/cpufreq/policy0/scaling_min_freq |
| 126 | `writeIOBacktrace` | file | `/proc/task_info/task_sched_info/io_backtrace` | /proc/self/stack |
| 127 | `writeJankTaskTrackEnable` | file | `/proc/jank_info/cpu_jank_info/task_track_enable` | /proc/trans_scheduler/enable |
| 128 | `writeKmallocDebugCreate` | file | `/proc/oplus_healthinfo/kmalloc_debug_create` | /proc/slabinfo |
| 129 | `writeKmallocDebugCreateWithType` | file | `/proc/oplus_healthinfo/kmalloc_debug_create` | /proc/slabinfo |
| 130 | `writeMemMonitor` | file | `/proc/memory_monitor` | /proc/meminfo |
| 131 | `writeMemleakDetectThread` | file | `/proc/oplus_healthinfo/memleak_detect_thread` | /proc/leakage/spower_test |
| 132 | `writeNandswapProc` | file | `/proc/nandswap/fn_enable` | /proc/swaps |
| 133 | `writeOplusReserve3` | file | `/dev/block/by-name/oplusreserve3` | _(none: graceful default)_ |
| 134 | `writePidsSet` | file | `/proc/task_info/task_sched_info/pids_set` | /proc/uid_procstat/set |
| 135 | `writeSchedInfoThreshold` | file | `/proc/task_info/task_sched_info/sched_info_threshold` | /proc/sched_debug |
| 136 | `writeStorageFeature` | file | `/proc/fs/ext4/dm-11/mb_groups` | /sys/fs/f2fs/dm-39 |
| 137 | `writeTaskSchedInfo` | file | `/proc/task_info/task_sched_info/task_sched_info_enable` | /proc/sched_debug |
| 138 | `writeTidsSet` | file | `/proc/task_info/task_sched_info/tids_set` | /proc/uid_procstat/set |
| 139 | `writeUxState` | file | `/proc/oplus_scheduler/sched_assist/ux_task` | _(none: graceful default)_ |
| 140 | `writeVaFeature` | file | `/proc/oplus_healthinfo/va_feature` | /proc/self/maps |
| 141 | `setFreqGoverner` | file | `/sys/devices/system/cpu/cpufreq/policy0/scaling_governor` | _(none: graceful default)_ |
| 142 | `enableAudioPerf` | file | `/proc/oplus_scheduler/sched_assist/audio/enable` | _(none: graceful default)_ |
| 143 | `writeTmemoryMemory` | file | `/sys/tmemory/` | _(none: graceful default)_ |
| 144 | `writeTmemorySwitch` | file | `/sys/tmemory/` | _(none: graceful default)_ |
| 145 | `writeTmemoryFlushBusy` | file | `/sys/tmemory/` | _(none: graceful default)_ |
| 146 | `writeTmemoryFlushIdle` | file | `/sys/tmemory/` | _(none: graceful default)_ |
| 147 | `writeTmemoryCapacity` | file | `/sys/tmemory/` | _(none: graceful default)_ |
| 148 | `writeTmemoryHighWaterRatio` | file | `/sys/tmemory/` | _(none: graceful default)_ |
| 149 | `readTmemorySysdirtypages` | file | `/sys/tmemory/` | _(none: graceful default)_ |
| 150 | `readTmemoryDirtypages` | file | `/sys/tmemory/` | _(none: graceful default)_ |
| 151 | `readTmemoryErrorStat` | file | `/sys/tmemory/` | _(none: graceful default)_ |
| 152 | `readTmemoryIoLatency` | file | `/sys/tmemory/` | _(none: graceful default)_ |
| 153 | `getClmThreshold` | file | `/proc/jank_info/cpu_jank_info/clm_report_threshold` | /proc/cpu_loading/overThrhld |
| 154 | `getClmMuxSwitch` | file | `/data/vendor/trans_sched/cpu/clm_mux_switch` | /proc/cpu_loading/onoff |
| 155 | `setClmThreshold` | file | `/proc/jank_info/cpu_jank_info/clm_report_threshold` | /proc/cpu_loading/overThrhld |
| 156 | `setClmMuxSwitch` | file | `/data/vendor/trans_sched/cpu/clm_mux_switch` | /proc/cpu_loading/onoff |

**156 of 169 methods** are node-mapped; the remaining
13 have no OPlus node and no equivalent, and return a graceful
default (empty string / 0 / false / zeroed struct) — the same thing the shipped
HAL does when its node is absent.

## Measured: OPlus node presence on this kernel

Of the 139 nodes the shipped service references, only **18 exist** here
(9 of them the DAMON reclaim parameters, which are upstream Linux and work
as-is; the rest are OPlus kernel patches with no counterpart):

- `/data/vendor/trans_sched/sched_impt_task`
- `/dev/block/by-name/userdata`
- `/dev/memcg/`
- `/proc/fs/ext4/`
- `/proc/version`
- `/sys/block/zram0/`
- `/sys/fs/f2fs/`
- `/sys/module/damon_reclaim/parameters/aggr_interval`
- `/sys/module/damon_reclaim/parameters/enabled`
- `/sys/module/damon_reclaim/parameters/min_age`
- `/sys/module/damon_reclaim/parameters/quota_ms`
- `/sys/module/damon_reclaim/parameters/quota_reset_interval_ms`
- `/sys/module/damon_reclaim/parameters/quota_sz`
- `/sys/module/damon_reclaim/parameters/sample_interval`
- `/sys/module/damon_reclaim/parameters/wmarks_high`
- `/sys/module/damon_reclaim/parameters/wmarks_low`
- `/sys/module/damon_reclaim/parameters/wmarks_mid`

The raw existence sweep is in `re/kernel_nodes.txt` (the shipped node list);
the device-side sweep output is in `re/kernel_nodes_sweep.txt` (this worktree
is gitignored and has no VCS, so the evidence lives in the tree).

## Caveat: three fallbacks exist but cannot serve a read

The sweep checked existence (`test -e`) **and** readability (open+read, which
is what the HAL actually does). All fallback paths exist; three are not
readable as plain files:

| path | what it actually is | effect |
|---|---|---|
| `/proc/uid_procstat/set` | mode `--w--w--w-`, a **write-only** procfs node (`cat` -> I/O error) | `readPidsSet` / `readTidsSet` return empty; the write methods that use it are correct |
| `/sys/fs/f2fs/dm-39` | a **directory** of f2fs tunables, not a file | harmless: `readStorageFeature`'s first candidate `/proc/fs/ext4/dm-11/mb_groups` exists and answers, so the second is never consulted |
| `/proc/fs/ext4/` | a **directory** of per-device stats (`dm-11`, `dm-12`, ...) | correct usage — `getF2fsMovedBlks` has op `dir` |

## Write safety

Every write fallback was checked for what it actually controls, because on an
OPlus kernel the write went to an absent node (no effect) while here it lands
on a live MTK control node. A fallback is only used for a write when it
controls the SAME feature the OPlus node did; writing a different feature's
control node would change unrelated device behaviour. Read-only nodes are
harmless (the write fails with EINVAL, as it did on the stock HAL).

One fallback was **removed** for safety: `readOplusReserve3` /
`writeOplusReserve3` pointed at `/dev/block/by-name/userdata`, i.e. the
user's data partition. The OPlus reserve partitions do not exist here and the
shipped HAL returned empty for both, so no fallback is the faithful and
safe answer.
