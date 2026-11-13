// Reconstructed parcelable — the per-process memory-stat record.
// The shipped serializer (ProcMemStat) emits, in order:
//   int32, String, bool, char, then 16x int32, then one trailing int32 that
//   writeToParcel finalises via getDataPosition/setDataPosition (a nullable
//   int). readFromParcel reads 17 int32 + String + bool + char = 20 values.
// This reconstruction is symmetric (aidl generates matching read+write) and
// keeps the writer's 18th int as @nullable so the byte stream the original
// writes is fully consumed. Types + ORDER are what matter on the wire; the
// names below are inferred and wire-irrelevant.
package vendor.oplus.hardware.performance;

@VintfStability
parcelable ProcMemStat {
    int pid;
    String comm;
    boolean isForeground;
    char oomAdj;
    int rss;
    int swap;
    int pss;
    int uss;
    int sharedClean;
    int sharedDirty;
    int privateClean;
    int privateDirty;
    int swapPss;
    int anonRss;
    int fileRss;
    int shmemRss;
    int gpu;
    int kernel;
    int dmabuf;
    int reserved;
}
