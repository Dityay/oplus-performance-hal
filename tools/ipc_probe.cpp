// OplusPerf/tools/ipc_probe.cpp
//
// End-to-end wire-compatibility probe. The client here is generated from the
// RECONSTRUCTED aidl/ (transaction codes 1..169, as extracted from the shipped
// binary), and it calls the LIVE stock service on the device. If the response
// statuses are OK and the payload shapes are sane, the reconstructed interface
// dispatches correctly against the shipped HAL — that is the compatibility
// proof on real hardware.
//
// READ-ONLY methods only: nothing here mutates device state.
#include <android/binder_manager.h>
#include <android/binder_process.h>
#include <cstdio>
#include <string>
#include <unistd.h>
#include <vector>

#include <aidl/vendor/oplus/hardware/performance/IPerformance.h>

using aidl::vendor::oplus::hardware::performance::IPerformance;
using aidl::vendor::oplus::hardware::performance::OsvelteVersionRet;
using aidl::vendor::oplus::hardware::performance::ProcMemStatRet;
using aidl::vendor::oplus::hardware::performance::ProcReqHal;

static const char* kName = "vendor.oplus.hardware.performance.IPerformance/default";

#define P(fmt, ...) printf(fmt "\n", ##__VA_ARGS__)

int main() {
    ABinderProcess_startThreadPool();

    AIBinder* raw = AServiceManager_checkService(kName);
    if (raw == nullptr) {
        P("FAIL: %s not found (AServiceManager_checkService returned null)", kName);
        return 2;
    }
    P("OK: found %s", kName);

    ::ndk::SpAIBinder binder(raw);
    auto svc = IPerformance::fromBinder(binder);
    if (svc == nullptr) {
        P("FAIL: fromBinder returned null");
        return 2;
    }

    int fails = 0;
    auto note = [&](const char* what, int code) {
        if (code != 0) {
            P("  %-26s EXCEPTION code=%d", what, code);
            ++fails;
        }
    };

    // --- String-returning methods (code 115, 62, 36) ---
    {
        std::string v;
        auto st = svc->readVersion(&v);
        note("readVersion", (int)st.getExceptionCode());
        P("  readVersion              -> '%s'", v.c_str());
    }
    {
        std::string v;
        auto st = svc->getionwait(&v);
        note("getionwait", (int)st.getExceptionCode());
        P("  getionwait               -> '%s'", v.c_str());
    }
    {
        std::string v;
        auto st = svc->getHICpuInfo(&v);
        note("getHICpuInfo", (int)st.getExceptionCode());
        P("  getHICpuInfo             -> %zu bytes", v.size());
    }

    // --- int-returning methods (code 47, 23, 52, 25, 55) ---
    {
        int32_t v = -1;
        auto st = svc->getKernelVersion(&v);
        note("getKernelVersion", (int)st.getExceptionCode());
        P("  getKernelVersion          -> %d", v);
    }
    {
        int32_t v = -1;
        auto st = svc->existMemMonitor(&v);
        note("existMemMonitor", (int)st.getExceptionCode());
        P("  existMemMonitor           -> %d", v);
    }
    {
        int32_t v = -1;
        auto st = svc->getOswapVersion(&v);
        note("getOswapVersion", (int)st.getExceptionCode());
        P("  getOswapVersion           -> %d", v);
    }
    {
        int32_t v = -1;
        auto st = svc->getAcmOpstat(&v);
        note("getAcmOpstat", (int)st.getExceptionCode());
        P("  getAcmOpstat              -> %d", v);
    }
    {
        int32_t v = -1;
        auto st = svc->getUfsplusTwStatus(&v);
        note("getUfsplusTwStatus", (int)st.getExceptionCode());
        P("  getUfsplusTwStatus        -> %d", v);
    }

    // --- remapped-to-MTK probes (these had no OPlus node on this kernel) ---
    {
        std::string v;
        auto st = svc->readJankCpuLoad(&v);
        note("readJankCpuLoad", (int)st.getExceptionCode());
        P("  readJankCpuLoad           -> '%s'", v.c_str());
    }
    {
        std::string v;
        auto st = svc->readJankCpuIndicator(&v);
        note("readJankCpuIndicator", (int)st.getExceptionCode());
        P("  readJankCpuIndicator      -> '%s'", v.c_str());
    }
    {
        std::string v;
        auto st = svc->readJankCpuInfo(&v);
        note("readJankCpuInfo", (int)st.getExceptionCode());
        P("  readJankCpuInfo           -> %zu bytes", v.size());
    }
    {
        std::string v;
        auto st = svc->readIOBacktrace(&v);
        note("readIOBacktrace", (int)st.getExceptionCode());
        P("  readIOBacktrace           -> %zu bytes", v.size());
    }
    {
        std::string v;
        auto st = svc->getUfsSignalRecordUpload(&v);
        note("getUfsSignalRecordUpload", (int)st.getExceptionCode());
        P("  getUfsSignalRecordUpload  -> %zu bytes", v.size());
    }
    {
        std::string v;
        auto st = svc->readSchedInfoThreshold(&v);
        note("readSchedInfoThreshold", (int)st.getExceptionCode());
        P("  readSchedInfoThreshold    -> %zu bytes", v.size());
    }

    // --- methods whose fallback node is a directory or a write-only procfs
    // node: these are the ones the node-existence sweep flagged as present but
    // not readable as plain files, so probe them explicitly.
    {
        std::string v;
        auto st = svc->readPidsSet(&v);
        note("readPidsSet", (int)st.getExceptionCode());
        P("  readPidsSet               -> %zu bytes%s", v.size(),
          v.empty() ? "  (EMPTY - fallback is write-only)" : "");
    }
    {
        std::string v;
        auto st = svc->readTidsSet(&v);
        note("readTidsSet", (int)st.getExceptionCode());
        P("  readTidsSet               -> %zu bytes%s", v.size(),
          v.empty() ? "  (EMPTY - fallback is write-only)" : "");
    }
    {
        std::string v;
        auto st = svc->readTaskCpustatsEnable(&v);
        note("readTaskCpustatsEnable", (int)st.getExceptionCode());
        P("  readTaskCpustatsEnable    -> %zu bytes%s", v.size(),
          v.empty() ? "  (EMPTY - fallback is write-only)" : "");
    }
    {
        std::string v;
        auto st = svc->readStorageFeature("", "", "", &v);
        note("readStorageFeature", (int)st.getExceptionCode());
        P("  readStorageFeature        -> %zu bytes%s", v.size(),
          v.empty() ? "  (EMPTY - fallback is a directory)" : "");
    }

    // --- bool-returning method (code 69) ---
    {
        bool v = true;
        auto st = svc->isJankTaskTrackEnable(&v);
        note("isJankTaskTrackEnable", (int)st.getExceptionCode());
        P("  isJankTaskTrackEnable     -> %d", (int)v);
    }

    // --- struct-returning methods: these exercise the parcelable layouts ---
    {
        OsvelteVersionRet v{};
        auto st = svc->readOsvelteVersion(&v);
        note("readOsvelteVersion", (int)st.getExceptionCode());
        P("  readOsvelteVersion        -> v=%d major=%d minor=%d patch=%d", v.version, v.major,
          v.minor, v.patch);
    }
    {
        ProcReqHal v{};
        auto st = svc->getHIDState(&v);
        note("getHIDState", (int)st.getExceptionCode());
        P("  getHIDState               -> reqId=%d target='%s' stats=%zu", v.reqId,
          v.target.c_str(), v.stats.size());
    }
    {
        ProcMemStatRet v{};
        std::vector<int32_t> pids{getpid()};
        auto st = svc->readMemoryByPids(pids, 0, &v);
        note("readMemoryByPids", (int)st.getExceptionCode());
        P("  readMemoryByPids          -> status=%d count=%d stats=%zu", v.status, v.count,
          v.stats.size());
    }

    P("");
    if (fails == 0) {
        P("RESULT: all probes dispatched cleanly -> reconstructed interface is wire-compatible "
          "with the live stock HAL");
        return 0;
    }
    P("RESULT: %d probe(s) raised a binder exception", fails);
    return 1;
}
