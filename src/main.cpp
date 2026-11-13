// OplusPerf service entry point.
// Registers vendor.oplus.hardware.performance.IPerformance/default on the
// vndbinder, matching the shipped init rc:
//   service oplus.performance.hal.service-1-0 /vendor/bin/hw/vendor-oplus-hardware-performance-V1-service
//       interface aidl vendor.oplus.hardware.performance.IPerformance/default
#include <android/binder_manager.h>
#include <android/binder_process.h>
#include <android/log.h>

#include "PerfHal.h"

#define LOGI(...) __android_log_print(ANDROID_LOG_INFO, "OplusPerfHAL", __VA_ARGS__)

static const char* kServiceName = "vendor.oplus.hardware.performance.IPerformance/default";

int main() {
    // The stock rc grants SYS_PTRACE + readproc; the diagnostic read path is
    // many small /proc reads per transaction, so give the pool a few threads.
    ABinderProcess_startThreadPool();

    auto hal = ::ndk::SharedRefBase::make<PerfHal>();
    ::ndk::SpAIBinder binder = hal->asBinder();
    binder_exception_t st = AServiceManager_addService(binder.get(), kServiceName);
    if (st != EX_NONE) {
        LOGI("failed to register %s (status %d)", kServiceName, (int)st);
        return 1;
    }
    LOGI("registered %s", kServiceName);
    ABinderProcess_joinThreadPool();
    return 0;
}
