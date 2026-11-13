// Reconstructed parcelable. Types + order match the shipped serializer
// (ProcMemStatRet::readFromParcel: readInt32, readInt32, readParcelableArray
// of ProcMemStat). Names are inferred and wire-irrelevant.
package vendor.oplus.hardware.performance;

import vendor.oplus.hardware.performance.ProcMemStat;

@VintfStability
parcelable ProcMemStatRet {
    int status;
    int count;
    ProcMemStat[] stats;
}
