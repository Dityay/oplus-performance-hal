#!/bin/bash
# Stage the Rust port of the OPlus Performance HAL into the unpacked vendor tree.
#
# Staged:
#   $ROM_TREE/vendor/bin/hw/vendor-oplus-hardware-performance-V1-service
#
# The init rc ($ROM_TREE/vendor/etc/init/vendor-oplus-hardware-performance-V1-service.rc:
# class hal, capabilities SYS_PTRACE, user root, group system readproc, interface
# aidl vendor.oplus.hardware.performance.IPerformance/default) and the VINTF
# fragment ($ROM_TREE/vendor/etc/vintf/manifest/manifest_performance_aidl.xml)
# point at this same path and stay unchanged.
#
# The binary lands at the same path as the build it replaces, so the existing
# fs_config / file_contexts entries still apply and nothing is regenerated
# (that only happens at repack time).
#
# This only touches files in the unpacked tree. It does NOT repack, does NOT
# flash, and does NOT touch fs_config or file_contexts.
set -euo pipefail

W=$(cd "$(dirname "$0")/.." && pwd)
ROM_TREE="${ROM_TREE:-$W/../vendor-tree}"
SRC="$W/rust/target/aarch64-linux-android/release/vendor-oplus-hardware-performance-V1-service"

GRN=$'\033[32m'; RED=$'\033[31m'; NC=$'\033[0m'

[[ -f "$SRC" ]] || { echo -e "${RED}missing $SRC: run scripts/build_rust.sh first${NC}"; exit 1; }

live="$ROM_TREE/vendor/bin/hw/vendor-oplus-hardware-performance-V1-service"
install -m 0755 "$SRC" "$live"
echo -e "${GRN}staged${NC} $live"
echo "done"