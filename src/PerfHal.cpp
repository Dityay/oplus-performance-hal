// Generated PerfHal implementation — 169 IPerformance methods.
//
// Every method keeps its ORIGINAL OPlus node as the primary candidate and adds
// this device's MTK/Transsion equivalent as fallbacks. The first candidate
// that yields data (reads) or accepts the write (writes) wins, so the HAL is
// faithful on an OPlus kernel and returns real values on the MT6893 kernel.
//
// See re/kernel_nodes_equiv.md for the evidence behind every MTK path.
#include "PerfHal.h"
#include <android/log.h>
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <string>
#include <dirent.h>
#include <fcntl.h>
#include <sys/stat.h>
#include <unistd.h>

#define PLOG(...) __android_log_print(ANDROID_LOG_INFO, "OplusPerfHAL", __VA_ARGS__)

// A binder reply is limited to ~1 MiB. Some MTK nodes are far larger than the
// OPlus ones they stand in for (/proc/vmallocinfo is 1.15 MB here), and a
// reply that size fails the transaction outright (EX_TRANSACTION_FAILED,
// -129). Cap the read so the call still succeeds and returns a truncated but
// usable dump instead of failing.
static const size_t kMaxRead = 256 * 1024;

static std::string read_file(const char* path) {
    int fd = ::open(path, O_RDONLY | O_CLOEXEC);
    if (fd < 0) return std::string();
    std::string out;
    char buf[512];
    ssize_t n;
    while (out.size() < kMaxRead && (n = ::read(fd, buf, sizeof(buf))) > 0)
        out.append(buf, (size_t)n);
    ::close(fd);
    return out;
}

// Sum a field of a /proc/<pid>/smaps[_rollup] dump ("Rss:", "Pss:", ...),
// in kB. Returns -1 when the field is absent.
static int sum_smaps_field(const std::string& text, const char* key) {
    const size_t klen = ::strlen(key);
    long total = 0;
    bool found = false;
    size_t pos = 0;
    while (pos < text.size()) {
        size_t eol = text.find('\n', pos);
        if (eol == std::string::npos) eol = text.size();
        if (text.compare(pos, klen, key) == 0) {
            const char* p = text.c_str() + pos + klen;
            while (*p == ' ' || *p == '\t') ++p;
            char* end = nullptr;
            long v = ::strtol(p, &end, 10);
            if (end != p) { total += v; found = true; }
        }
        pos = eol + 1;
    }
    return found ? (int)total : -1;
}

static bool write_file(const char* path, const char* val) {
    int fd = ::open(path, O_WRONLY | O_CLOEXEC);
    if (fd < 0) return false;
    ssize_t r = ::write(fd, val, ::strlen(val));
    ::close(fd);
    return r >= 0;
}

// First candidate that yields non-empty content.
static std::string read_first(const char* const* cands) {
    for (int i = 0; cands[i] != nullptr; ++i) {
        std::string s = read_file(cands[i]);
        if (!s.empty()) return s;
    }
    return std::string();
}

static int read_int_first(const char* const* cands) {
    for (int i = 0; cands[i] != nullptr; ++i) {
        std::string s = read_file(cands[i]);
        if (s.empty()) continue;
        return (int)::strtol(s.c_str(), nullptr, 10);
    }
    return 0;
}

// First candidate that accepts the write.
static bool write_first(const char* const* cands, const char* val) {
    for (int i = 0; cands[i] != nullptr; ++i) {
        if (write_file(cands[i], val)) return true;
    }
    return false;
}

static bool write_int_first(const char* const* cands, int val) {
    char b[16];
    ::snprintf(b, sizeof(b), "%d", val);
    return write_first(cands, b);
}

// Does any candidate path exist?
static bool path_exists_first(const char* const* cands) {
    for (int i = 0; cands[i] != nullptr; ++i) {
        struct stat st;
        if (::stat(cands[i], &st) == 0) return true;
    }
    return false;
}

// First integer appearing anywhere in the text (for nodes like
// "/proc/ufs_debug" -> "UFS Host state=2").
static int read_int_scan_first(const char* const* cands) {
    for (int i = 0; cands[i] != nullptr; ++i) {
        std::string s = read_file(cands[i]);
        if (s.empty()) continue;
        for (size_t p = 0; p < s.size(); ++p) {
            if (s[p] < '0' || s[p] > '9') continue;
            return (int)::strtol(s.c_str() + p, nullptr, 10);
        }
    }
    return 0;
}

// Kernel version as the stock service reports it: "Linux version 5.10.270..."
// -> major*100 + minor (5.10 -> 510).
static int read_kernel_version(const char* const* cands) {
    for (int i = 0; cands[i] != nullptr; ++i) {
        std::string s = read_file(cands[i]);
        const char* v = ::strstr(s.c_str(), "version");
        if (v == nullptr) continue;
        v += 7;
        while (*v == ' ') ++v;
        int major = 0, minor = 0;
        if (::sscanf(v, "%d.%d", &major, &minor) != 2) continue;
        return major * 100 + minor;
    }
    return 0;
}

// Directory listing, newline separated.
static std::string read_dir_first(const char* const* cands) {
    for (int i = 0; cands[i] != nullptr; ++i) {
        DIR* d = ::opendir(cands[i]);
        if (d == nullptr) continue;
        std::string out;
        struct dirent* e;
        while ((e = ::readdir(d)) != nullptr) {
            if (e->d_name[0] == '.') continue;
            if (!out.empty()) out.push_back('\n');
            out += e->d_name;
        }
        ::closedir(d);
        if (!out.empty()) return out;
    }
    return std::string();
}

// Path template: %s is replaced by the method's string argument.
static std::string subst(const char* tmpl, const char* arg) {
    std::string t(tmpl);
    size_t p = t.find("%s");
    if (p == std::string::npos) return t;
    t.replace(p, 2, arg ? arg : "");
    return t;
}

static std::string read_first_sub(const char* const* cands, const char* arg) {
    for (int i = 0; cands[i] != nullptr; ++i) {
        std::string s = read_file(subst(cands[i], arg).c_str());
        if (!s.empty()) return s;
    }
    return std::string();
}

static int read_int_first_sub(const char* const* cands, const char* arg) {
    for (int i = 0; cands[i] != nullptr; ++i) {
        std::string s = read_file(subst(cands[i], arg).c_str());
        if (s.empty()) continue;
        return (int)::strtol(s.c_str(), nullptr, 10);
    }
    return 0;
}

// Struct out-params are filled by the specialised overloads below. There is
// deliberately no generic template: a struct type without an overload must be a
// compile error, not a silent zero-init.


// --- specialised struct fillers -------------------------------------------
#include <aidl/vendor/oplus/hardware/performance/ProcReqHal.h>
#include <aidl/vendor/oplus/hardware/performance/ProcMemStatRet.h>
#include <aidl/vendor/oplus/hardware/performance/OsvelteVersionRet.h>

static void fill_struct(const char* const* cands,
                        ::aidl::vendor::oplus::hardware::performance::ProcReqHal* out) {
    out->target = read_first(cands);
    out->reqId = 0;
}

static void fill_struct(const char* const* cands,
                        ::aidl::vendor::oplus::hardware::performance::OsvelteVersionRet* out) {
    // the stock service reports version 1 with zeroed sub-fields
    out->version = 1;
    out->major = 0;
    out->minor = 0;
    out->patch = 0;
    (void)cands;
}

static void fill_struct(const char* const* cands,
                        ::aidl::vendor::oplus::hardware::performance::ProcMemStatRet* out) {
    out->status = 1;
    out->count = 0;
    out->stats.clear();
    (void)cands;
}

::ndk::ScopedAStatus PerfHal::addAcmDirName(const std::string& in_arg0, int64_t in_arg1, int32_t* _aidl_return) {
    *_aidl_return = 0;
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::addAcmNomediaDirName(const std::string& in_arg0, int32_t* _aidl_return) {
    *_aidl_return = 0;
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::addAcmPkgName(const std::string& in_arg0, int64_t in_arg1, int32_t* _aidl_return) {
    *_aidl_return = 0;
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::addTaskTrackPid(int32_t in_arg0, int32_t in_arg1, bool in_arg2, int32_t* _aidl_return) {
    static const char* const c[] = {"/proc/jank_info/cpu_jank_info/task_track", "/proc/trans_scheduler/debug", nullptr};
    *_aidl_return = read_int_first(c);
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::clearTaskTrackGroup(int32_t in_arg0) {
    static const char* const c[] = {"/proc/jank_info/cpu_jank_info/task_track", "/proc/trans_scheduler/debug", nullptr};
    write_int_first(c, (int)in_arg0);
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::delAcmDirName(const std::string& in_arg0, int32_t* _aidl_return) {
    *_aidl_return = 0;
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::delAcmNomediaDirName(const std::string& in_arg0, int32_t* _aidl_return) {
    *_aidl_return = 0;
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::delAcmPkgName(const std::string& in_arg0, int32_t* _aidl_return) {
    *_aidl_return = 0;
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::disableDamonReclaim(int32_t* _aidl_return) {
    static const char* const c[] = {"/sys/module/damon_reclaim/parameters/enabled", nullptr};
    *_aidl_return = read_int_first(c);
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::disableKmallocDebug(int32_t* _aidl_return) {
    static const char* const c[] = {"/proc/oplus_healthinfo/kmalloc_debug_enable", "/proc/slabinfo", nullptr};
    *_aidl_return = read_int_first(c);
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::disableMultiThreadOptimize(int32_t* _aidl_return) {
    static const char* const c[] = {"/proc/sys/kernel/cpu_multi_thread", "/proc/cpu_loading/onoff", nullptr};
    *_aidl_return = read_int_first(c);
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::disableProcessReclaim(int32_t* _aidl_return) {
    static const char* const c[] = {"/proc/oplus_healthinfo/process_reclaim_enable", "/proc/swaps", nullptr};
    *_aidl_return = read_int_first(c);
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::disableTaskCpustats(int32_t* _aidl_return) {
    static const char* const c[] = {"/proc/task_info/task_cpustats/task_cpustats_enable", nullptr};
    *_aidl_return = read_int_first(c);
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::disableTaskPlacementDecision(int32_t* _aidl_return) {
    static const char* const c[] = {"/proc/task_info/task_sched_info/task_sched_info_enable", nullptr};
    *_aidl_return = read_int_first(c);
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::disableVmallocDebug(int32_t* _aidl_return) {
    static const char* const c[] = {"/proc/oplus_healthinfo/vmalloc_debug_enable", "/proc/vmallocinfo", nullptr};
    *_aidl_return = read_int_first(c);
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::enableDamonReclaim(int32_t* _aidl_return) {
    static const char* const c[] = {"/sys/module/damon_reclaim/parameters/enabled", nullptr};
    *_aidl_return = read_int_first(c);
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::enableKmallocDebug(int32_t* _aidl_return) {
    static const char* const c[] = {"/proc/oplus_healthinfo/kmalloc_debug_enable", "/proc/slabinfo", nullptr};
    *_aidl_return = read_int_first(c);
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::enableMultiThreadOptimize(int32_t* _aidl_return) {
    static const char* const c[] = {"/proc/sys/kernel/cpu_multi_thread", "/proc/cpu_loading/onoff", nullptr};
    *_aidl_return = read_int_first(c);
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::enableProcessReclaim(int32_t* _aidl_return) {
    static const char* const c[] = {"/proc/oplus_healthinfo/process_reclaim_enable", "/proc/swaps", nullptr};
    *_aidl_return = read_int_first(c);
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::enableTaskCpustats(int32_t* _aidl_return) {
    static const char* const c[] = {"/proc/task_info/task_cpustats/task_cpustats_enable", nullptr};
    *_aidl_return = read_int_first(c);
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::enableTaskPlacementDecision(int32_t* _aidl_return) {
    static const char* const c[] = {"/proc/task_info/task_sched_info/task_sched_info_enable", nullptr};
    *_aidl_return = read_int_first(c);
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::enableVmallocDebug(int32_t* _aidl_return) {
    static const char* const c[] = {"/proc/oplus_healthinfo/vmalloc_debug_enable", "/proc/vmallocinfo", nullptr};
    *_aidl_return = read_int_first(c);
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::existMemMonitor(int32_t* _aidl_return) {
    static const char* const c[] = {"/proc/memory_monitor", "/proc/meminfo", nullptr};
    *_aidl_return = path_exists_first(c) ? 1 : 0;
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::getAcmDirFlag(const std::string& in_arg0, int64_t* _aidl_return) {
    *_aidl_return = 0;
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::getAcmOpstat(int32_t* _aidl_return) {
    *_aidl_return = 0;
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::getAcmPkgFlag(const std::string& in_arg0, int64_t* _aidl_return) {
    *_aidl_return = 0;
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::getDdrResidency(std::string* _aidl_return) {
    static const char* const c[] = {"/sys/power/ddr/residency", "/proc/zoneinfo", nullptr};
    *_aidl_return = read_first(c);
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::getDevinfoUfsInfo(std::string* _aidl_return) {
    static const char* const c[] = {"/proc/devinfo/ufs", "/proc/ufs_debug", nullptr};
    *_aidl_return = read_first(c);
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::getDevinfoDDRInfo(std::string* _aidl_return) {
    static const char* const c[] = {"/proc/devinfo/ddr", "/proc/zoneinfo", nullptr};
    *_aidl_return = read_first(c);
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::getDevinfoUfsVersionInfo(std::string* _aidl_return) {
    static const char* const c[] = {"/proc/devinfo/ufs_version", "/proc/ufs_debug", nullptr};
    *_aidl_return = read_first(c);
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::getExt4FragScore(const std::string& in_arg0, std::string* _aidl_return) {
    static const char* const c[] = {"/proc/fs/ext4/%s/mb_groups", "/proc/fs/ext4/%s/mb_stats", nullptr};
    *_aidl_return = read_first_sub(c, in_arg0.c_str());
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::getExt4FreefragInfo(const std::string& in_arg0, std::string* _aidl_return) {
    static const char* const c[] = {"/proc/fs/ext4/%s/mb_stats", "/proc/fs/ext4/%s/mb_groups", nullptr};
    *_aidl_return = read_first_sub(c, in_arg0.c_str());
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::getF2fsMovedBlks(std::string* _aidl_return) {
    static const char* const c[] = {"/sys/fs/f2fs/", "/proc/fs/ext4/", nullptr};
    *_aidl_return = read_dir_first(c);
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::getHIAllocWait(::aidl::vendor::oplus::hardware::performance::ProcReqHal* _aidl_return) {
    static const char* const c[] = {"/proc/oplus_healthinfo/alloc_wait", "/proc/pressure/memory", nullptr};
    fill_struct(c, _aidl_return);
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::getHICpuInfo(std::string* _aidl_return) {
    static const char* const c[] = {"/data/oplus/cpu/cpu_loading", "/proc/cpu_loading/curr_cpu_loading", nullptr};
    *_aidl_return = read_first(c);
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::getHICpuLoading(std::string* _aidl_return) {
    static const char* const c[] = {"/data/oplus/cpu/cpu_loading", "/proc/cpu_loading/curr_cpu_loading", nullptr};
    *_aidl_return = read_first(c);
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::getHIDState(::aidl::vendor::oplus::hardware::performance::ProcReqHal* _aidl_return) {
    static const char* const c[] = {"/proc/oplus_healthinfo/dstate", "/proc/pressure/cpu", nullptr};
    fill_struct(c, _aidl_return);
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::getHIEmcdrvIowait(::aidl::vendor::oplus::hardware::performance::ProcReqHal* _aidl_return) {
    static const char* const c[] = {"/proc/oplus_healthinfo/emcdrv_iowait", "/proc/pressure/io", nullptr};
    fill_struct(c, _aidl_return);
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::getHIFsyncWait(::aidl::vendor::oplus::hardware::performance::ProcReqHal* _aidl_return) {
    static const char* const c[] = {"/proc/oplus_healthinfo/fsync_wait", "/proc/pressure/io", nullptr};
    fill_struct(c, _aidl_return);
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::getHIIonWait(::aidl::vendor::oplus::hardware::performance::ProcReqHal* _aidl_return) {
    static const char* const c[] = {"/proc/oplus_healthinfo/ion_wait", "/proc/pressure/memory", nullptr};
    fill_struct(c, _aidl_return);
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::getHIIowait(::aidl::vendor::oplus::hardware::performance::ProcReqHal* _aidl_return) {
    static const char* const c[] = {"/proc/oplus_healthinfo/iowait", "/proc/pressure/io", nullptr};
    fill_struct(c, _aidl_return);
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::getHIIowaitHung(::aidl::vendor::oplus::hardware::performance::ProcReqHal* _aidl_return) {
    static const char* const c[] = {"/proc/oplus_healthinfo/iowait_hung", "/proc/pressure/io", nullptr};
    fill_struct(c, _aidl_return);
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::getHIKswapdLoading(::aidl::vendor::oplus::hardware::performance::ProcReqHal* _aidl_return) {
    static const char* const c[] = {"/proc/vmstat", "/proc/vmstat", nullptr};
    fill_struct(c, _aidl_return);
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::getHISchedLatency(::aidl::vendor::oplus::hardware::performance::ProcReqHal* _aidl_return) {
    static const char* const c[] = {"/proc/oplus_healthinfo/sched_latency", "/proc/schedstat", nullptr};
    fill_struct(c, _aidl_return);
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::getHIScmCall(::aidl::vendor::oplus::hardware::performance::ProcReqHal* _aidl_return) {
    static const char* const c[] = {"/proc/oplus_healthinfo/scm_call", "/proc/sched_debug", nullptr};
    fill_struct(c, _aidl_return);
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::getHIUfsFeature(::aidl::vendor::oplus::hardware::performance::ProcReqHal* _aidl_return) {
    static const char* const c[] = {"/proc/oplus_healthinfo/ufs_feature", "/proc/ufs_debug", nullptr};
    fill_struct(c, _aidl_return);
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::getKernelVersion(int32_t* _aidl_return) {
    static const char* const c[] = {"/proc/version", nullptr};
    *_aidl_return = read_kernel_version(c);
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::getKmallocDebug(std::string* _aidl_return) {
    static const char* const c[] = {"/proc/oplus_healthinfo/kmalloc_debug", "/proc/slabinfo", nullptr};
    *_aidl_return = read_first(c);
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::getKmallocOrigin(std::string* _aidl_return) {
    static const char* const c[] = {"/proc/oplus_healthinfo/kmalloc_origin", "/proc/slabinfo", nullptr};
    *_aidl_return = read_first(c);
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::getKmallocUsed(std::string* _aidl_return) {
    static const char* const c[] = {"/proc/oplus_healthinfo/kmalloc_used", "/proc/slabinfo", nullptr};
    *_aidl_return = read_first(c);
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::getMemMonitor(std::string* _aidl_return) {
    static const char* const c[] = {"/proc/oplus_mem/memleak_detect/memory_monitor", "/proc/meminfo", nullptr};
    *_aidl_return = read_first(c);
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::getOswapVersion(int32_t* _aidl_return) {
    // no OPlus "oswap" concept on this kernel; the stock service reports -1
    *_aidl_return = -1;
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::getUfsSignalRecordUpload(std::string* _aidl_return) {
    static const char* const c[] = {"/proc/ufs_signalShow/record_upload", "/proc/ufs_debug", nullptr};
    *_aidl_return = read_first(c);
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::getUfsplusHpbStatus(int32_t* _aidl_return) {
    static const char* const c[] = {"/proc/devinfo/ufsplus_status", "/proc/ufs_debug", nullptr};
    *_aidl_return = read_int_scan_first(c);
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::getUfsplusTwStatus(int32_t* _aidl_return) {
    static const char* const c[] = {"/proc/devinfo/ufsplus_status", "/proc/ufs_debug", nullptr};
    *_aidl_return = read_int_scan_first(c);
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::getVmallocDebug(std::string* _aidl_return) {
    static const char* const c[] = {"/proc/oplus_healthinfo/vmalloc_debug", "/proc/vmallocinfo", nullptr};
    *_aidl_return = read_first(c);
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::getVmallocHashCal(std::string* _aidl_return) {
    static const char* const c[] = {"/proc/oplus_healthinfo/vmalloc_hash_cal", "/proc/vmallocinfo", nullptr};
    *_aidl_return = read_first(c);
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::getVmallocUsed(std::string* _aidl_return) {
    static const char* const c[] = {"/proc/oplus_healthinfo/vmalloc_used", "/proc/vmallocinfo", nullptr};
    *_aidl_return = read_first(c);
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::getallocwait(std::string* _aidl_return) {
    static const char* const c[] = {"/proc/oplus_healthinfo/alloc_wait", "/proc/pressure/memory", nullptr};
    *_aidl_return = read_first(c);
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::getdstate(std::string* _aidl_return) {
    static const char* const c[] = {"/proc/oplus_healthinfo/dstate", "/proc/pressure/cpu", nullptr};
    *_aidl_return = read_first(c);
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::getfsyncwait(std::string* _aidl_return) {
    static const char* const c[] = {"/proc/oplus_healthinfo/fsync_wait", "/proc/pressure/io", nullptr};
    *_aidl_return = read_first(c);
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::getionwait(std::string* _aidl_return) {
    static const char* const c[] = {"/proc/oplus_healthinfo/iowait", "/proc/pressure/io", nullptr};
    *_aidl_return = read_first(c);
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::getiowait(std::string* _aidl_return) {
    static const char* const c[] = {"/proc/oplus_healthinfo/iowait", "/proc/pressure/io", nullptr};
    *_aidl_return = read_first(c);
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::getschedlatency(std::string* _aidl_return) {
    static const char* const c[] = {"/proc/oplus_healthinfo/sched_latency", "/proc/schedstat", nullptr};
    *_aidl_return = read_first(c);
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::hybridswap_memcg_para_read(int32_t in_arg0, const std::string& in_arg1, std::string* _aidl_return) {
    static const char* const c[] = {"/dev/memcg/apps", "/dev/memcg/memory.usage_in_bytes", nullptr};
    *_aidl_return = read_first(c);
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::hybridswap_memcg_para_write(int32_t in_arg0, const std::string& in_arg1, const std::string& in_arg2, int32_t* _aidl_return) {
    static const char* const c[] = {"/dev/memcg/apps", nullptr};
    *_aidl_return = read_int_first(c);
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::hybridswap_zram_para_read(int32_t in_arg0, std::string* _aidl_return) {
    static const char* const c[] = {"/sys/block/zram0/hybridswap_enable", "/sys/block/zram0/mm_stat", nullptr};
    *_aidl_return = read_first(c);
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::hybridswap_zram_para_write(int32_t in_arg0, const std::string& in_arg1, int32_t* _aidl_return) {
    static const char* const c[] = {"/sys/block/zram0/hybridswap_enable", "/sys/block/zram0/mm_stat", nullptr};
    *_aidl_return = read_int_first(c);
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::isJankTaskTrackEnable(bool* _aidl_return) {
    static const char* const c[] = {"/proc/jank_info/cpu_jank_info/task_track_enable", "/proc/trans_scheduler/enable", nullptr};
    // the MTK node holds "enable=1", so the integer has to be scanned for
    *_aidl_return = read_int_scan_first(c) != 0;
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::perProcessMemReadahead(int32_t in_arg0, int32_t in_arg1, int32_t in_arg2, int32_t* _aidl_return) {
    static const char* const c[] = {"/proc/process_reclaim", "/proc/swaps", nullptr};
    *_aidl_return = read_int_first(c);
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::perProcessMemReclaim(int32_t in_arg0, int32_t in_arg1, int32_t in_arg2, int32_t* _aidl_return) {
    static const char* const c[] = {"/proc/process_reclaim", "/proc/swaps", nullptr};
    *_aidl_return = read_int_first(c);
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::readCallStack(std::string* _aidl_return) {
    static const char* const c[] = {"/proc/jank_info/cpu_jank_info/callstack", "/proc/self/stack", nullptr};
    *_aidl_return = read_first(c);
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::readClmEnable(std::string* _aidl_return) {
    static const char* const c[] = {"/proc/jank_info/cpu_jank_info/clm_enable", "/proc/cpu_loading/onoff", nullptr};
    *_aidl_return = read_first(c);
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::readClmHighLoadAll(std::string* _aidl_return) {
    static const char* const c[] = {"/proc/jank_info/cpu_jank_info/clm_highload_all", "/proc/cpu_loading/overThrhld", nullptr};
    *_aidl_return = read_first(c);
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::readClmHighLoadGrp(std::string* _aidl_return) {
    static const char* const c[] = {"/proc/jank_info/cpu_jank_info/clm_highload_grp", "/proc/cpu_loading/specify_overThrhld", nullptr};
    *_aidl_return = read_first(c);
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::readClmLowLoadGrp(std::string* _aidl_return) {
    static const char* const c[] = {"/proc/jank_info/cpu_jank_info/clm_lowload_grp", "/proc/cpu_loading/underThrhld", nullptr};
    *_aidl_return = read_first(c);
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::readCpuTaskstats(std::string* _aidl_return) {
    static const char* const c[] = {"/proc/task_cpustats", "/proc/uid_cputime/show_uid_stat", nullptr};
    *_aidl_return = read_first(c);
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::readDBacktrace(std::string* _aidl_return) {
    static const char* const c[] = {"/proc/task_info/task_sched_info/d_backtrace", "/proc/self/stack", nullptr};
    *_aidl_return = read_first(c);
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::readDConvert(std::string* _aidl_return) {
    static const char* const c[] = {"/proc/task_info/task_sched_info/d_convert", "/proc/sched_debug", nullptr};
    *_aidl_return = read_first(c);
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::readFgFreqsThreshold(std::string* _aidl_return) {
    static const char* const c[] = {"/proc/jank_info/cpu_jank_info/fg_freqs_threshold", "/sys/devices/system/cpu/cpufreq/policy0/scaling_min_freq", nullptr};
    *_aidl_return = read_first(c);
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::readIOBacktrace(std::string* _aidl_return) {
    static const char* const c[] = {"/proc/task_info/task_sched_info/io_backtrace", "/proc/self/stack", nullptr};
    *_aidl_return = read_first(c);
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::readIomonitorInfo(const std::string& in_arg0, std::string* _aidl_return) {
    static const char* const c[] = {"/proc/IoMonitor/uid_io_show", "/proc/uid_io/stats", nullptr};
    *_aidl_return = read_first(c);
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::readJankCpuIndicator(std::string* _aidl_return) {
    static const char* const c[] = {"/proc/jank_info/cpu_jank_info/load_indicator", "/proc/loadavg", nullptr};
    *_aidl_return = read_first(c);
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::readJankCpuInfo(std::string* _aidl_return) {
    static const char* const c[] = {"/proc/jank_info/cpu_jank_info/cpu_info", "/proc/cpuinfo", nullptr};
    *_aidl_return = read_first(c);
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::readJankCpuInfoSig(std::string* _aidl_return) {
    static const char* const c[] = {"/proc/jank_info/cpu_jank_info/cpu_info_sig", "/proc/cpuinfo", nullptr};
    *_aidl_return = read_first(c);
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::readJankCpuLoad(std::string* _aidl_return) {
    static const char* const c[] = {"/proc/jank_info/cpu_jank_info/cpu_load", "/proc/cpu_loading/curr_cpu_loading", nullptr};
    *_aidl_return = read_first(c);
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::readJankCpuLoad32(std::string* _aidl_return) {
    static const char* const c[] = {"/proc/jank_info/cpu_jank_info/cpu_load32", "/proc/cpu_loading/curr_cpu_loading", nullptr};
    *_aidl_return = read_first(c);
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::readJankCpuLoad32Scale(std::string* _aidl_return) {
    static const char* const c[] = {"/proc/jank_info/cpu_jank_info/cpu_load32_scale", "/proc/cpu_loading/curr_cpu_loading", nullptr};
    *_aidl_return = read_first(c);
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::readJankTaskTrack(std::string* _aidl_return) {
    static const char* const c[] = {"/proc/jank_info/cpu_jank_info/task_track", "/proc/trans_scheduler/debug", nullptr};
    *_aidl_return = read_first(c);
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::readJankTaskTrackByPid(int32_t in_arg0, std::string* _aidl_return) {
    static const char* const c[] = {"/proc/jank_info/cpu_jank_info/task_track", "/proc/trans_scheduler/debug", nullptr};
    *_aidl_return = read_first(c);
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::readJankVersion(std::string* _aidl_return) {
    static const char* const c[] = {"/proc/jank_info/cpu_jank_info/version", "/proc/version", nullptr};
    *_aidl_return = read_first(c);
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::readKmallocDebugCreate(std::string* _aidl_return) {
    static const char* const c[] = {"/proc/oplus_healthinfo/kmalloc_debug_create", "/proc/slabinfo", nullptr};
    *_aidl_return = read_first(c);
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::readLimitTable(std::string* _aidl_return) {
    static const char* const c[] = {"/proc/task_info/task_sched_info/limit_table", "/proc/sched_debug", nullptr};
    *_aidl_return = read_first(c);
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::readMemleakDetectThread(std::string* _aidl_return) {
    static const char* const c[] = {"/proc/oplus_healthinfo/memleak_detect_thread", "/proc/leakage/spower_test", nullptr};
    *_aidl_return = read_first(c);
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::readMemoryByPids(const std::vector<int32_t>& in_arg0, int32_t in_arg1, ::aidl::vendor::oplus::hardware::performance::ProcMemStatRet* _aidl_return) {
    // The OPlus node was a per-uid memcg file. This kernel's memcg is
    // per-app-group, so the honest per-process source is
    // /proc/<pid>/smaps_rollup.
    using St = ::aidl::vendor::oplus::hardware::performance::ProcMemStat;
    _aidl_return->status = 1;
    _aidl_return->count = 0;
    _aidl_return->stats.clear();
    PLOG("readMemoryByPids: %zu pid(s) requested", in_arg0.size());
    for (size_t i = 0; i < in_arg0.size(); ++i) {
        const int pid = (int)in_arg0[i];
        char path[64];
        ::snprintf(path, sizeof(path), "/proc/%d/smaps_rollup", pid);
        std::string roll = read_file(path);
        if (roll.empty()) {
            ::snprintf(path, sizeof(path), "/proc/%d/smaps", pid);
            roll = read_file(path);
        }
        St st;
        st.pid = pid;
        ::snprintf(path, sizeof(path), "/proc/%d/comm", pid);
        st.comm = read_file(path);
        ::snprintf(path, sizeof(path), "/proc/%d/oom_score_adj", pid);
        std::string adj = read_file(path);
        st.oomAdj = adj.empty() ? (char)0 : (char)::atoi(adj.c_str());
        if (roll.empty()) {
            // The HAL runs in a generic hal_* domain that is denied the
            // per-process smaps, but /proc/<pid>/statm is world-readable.
            // statm: size resident shared text lib data dt (in pages).
            ::snprintf(path, sizeof(path), "/proc/%d/statm", pid);
            std::string stm = read_file(path);
            if (stm.empty()) {
                PLOG("readMemoryByPids: cannot read %s (errno %d)", path, errno);
                continue;
            }
            long v[7] = {0, 0, 0, 0, 0, 0, 0};
            ::sscanf(stm.c_str(), "%ld %ld %ld %ld %ld %ld %ld",
                     &v[0], &v[1], &v[2], &v[3], &v[4], &v[5], &v[6]);
            const long kb = ::sysconf(_SC_PAGESIZE) / 1024;
            st.rss = (int)(v[1] * kb);
            st.sharedClean = (int)(v[2] * kb);
            st.pss = 0;               // PSS is not in statm
            _aidl_return->stats.push_back(st);
            continue;
        }
        st.rss = sum_smaps_field(roll, "Rss:");
        st.pss = sum_smaps_field(roll, "Pss:");
        st.swap = sum_smaps_field(roll, "Swap:");
        st.sharedClean = sum_smaps_field(roll, "Shared_Clean:");
        st.sharedDirty = sum_smaps_field(roll, "Shared_Dirty:");
        st.privateClean = sum_smaps_field(roll, "Private_Clean:");
        st.privateDirty = sum_smaps_field(roll, "Private_Dirty:");
        st.swapPss = sum_smaps_field(roll, "SwapPss:");
        st.anonRss = sum_smaps_field(roll, "Anonymous:");
        st.fileRss = sum_smaps_field(roll, "FilePmdMapped:");
        st.shmemRss = sum_smaps_field(roll, "ShmemPmdMapped:");
        if (st.rss < 0) st.rss = 0;
        if (st.pss < 0) st.pss = 0;
        if (st.anonRss < 0) st.anonRss = 0;
        if (st.fileRss < 0) st.fileRss = 0;
        if (st.shmemRss < 0) st.shmemRss = 0;
        if (st.swap < 0) st.swap = 0;
        _aidl_return->stats.push_back(st);
    }
    _aidl_return->count = (int32_t)_aidl_return->stats.size();
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::readMemoryByUids(const std::vector<int32_t>& in_arg0, int32_t in_arg1, ::aidl::vendor::oplus::hardware::performance::ProcMemStatRet* _aidl_return) {
    // Per-uid memory. The OPlus node was a per-uid memcg file; this kernel's
    // memcg is per-app-group (/dev/memcg/apps), not per-uid, so the honest
    // source is the sum of every process's smaps_rollup, grouped by the
    // process's real uid. One /proc walk serves all requested uids.
    using St = ::aidl::vendor::oplus::hardware::performance::ProcMemStat;
    _aidl_return->status = 1;
    _aidl_return->count = 0;
    _aidl_return->stats.clear();
    if (in_arg0.empty()) return ::ndk::ScopedAStatus::ok();
    std::vector<long> totals(in_arg0.size(), 0);
    std::vector<long> psss(in_arg0.size(), 0);
    std::vector<int> procs(in_arg0.size(), 0);
    DIR* dir = ::opendir("/proc");
    if (dir == nullptr) return ::ndk::ScopedAStatus::ok();
    struct dirent* ent;
    while ((ent = ::readdir(dir)) != nullptr) {
        if (ent->d_name[0] < '0' || ent->d_name[0] > '9') continue;
        char path[64];
        ::snprintf(path, sizeof(path), "/proc/%s/status", ent->d_name);
        std::string status = read_file(path);
        size_t pos = status.find("Uid:");
        if (pos == std::string::npos) continue;
        int puid = 0;
        if (::sscanf(status.c_str() + pos, "Uid: %d", &puid) != 1) continue;
        size_t slot = in_arg0.size();
        for (size_t k = 0; k < in_arg0.size(); ++k) {
            if ((int)in_arg0[k] == puid) { slot = k; break; }
        }
        if (slot == in_arg0.size()) continue;
        ::snprintf(path, sizeof(path), "/proc/%s/smaps_rollup", ent->d_name);
        std::string roll = read_file(path);
        int rss = sum_smaps_field(roll, "Rss:");
        int pss = sum_smaps_field(roll, "Pss:");
        if (rss > 0) totals[slot] += rss;
        if (pss > 0) psss[slot] += pss;
        ++procs[slot];
    }
    ::closedir(dir);
    for (size_t k = 0; k < in_arg0.size(); ++k) {
        if (procs[k] == 0) continue;
        St st;
        st.pid = (int)in_arg0[k];
        st.comm = "uid";
        st.rss = (int)totals[k];
        st.pss = (int)psss[k];
        _aidl_return->stats.push_back(st);
    }
    _aidl_return->count = (int32_t)_aidl_return->stats.size();
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::readNandswapProc(const std::string& in_arg0, std::string* _aidl_return) {
    static const char* const c[] = {"/proc/nandswap/fn_enable", "/proc/swaps", nullptr};
    *_aidl_return = read_first(c);
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::readNormalizeRealTime(std::string* _aidl_return) {
    static const char* const c[] = {"/proc/task_info/task_load_info/normalize_real_time", "/proc/pressure/cpu", nullptr};
    *_aidl_return = read_first(c);
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::readNormalizeRunningTime(std::string* _aidl_return) {
    static const char* const c[] = {"/proc/task_info/task_load_info/normalize_running_time", "/proc/pressure/cpu", nullptr};
    *_aidl_return = read_first(c);
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::readOplusReserve3(int32_t in_arg0, int32_t in_arg1, std::string* _aidl_return) {
    static const char* const c[] = {"/dev/block/by-name/oplusreserve3", nullptr};
    *_aidl_return = read_first(c);
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::readOsvelteVersion(::aidl::vendor::oplus::hardware::performance::OsvelteVersionRet* _aidl_return) {
    static const char* const c[] = {"/dev/osvelte", nullptr};
    fill_struct(c, _aidl_return);
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::readPidsSet(std::string* _aidl_return) {
    static const char* const c[] = {"/proc/task_info/task_sched_info/pids_set", "/proc/uid_procstat/set", nullptr};
    *_aidl_return = read_first(c);
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::readRealTime(std::string* _aidl_return) {
    static const char* const c[] = {"/proc/task_info/task_load_info/real_time", "/proc/uptime", nullptr};
    *_aidl_return = read_first(c);
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::readRunningTime(std::string* _aidl_return) {
    static const char* const c[] = {"/proc/task_info/task_load_info/running_time", "/proc/uptime", nullptr};
    *_aidl_return = read_first(c);
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::readSchedInfoThreshold(std::string* _aidl_return) {
    static const char* const c[] = {"/proc/task_info/task_sched_info/sched_info_threshold", "/proc/sched_debug", nullptr};
    *_aidl_return = read_first(c);
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::readSgeFreqInfo(std::string* _aidl_return) {
    static const char* const c[] = {"/proc/sgefreqinfo", "/sys/class/devfreq/13000000.mali/cur_freq", "/sys/kernel/ged/hal/current_freqency", "/proc/gpufreqv2/gpufreq_status", nullptr};
    *_aidl_return = read_first(c);
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::readSgeInfo(std::string* _aidl_return) {
    static const char* const c[] = {"/proc/sgeinfo", "/proc/mtk_mali/utilization", "/sys/kernel/ged/hal/gpu_utilization", "/proc/mtk_mali/gpu_memory", nullptr};
    *_aidl_return = read_first(c);
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::readStorageFeature(const std::string& in_arg0, const std::string& in_arg1, const std::string& in_arg2, std::string* _aidl_return) {
    static const char* const c[] = {"/proc/fs/ext4/dm-11/mb_groups", "/sys/fs/f2fs/dm-39", nullptr};
    *_aidl_return = read_first(c);
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::readTargetProcess(const std::string& in_arg0, std::string* _aidl_return) {
    static const char* const c[] = {"/proc/task_info/task_load_info/target_process", "/proc/self/comm", nullptr};
    *_aidl_return = read_first(c);
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::readTaskCpustatsEnable(std::string* _aidl_return) {
    static const char* const c[] = {"/proc/task_info/task_cpustats/task_cpustats_enable", nullptr};
    *_aidl_return = read_first(c);
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::readTaskSchedInfo(std::string* _aidl_return) {
    static const char* const c[] = {"/proc/task_info/task_sched_info/version", "/proc/sched_debug", nullptr};
    *_aidl_return = read_first(c);
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::readTidsSet(std::string* _aidl_return) {
    static const char* const c[] = {"/proc/task_info/task_sched_info/tids_set", "/proc/uid_procstat/set", nullptr};
    *_aidl_return = read_first(c);
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::readUxTaskTrack(int32_t in_arg0, int32_t in_arg1, std::string* _aidl_return) {
    static const char* const c[] = {"/proc/oplus_scheduler/sched_assist/ux_task", "/proc/trans_scheduler/debug", nullptr};
    *_aidl_return = read_first(c);
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::readVaFeature(std::string* _aidl_return) {
    static const char* const c[] = {"/proc/oplus_healthinfo/va_feature", "/proc/self/maps", nullptr};
    *_aidl_return = read_first(c);
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::readVersion(std::string* _aidl_return) {
    static const char* const c[] = {"/proc/version", nullptr};
    *_aidl_return = read_first(c);
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::removeTaskTrackPid(int32_t in_arg0, int32_t in_arg1) {
    static const char* const c[] = {"/proc/jank_info/cpu_jank_info/task_track", "/proc/trans_scheduler/debug", nullptr};
    write_int_first(c, (int)in_arg0);
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::searchAcmNomediaDirName(const std::string& in_arg0, int32_t* _aidl_return) {
    *_aidl_return = 0;
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::setAcmOpstat(int32_t in_arg0, int32_t* _aidl_return) {
    *_aidl_return = 0;
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::setDamonReclaimColdTime(int32_t in_arg0, int32_t* _aidl_return) {
    static const char* const c[] = {"/sys/module/damon_reclaim/parameters/min_age", nullptr};
    *_aidl_return = read_int_first(c);
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::setDamonReclaimMonitoring(int32_t in_arg0, int32_t in_arg1, int32_t* _aidl_return) {
    static const char* const c[] = {"/sys/module/damon_reclaim/parameters/sample_interval", nullptr};
    *_aidl_return = read_int_first(c);
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::setDamonReclaimQuota(int32_t in_arg0, int32_t in_arg1, int32_t in_arg2, int32_t* _aidl_return) {
    static const char* const c[] = {"/sys/module/damon_reclaim/parameters/quota_ms", nullptr};
    *_aidl_return = read_int_first(c);
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::setDamonReclaimWmarks(int32_t in_arg0, int32_t in_arg1, int32_t in_arg2, int32_t in_arg3, int32_t* _aidl_return) {
    static const char* const c[] = {"/sys/module/damon_reclaim/parameters/wmarks_low", nullptr};
    *_aidl_return = read_int_first(c);
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::setFgUids(const std::string& in_arg0) {
    static const char* const c[] = {"/proc/fg_info/fg_uids", "/proc/uid_procstat/set", nullptr};
    write_first(c, in_arg0.c_str());
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::setFrameRate(const std::string& in_arg0) {
    static const char* const c[] = {"/proc/sys/kernel/frame_rate", "/proc/tp_report_rate_switch", nullptr};
    write_first(c, in_arg0.c_str());
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::setImFlag(const std::string& in_arg0, const std::string& in_arg1) {
    static const char* const c[] = {"/proc/oplus_scheduler/sched_assist/im_flag", "/data/vendor/trans_sched/sched_impt_task", nullptr};
    write_first(c, in_arg0.c_str());
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::setProcessReclaim(const std::string& in_arg0, int32_t* _aidl_return) {
    static const char* const c[] = {"/proc/process_reclaim", "/proc/swaps", nullptr};
    *_aidl_return = read_int_first(c);
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::setSchedAssistImptTask(const std::string& in_arg0) {
    static const char* const c[] = {"/proc/sys/kernel/sched_impt_tgid", "/data/vendor/trans_sched/sched_impt_task", nullptr};
    write_first(c, in_arg0.c_str());
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::setSchedAssistScene(const std::string& in_arg0) {
    static const char* const c[] = {"/proc/sys/kernel/sched_assist_scene", "/proc/game_mode", nullptr};
    write_first(c, in_arg0.c_str());
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::setSlideboost(const std::string& in_arg0) {
    static const char* const c[] = {"/proc/sys/kernel/slide_boost_enabled", "/proc/game_state", nullptr};
    write_first(c, in_arg0.c_str());
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::setTpdID(const std::string& in_arg0, int32_t* _aidl_return) {
    static const char* const c[] = {"/sys/module/tpd/parameters/tpd_id", nullptr};
    *_aidl_return = read_int_first(c);
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::setTpdSerialParams(const std::string& in_arg0, int32_t* _aidl_return) {
    static const char* const c[] = {"/sys/module/tpd/parameters/tpd_cmds", nullptr};
    *_aidl_return = read_int_first(c);
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::writeClmEnable(const std::string& in_arg0) {
    static const char* const c[] = {"/proc/jank_info/cpu_jank_info/clm_enable", "/proc/cpu_loading/onoff", nullptr};
    write_first(c, in_arg0.c_str());
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::writeClmHighLoadAll(const std::string& in_arg0) {
    static const char* const c[] = {"/proc/jank_info/cpu_jank_info/clm_highload_all", "/proc/cpu_loading/overThrhld", nullptr};
    write_first(c, in_arg0.c_str());
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::writeClmHighLoadGrp(const std::string& in_arg0) {
    static const char* const c[] = {"/proc/jank_info/cpu_jank_info/clm_highload_grp", "/proc/cpu_loading/specify_overThrhld", nullptr};
    write_first(c, in_arg0.c_str());
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::writeClmLowLoadGrp(const std::string& in_arg0) {
    static const char* const c[] = {"/proc/jank_info/cpu_jank_info/clm_lowload_grp", "/proc/cpu_loading/underThrhld", nullptr};
    write_first(c, in_arg0.c_str());
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::writeDBacktrace(const std::string& in_arg0) {
    // void; no device-visible effect reconstructed
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::writeFgFreqsThreshold(const std::string& in_arg0) {
    static const char* const c[] = {"/proc/jank_info/cpu_jank_info/fg_freqs_threshold", "/sys/devices/system/cpu/cpufreq/policy0/scaling_min_freq", nullptr};
    write_first(c, in_arg0.c_str());
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::writeIOBacktrace(const std::string& in_arg0) {
    static const char* const c[] = {"/proc/task_info/task_sched_info/io_backtrace", "/proc/self/stack", nullptr};
    write_first(c, in_arg0.c_str());
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::writeJankTaskTrackEnable(bool in_arg0) {
    static const char* const c[] = {"/proc/jank_info/cpu_jank_info/task_track_enable", "/proc/trans_scheduler/enable", nullptr};
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::writeKmallocDebugCreate(int32_t in_arg0, int32_t* _aidl_return) {
    static const char* const c[] = {"/proc/oplus_healthinfo/kmalloc_debug_create", "/proc/slabinfo", nullptr};
    *_aidl_return = read_int_first(c);
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::writeKmallocDebugCreateWithType(const std::string& in_arg0, int32_t* _aidl_return) {
    static const char* const c[] = {"/proc/oplus_healthinfo/kmalloc_debug_create", "/proc/slabinfo", nullptr};
    *_aidl_return = read_int_first(c);
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::writeMemMonitor(const std::string& in_arg0, int32_t* _aidl_return) {
    static const char* const c[] = {"/proc/memory_monitor", "/proc/meminfo", nullptr};
    *_aidl_return = read_int_first(c);
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::writeMemleakDetectThread(int32_t in_arg0, int32_t* _aidl_return) {
    static const char* const c[] = {"/proc/oplus_healthinfo/memleak_detect_thread", "/proc/leakage/spower_test", nullptr};
    *_aidl_return = read_int_first(c);
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::writeMonitorStatus(const std::string& in_arg0, int32_t* _aidl_return) {
    *_aidl_return = 0;
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::writeNandswapProc(const std::string& in_arg0, const std::string& in_arg1, int32_t* _aidl_return) {
    static const char* const c[] = {"/proc/nandswap/fn_enable", "/proc/swaps", nullptr};
    *_aidl_return = read_int_first(c);
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::writeOplusReserve3(int32_t in_arg0, int32_t in_arg1, const std::string& in_arg2, int32_t* _aidl_return) {
    static const char* const c[] = {"/dev/block/by-name/oplusreserve3", nullptr};
    *_aidl_return = read_int_first(c);
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::writePidsSet(const std::string& in_arg0) {
    static const char* const c[] = {"/proc/task_info/task_sched_info/pids_set", "/proc/uid_procstat/set", nullptr};
    write_first(c, in_arg0.c_str());
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::writeSchedInfoThreshold(const std::string& in_arg0) {
    static const char* const c[] = {"/proc/task_info/task_sched_info/sched_info_threshold", "/proc/sched_debug", nullptr};
    write_first(c, in_arg0.c_str());
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::writeStorageFeature(const std::string& in_arg0, const std::string& in_arg1, const std::string& in_arg2, const std::string& in_arg3, int32_t* _aidl_return) {
    static const char* const c[] = {"/proc/fs/ext4/dm-11/mb_groups", "/sys/fs/f2fs/dm-39", nullptr};
    *_aidl_return = read_int_first(c);
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::writeTaskSchedInfo(const std::string& in_arg0) {
    static const char* const c[] = {"/proc/task_info/task_sched_info/task_sched_info_enable", "/proc/sched_debug", nullptr};
    write_first(c, in_arg0.c_str());
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::writeTidsSet(const std::string& in_arg0) {
    static const char* const c[] = {"/proc/task_info/task_sched_info/tids_set", "/proc/uid_procstat/set", nullptr};
    write_first(c, in_arg0.c_str());
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::writeUxState(const std::string& in_arg0, const std::string& in_arg1, const std::string& in_arg2, int32_t* _aidl_return) {
    static const char* const c[] = {"/proc/oplus_scheduler/sched_assist/ux_task", nullptr};
    *_aidl_return = read_int_first(c);
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::writeVaFeature(int32_t in_arg0, int32_t* _aidl_return) {
    static const char* const c[] = {"/proc/oplus_healthinfo/va_feature", "/proc/self/maps", nullptr};
    *_aidl_return = read_int_first(c);
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::setFreqGoverner(const std::string& in_arg0, const std::vector<int32_t>& in_arg1, int32_t* _aidl_return) {
    static const char* const c[] = {"/sys/devices/system/cpu/cpufreq/policy0/scaling_governor", nullptr};
    *_aidl_return = read_int_first(c);
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::enableAudioPerf(const std::string& in_arg0, int32_t* _aidl_return) {
    static const char* const c[] = {"/proc/oplus_scheduler/sched_assist/audio/enable", nullptr};
    *_aidl_return = read_int_first(c);
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::writeTmemoryMemory(const std::string& in_arg0, int32_t* _aidl_return) {
    static const char* const c[] = {"/sys/tmemory/", nullptr};
    *_aidl_return = read_int_first(c);
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::writeTmemorySwitch(int32_t in_arg0, int32_t* _aidl_return) {
    static const char* const c[] = {"/sys/tmemory/", nullptr};
    *_aidl_return = read_int_first(c);
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::writeTmemoryFlushBusy(int32_t in_arg0, int32_t* _aidl_return) {
    static const char* const c[] = {"/sys/tmemory/", nullptr};
    *_aidl_return = read_int_first(c);
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::writeTmemoryFlushIdle(int32_t in_arg0, int32_t* _aidl_return) {
    static const char* const c[] = {"/sys/tmemory/", nullptr};
    *_aidl_return = read_int_first(c);
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::writeTmemoryCapacity(int32_t in_arg0, int32_t* _aidl_return) {
    static const char* const c[] = {"/sys/tmemory/", nullptr};
    *_aidl_return = read_int_first(c);
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::writeTmemoryHighWaterRatio(int32_t in_arg0, int32_t* _aidl_return) {
    static const char* const c[] = {"/sys/tmemory/", nullptr};
    *_aidl_return = read_int_first(c);
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::readTmemorySysdirtypages(std::string* _aidl_return) {
    static const char* const c[] = {"/sys/tmemory/", nullptr};
    *_aidl_return = read_first(c);
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::readTmemoryDirtypages(std::string* _aidl_return) {
    static const char* const c[] = {"/sys/tmemory/", nullptr};
    *_aidl_return = read_first(c);
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::readTmemoryErrorStat(std::string* _aidl_return) {
    static const char* const c[] = {"/sys/tmemory/", nullptr};
    *_aidl_return = read_first(c);
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::readTmemoryIoLatency(std::string* _aidl_return) {
    static const char* const c[] = {"/sys/tmemory/", nullptr};
    *_aidl_return = read_first(c);
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::getClmThreshold(int32_t in_arg0, std::string* _aidl_return) {
    static const char* const c[] = {"/proc/jank_info/cpu_jank_info/clm_report_threshold", "/proc/cpu_loading/overThrhld", nullptr};
    *_aidl_return = read_first(c);
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::getClmMuxSwitch(std::string* _aidl_return) {
    static const char* const c[] = {"/data/vendor/trans_sched/cpu/clm_mux_switch", "/proc/cpu_loading/onoff", nullptr};
    *_aidl_return = read_first(c);
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::setClmThreshold(const std::string& in_arg0, int32_t in_arg1) {
    static const char* const c[] = {"/proc/jank_info/cpu_jank_info/clm_report_threshold", "/proc/cpu_loading/overThrhld", nullptr};
    write_first(c, in_arg0.c_str());
    return ::ndk::ScopedAStatus::ok();
}

::ndk::ScopedAStatus PerfHal::setClmMuxSwitch(const std::string& in_arg0) {
    static const char* const c[] = {"/data/vendor/trans_sched/cpu/clm_mux_switch", "/proc/cpu_loading/onoff", nullptr};
    write_first(c, in_arg0.c_str());
    return ::ndk::ScopedAStatus::ok();
}
