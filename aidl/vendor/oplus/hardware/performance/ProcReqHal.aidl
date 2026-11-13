// Reconstructed parcelable. Field TYPES and ORDER match the shipped
// vendor.oplus.hardware.performance-V1-ndk_platform.so serializer exactly
// (ProcReqHal::readFromParcel: readInt32, readString, readParcelableArray).
// Field NAMES are unrecoverable from the stripped binary and do not affect the
// AIDL wire format (only type + order do).
package vendor.oplus.hardware.performance;

import vendor.oplus.hardware.performance.ProcMemStat;

@VintfStability
parcelable ProcReqHal {
    int reqId;
    String target;
    ProcMemStat[] stats;
}
