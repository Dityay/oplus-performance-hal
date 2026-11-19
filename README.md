# OPlusPerf

A Rust implementation of the OPlus `vendor.oplus.hardware.performance` AIDL HAL (V1, `IPerformance/default`).

## What the service does

The service registers a VINTF-stable binder interface that exposes the platform's performance and diagnostics nodes to the framework. The interface is wide and mostly shallow:

- readers returning a `String` with a node dump, a value, or a version banner;
- readers returning an `int` with an enable flag or status code;
- writers that push a string or an integer into a knob;
- two struct-returning memory readers (`readMemoryByPids`, `readMemoryByUids`) that report per-process and per-uid memory usage from `/proc`;
- health-info, app-compaction and memory groups that return parcelable records.

Each method keeps its OPlus node as the primary candidate and walks a list of fallback nodes until one answers. Reads are bounded so a large node dump still fits in a binder reply.

## Repository layout

```
aidl/vendor/oplus/hardware/performance/   AIDL interface and parcelables
rust/src/lib.rs                 crate root
rust/src/nodes.rs               node plan table, in AIDL declaration order
rust/src/nodeconv.rs            candidate-walk read/write helpers
rust/src/core.rs                memory-stat walks and smaps parsing
rust/src/parcel.rs              byte-level parcel codec used by the host tests
rust/src/parcelables.rs         AIDL parcelables and their serialisers
rust/src/io_shim.rs             path I/O with a test-root override
rust/src/wire.rs                libbinder_ndk FFI and AParcel helpers
rust/src/hal.rs                 onTransact argument decoding and plan execution
rust/src/bin/perf_service.rs    service entry point: thread pool and addService
src/                            reference C++ implementation of the same interface
platform_abi/                   platform libc++ ABI override header
scripts/                        build, verification and staging helpers
tools/                          IPC probe sources
```

## AIDL

The interface and its four parcelables live under `aidl/vendor/oplus/hardware/performance/`. It is `@VintfStability`, version 1, with a frozen-interface hash passed to `aidl` by the build scripts. Transaction codes follow declaration order, so the order of declarations in `IPerformance.aidl` is part of the wire contract and must not be shuffled.

Two tests re-parse the AIDL at test time and check that the node table and the argument-decoding arms still match it exactly.

## Reference C++ tree

`src/` holds the C++ implementation of the interface (`PerfHal.cpp`, `PerfHal.h`, `main.cpp`). `Android.bp` builds it with Soong as `vendor-oplus-hardware-performance-V1-service` and wires up the init rc and VINTF fragment. `scripts/build.sh` builds the same interface with the standalone NDK/AIDL toolchain, including the AIDL backend shared library and its 32-bit variant.

## Build requirements

- Rust toolchain from `rust/rust-toolchain.toml` with the `aarch64-linux-android` target (installed by rustup).
- Android NDK with `clang++` and `llvm-readelf` / `llvm-strings`.
- AOSP `aidl` tool for the C++/AIDL path.
- AOSP headers providing `android/binder_manager.h` for the C++ build.
- A ROM/vendor tree supplying `libbinder_ndk.so` and the other platform libraries. Point `ROM_TREE` at it; the scripts default to a sibling `vendor-tree/` directory.

## Prebuilt

[Get Prebuilt](https://github.com/Xia-s-Projekt/oplus-performance-hal/releases/latest) — AArch64 ODM archive containing the service under `odm/bin/hw/`, its init rc under `odm/etc/init/`, and the VINTF manifest under `odm/etc/vintf/manifest/`.

## Build

```sh
bash scripts/build_rust.sh
```

This runs the host unit tests, cross-builds the release binary, installs it under its service name, and checks the embedded identity strings and `DT_NEEDED` entries. The build links the platform `libbinder_ndk.so` from `ROM_TREE` through a local `vendor-libs/` symlink created by the script, because the service needs the `AServiceManager_addService` and `ABinderProcess_*` entry points that the public NDK stub does not export.

To build the C++/AIDL backend path:

```sh
bash scripts/build.sh
```

Host tests can be run directly with `cd rust && cargo test --offline`.

## Staging

```sh
ROM_TREE=/path/to/rom bash scripts/stage_vendor.sh
```

The script copies the built binary to `vendor/bin/hw/vendor-oplus-hardware-performance-V1-service` under `ROM_TREE`. The existing init rc and VINTF fragment already reference that path. Staging does not build an image, flash a device, or regenerate file-context configuration.
