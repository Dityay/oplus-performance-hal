#!/bin/bash
# Build the Rust port of the OPlus Performance HAL for aarch64 Android, then
# verify the artifact offline. Never touches the device and never repacks.
#
# Output:
#   rust/target/aarch64-linux-android/release/vendor-oplus-hardware-performance-V1-service
set -euo pipefail

W=$(cd "$(dirname "$0")/.." && pwd)

# Platform libraries come from the ROM/vendor tree. The crate's linker config
# resolves libbinder_ndk.so through a local symlink so no absolute path is baked
# into the repository.
ROM_TREE="${ROM_TREE:-$W/../vendor-tree}"
NDK="${ANDROID_NDK_HOME:-${NDK:-/opt/android-sdk/ndk/27.0.12077973/toolchains/llvm/prebuilt/linux-x86_64}}"
BINDER="$ROM_TREE/system/system/lib64/libbinder_ndk.so"
[[ -f "$BINDER" ]] || BINDER="$ROM_TREE/vendor/lib64/libbinder_ndk.so"
[[ -f "$BINDER" ]] || { echo "libbinder_ndk.so not found under ROM_TREE=$ROM_TREE" >&2; exit 1; }
mkdir -p "$W/vendor-libs"
ln -sf "$BINDER" "$W/vendor-libs/libbinder_ndk.so"

cd "$W/rust"

echo "=== host unit tests (node tables, parcelables, memory walks) ==="
cargo test --offline
echo

echo "=== cross-build aarch64-linux-android (release) ==="
cargo build --release --offline --target aarch64-linux-android --bins

OUT=target/aarch64-linux-android/release
SRC="$OUT/perf_service"
DST="$OUT/vendor-oplus-hardware-performance-V1-service"
[[ -f "$SRC" ]] || { echo "build produced no binary" >&2; exit 1; }
install -m 0755 "$SRC" "$DST"

echo
echo "=== built ==="
ls -la "$DST"
"$NDK/bin/llvm-readelf" -d "$DST" | grep NEEDED
echo
echo "=== identity (service name, interface hash, log tag) ==="
for s in "vendor.oplus.hardware.performance.IPerformance/default" \
         "a1051705ca91eedbba19076e9adacaae43014172" \
         "OplusPerfHAL"; do
    n=$("$NDK/bin/llvm-strings" "$DST" | grep -c -F "$s")
    [[ "$n" -ge 1 ]] || { echo "MISSING identity string: $s" >&2; exit 1; }
    printf '  %-58s found\n' "$s"
done
echo BUILD_OK