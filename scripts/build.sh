#!/bin/bash
# OplusPerf/scripts/build.sh
#
# Standalone host build of the reconstructed OPlus performance HAL for
# aarch64-linux-android using the NDK — mirrors the FaceHAL build.sh style.
# Generates the AIDL NDK backend from aidl/, compiles it with the service, links
# against the ROM's own libraries, then verifies every DT_NEEDED resolves in the
# tree. Never touches the device and never repacks.
#
# Usage: scripts/build.sh
set -euo pipefail

HERE="$(dirname "$(dirname "$(readlink -f "${BASH_SOURCE[0]}")")")"
CRATE="$HERE"
ROM_TREE="${ROM_TREE:-$HERE/../vendor-tree}"
AIDL_BIN="${AIDL_BIN:-/opt/android-sdk/build-tools/34.0.0/aidl}"
NDK="${ANDROID_NDK_HOME:-/opt/android-sdk/ndk/27.3.13750724}"

RED=$'\033[1;31m'; GRN=$'\033[1;32m'; YEL=$'\033[1;33m'; NC=$'\033[0m'

[[ -x "$AIDL_BIN" ]] || { echo -e "${RED}no aidl at $AIDL_BIN${NC}"; exit 1; }
# Accept an NDK root or its bundled toolchain dir.
if [[ ! -x "$NDK/bin/clang++" ]]; then
    for cand in "$NDK"/toolchains/llvm/prebuilt/*/; do
        [[ -x "${cand}bin/clang++" ]] && NDK="${cand%/}" && break
    done
fi
[[ -x "$NDK/bin/clang++" ]] || { echo -e "${RED}no usable NDK${NC}"; exit 1; }
echo -e "${GRN}NDK:${NC} $NDK"
echo -e "${GRN}aidl:${NC} $AIDL_BIN"

TC="$(dirname "$NDK")/prebuilt/linux-x86_64"
SYSROOT="$NDK/sysroot/usr/include"
# binder_manager.h (AServiceManager_addService) is a platform NDK header, not in
# the NDK sysroot; pull it from the AOSP headers snapshot on this host.
AOSP_HEADERS="${AOSP_HEADERS:-$HERE/../aosp-headers}"
INCLUDE_PLATFORM=""
for c in "$AOSP_HEADERS/include_platform" "$AOSP_HEADERS/include_ndk"; do
    [[ -f "$c/android/binder_manager.h" ]] && INCLUDE_PLATFORM="$c" && break
done
BUILD="$CRATE/build"
GEN="$BUILD/gen"
rm -rf "$BUILD"; mkdir -p "$GEN/ndk" "$GEN/hdr"

# --- 1. generate the AIDL NDK backend from the reconstructed interface -------
AIDLROOT="$CRATE/aidl"
PKG="$AIDLROOT/vendor/oplus/hardware/performance"
# --structured + --stability=vintf: the interface is @VintfStability and defined
# entirely in AIDL (all parcelables included).
# --version/--hash are the values the shipped HAL reports: aidl only emits the
# reserved transactions (getInterfaceVersion / getInterfaceHash) for a frozen
# interface, and the shipped backend answers 1 / a1051705ca91eedbba19076e9adacaae43014172.
"$AIDL_BIN" -I "$AIDLROOT" --structured --stability=vintf --lang=ndk \
    --version 1 --hash=a1051705ca91eedbba19076e9adacaae43014172 \
    -o "$GEN/ndk" -h "$GEN/hdr" \
    "$PKG/IPerformance.aidl" "$PKG/ProcReqHal.aidl" "$PKG/ProcMemStat.aidl" \
    "$PKG/ProcMemStatRet.aidl" "$PKG/OsvelteVersionRet.aidl"
echo -e "${GRN}aidl backend:${NC} $(ls "$GEN/ndk/vendor/oplus/hardware/performance/"*.cpp | wc -l) sources, $(find "$GEN/hdr" -name '*.h' | wc -l) headers"

# Safety net: if aidl did not emit the reserved transactions, inject them.
python3 "$CRATE/scripts/inject_reserved.py" \
    "$GEN/ndk/vendor/oplus/hardware/performance/IPerformance.cpp" \
    "a1051705ca91eedbba19076e9adacaae43014172"

# --- 2. compile --------------------------------------------------------------
CXX="$NDK/bin/clang++"
# API 30 = floor for the MT6893 vendor image (matches FaceHAL); the ROM is A15.
TARGET="aarch64-linux-android30"
INCLUDES=(
    -I "$GEN/hdr"
    -I "$SYSROOT"
)
[[ -n "$INCLUDE_PLATFORM" ]] && INCLUDES+=(-I "$INCLUDE_PLATFORM")
CXXFLAGS=(-std=c++17 -fPIC -Wall -O2 --target="$TARGET" -stdlib=libc++ "${INCLUDES[@]}")

SRCS=(
    "$GEN/ndk/vendor/oplus/hardware/performance/"*.cpp
    "$CRATE/src/PerfHal.cpp"
    "$CRATE/src/main.cpp"
)

echo -e "${GRN}compiling${NC} ${#SRCS[@]} translation units..."
mkdir -p "$BUILD/obj"
OBJS=()
for s in "${SRCS[@]}"; do
    o="$BUILD/obj/$(basename "${s%.cpp}").o"
    "$CXX" "${CXXFLAGS[@]}" -c "$s" -o "$o"
    OBJS+=("$o")
done

# --- 3. link against the ROM's own libraries ---------------------------------
V64="$ROM_TREE/system/system/lib64"
SYSROOT_LIB="$NDK/sysroot/usr/lib/aarch64-linux-android"
[[ -d "$V64" ]] || V64="$ROM_TREE/vendor/lib64"
OUT="$BUILD/vendor-oplus-hardware-performance-V1-service"
echo -e "${GRN}linking${NC} (DT_NEEDED from $V64; static libc++)"
# Static libc++ so the binary carries no DT_NEEDED on the runtime apex's
# libc++_shared.so (a vendor process cannot resolve it).
"$CXX" --target="$TARGET" -stdlib=libc++ -fPIC -pie \
    "${OBJS[@]}" "$SYSROOT_LIB/libc++_static.a" -o "$OUT" \
    -L "$V64" -lbinder_ndk -lbase -lcutils -llog -lutils \
    -Wl,--no-as-needed

echo
echo -e "${GRN}artifact${NC} $OUT ($(stat -c%s "$OUT") bytes)"
file "$OUT" | sed 's/^/  /'
echo -e "${GRN}DT_NEEDED${NC}"
readelf -d "$OUT" | grep NEEDED | sed 's/^/  /'

# --- 4. verify every DT_NEEDED resolves in the ROM tree ----------------------
BIONIC_CORE="^(libc\.so|libdl\.so|libm\.so|libc\+\+\.so|libc\+\+_shared\.so|libdl_android\.so|libnativehelper\.so)$"
fail=0
while read -r lib; do
    if [[ "$lib" =~ $BIONIC_CORE ]]; then
        echo "  OK   $lib (bionic core / runtime apex)"
    elif [[ -f "$ROM_TREE/vendor/lib64/$lib" ]]; then
        echo "  OK   $lib (vendor/lib64)"
    elif [[ -f "$ROM_TREE/system/system/lib64/$lib" ]]; then
        echo "  OK   $lib (system/lib64)"
    else
        echo "  FAIL $lib not found in the ROM tree"; fail=1
    fi
done < <(readelf -d "$OUT" | awk '/NEEDED/ {gsub(/[][]/,"",$5); print $5}')

[[ $fail -eq 0 ]] || { echo -e "${RED}unresolvable DT_NEEDED — do not stage${NC}"; exit 1; }
echo -e "${GRN}all DT_NEEDED resolvable${NC}"

# --- 5. build the AIDL backend shared library (the vendor-facing .so) -------
# The shipped vendor.oplus.hardware.performance-V1-ndk_platform.so is the
# AIDL backend of this interface. Rebuilding it from the reconstructed aidl/
# makes the whole vendor side source-based. Its link profile is reproduced
# exactly: SONAME = the file name, DT_NEEDED = libbinder_ndk + libc++.so +
# libc + libm + libdl (the ROM's own libc++.so, hence -nostdlib++ and an
# explicit -lc++ against the ROM tree — the NDK's libc++_shared.so would not
# resolve in a vendor process).
SHLIB="$BUILD/vendor.oplus.hardware.performance-V1-ndk_platform.so"
SHLIB_SONAME="vendor.oplus.hardware.performance-V1-ndk_platform.so"
echo
echo -e "${GRN}building AIDL backend${NC} $SHLIB_SONAME (platform libc++ ABI)"
# The vendor-facing .so must use the ROM's libc++ inline namespace
# (std::__1), not the NDK's (std::__ndk1): the ROM's libc++.so is the platform
# build and exports only __1 symbols, so an NDK-ABI library would carry
# unresolved libc++ symbols and could not be loaded. platform_abi/__config_site
# shadows the NDK's own and switches the namespace.
mkdir -p "$BUILD/abi_obj"
ABI_OBJS=()
for s in "$GEN/ndk/vendor/oplus/hardware/performance/"*.cpp; do
    o="$BUILD/abi_obj/$(basename "${s%.cpp}").o"
    "$CXX" -std=c++17 -fPIC -O2 --target="$TARGET" -stdlib=libc++ \
        -I "$CRATE/platform_abi" -I "$GEN/hdr" -I "$SYSROOT" -c "$s" -o "$o"
    ABI_OBJS+=("$o")
done
"$CXX" --target="$TARGET" -fPIC -shared -nostdlib++ \
    "${ABI_OBJS[@]}" -o "$SHLIB" \
    -L "$V64" -lc++ -lbinder_ndk -lbase -lcutils -llog -lutils \
    -Wl,-soname,"$SHLIB_SONAME" -Wl,--no-as-needed
echo "  $(stat -c%s "$SHLIB") bytes; SONAME=$(readelf -d "$SHLIB" | sed -n 's/.*SONAME.*\[\(.*\)\]/\1/p')"
readelf -d "$SHLIB" | grep NEEDED | sed 's/^/  /'

# --- 5b. 32-bit variant of the backend ---------------------------------------
# Two 32-bit system libraries import IPerformance::descriptor and
# IPerformance::fromBinder from this backend ($ROM_TREE/system/system/lib/
# oplus_performance_shared.so and liboplusvideoboostservice.so), so the
# 32-bit copy in vendor/lib/ must exist and export the same names.
SHLIB32="$BUILD/32/$SHLIB_SONAME"
mkdir -p "$BUILD/abi_obj32" "$BUILD/32"
ABI_OBJS32=()
for s in "$GEN/ndk/vendor/oplus/hardware/performance/"*.cpp; do
    o="$BUILD/abi_obj32/$(basename "${s%.cpp}").o"
    "$CXX" -std=c++17 -fPIC -O2 --target=armv7a-linux-androideabi30 -stdlib=libc++ \
        -I "$CRATE/platform_abi" -I "$GEN/hdr" -I "$SYSROOT" -c "$s" -o "$o"
    ABI_OBJS32+=("$o")
done
"$CXX" --target=armv7a-linux-androideabi30 -fPIC -shared -nostdlib++ \
    "${ABI_OBJS32[@]}" -o "$SHLIB32" \
    -L "$ROM_TREE/system/system/lib" -lc++ -lbinder_ndk -lbase -lcutils -llog -lutils \
    -Wl,-soname,"$SHLIB_SONAME" -Wl,--no-as-needed
echo "  32-bit: $(stat -c%s "$SHLIB32") bytes; $(file -b "$SHLIB32" | cut -d, -f1-2)"

# The rebuilt backend must be a drop-in for the shipped one: same exported
# symbol names (that is the ABI), and no unresolved libc++ symbols.
SHIPPED="$ROM_TREE/vendor/lib64/$SHLIB_SONAME"
if [[ -f "$SHIPPED" ]]; then
    newsyms=$(nm -D --defined-only "$SHLIB" 2>/dev/null | awk '{print $3}' | grep "^_ZN4aidl" | sort -u)
    oldsyms=$(nm -D --defined-only "$SHIPPED" 2>/dev/null | awk '{print $3}' | grep "^_ZN4aidl" | sort -u)
    missing=$(comm -23 <(echo "$oldsyms") <(echo "$newsyms"))
    extra=$(comm -13 <(echo "$oldsyms") <(echo "$newsyms"))
    echo "  ABI vs shipped: missing=$(echo "$missing" | grep -c . ) extra=$(echo "$extra" | grep -c . )"
    [[ -n "$missing" ]] && echo "$missing" | c++filt | sed 's/^/    MISSING: /'
    # libc++ symbols must resolve against the ROM's own libc++.so (platform
    # build, inline namespace std::__1) - a shared library may carry undefined
    # symbols past the link step, and an unresolvable one means it cannot load.
    defined=$(readelf -sW "$V64/libc++.so" 2>/dev/null | awk '$7!="UND" && $8 ~ /^_Z/{print $8}' | sort -u)
    unresolved=""
    while read -r s; do
        [[ -z "$s" ]] && continue
        grep -qxF "$s" <<<"$defined" || unresolved="$unresolved $s"
    done < <(nm -D --undefined-only "$SHLIB" 2>/dev/null | awk '{print $2}' | grep -E "^_ZNSt")
    if [[ -n "$unresolved" ]]; then
        echo "  WARN unresolved libc++ symbols:$unresolved"
    else
        echo "  all libc++ symbols resolve against the ROM's libc++.so"
    fi
fi

# --- 6. build the IPC probe (verifies wire compat against the live HAL) ------
# The probe's client is generated from the RECONSTRUCTED aidl/, so calling the
# stock service with it exercises the reconstructed transaction codes and
# parcelable layouts against the real thing.
if [[ -f "$CRATE/tools/ipc_probe.cpp" ]]; then
    PROBE="$BUILD/ipc_probe"
    echo
    echo -e "${GRN}building IPC probe${NC}"
    "$CXX" "${CXXFLAGS[@]}" -c "$CRATE/tools/ipc_probe.cpp" -o "$BUILD/obj/ipc_probe.o"
    # reuse the already-compiled AIDL backend objects (PerfHal.o/main.o are the
    # service and carry their own main(), so they are excluded here)
    "$CXX" --target="$TARGET" -stdlib=libc++ -fPIC -pie \
        "$BUILD/obj/ipc_probe.o" \
        "$BUILD/obj/IPerformance.o" "$BUILD/obj/ProcReqHal.o" \
        "$BUILD/obj/ProcMemStat.o" "$BUILD/obj/ProcMemStatRet.o" \
        "$BUILD/obj/OsvelteVersionRet.o" \
        "$SYSROOT_LIB/libc++_static.a" -o "$PROBE" \
        -L "$V64" -lbinder_ndk -lbase -lcutils -llog -lutils -Wl,--no-as-needed
    echo -e "${GRN}probe${NC} $PROBE ($(stat -c%s "$PROBE") bytes)"
fi

# --- 6b. build the FULL-coverage probe ----------------------------------------
# One call per interface method, so "does the HAL return data on this device?"
# is measured over all 169 methods instead of sampled. Generated from the same
# AIDL header the service is built from.
if [[ -f "$CRATE/tools/ipc_probe_full.cpp" ]]; then
    PROBE_FULL="$BUILD/ipc_probe_full"
    echo
    echo -e "${GRN}building full-coverage probe${NC}"
    "$CXX" "${CXXFLAGS[@]}" -c "$CRATE/tools/ipc_probe_full.cpp" -o "$BUILD/obj/ipc_probe_full.o"
    "$CXX" --target="$TARGET" -stdlib=libc++ -fPIC -pie \
        "$BUILD/obj/ipc_probe_full.o" \
        "$BUILD/obj/IPerformance.o" "$BUILD/obj/ProcReqHal.o" \
        "$BUILD/obj/ProcMemStat.o" "$BUILD/obj/ProcMemStatRet.o" \
        "$BUILD/obj/OsvelteVersionRet.o" \
        "$SYSROOT_LIB/libc++_static.a" -o "$PROBE_FULL" \
        -L "$V64" -lbinder_ndk -lbase -lcutils -llog -lutils -Wl,--no-as-needed
    echo -e "${GRN}full probe${NC} $PROBE_FULL ($(stat -c%s "$PROBE_FULL") bytes)"
fi
