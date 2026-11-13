# OplusPerf

Rust rewrite of the OPlus Performance HAL
(`vendor.oplus.hardware.performance.IPerformance`) for the X6739 / MT6893. No
C/C++ source, no prebuilt C libraries: the service is a single Rust binary
speaking the binder NDK C API directly and emitting the same AIDL wire format
the shipped backend does.

The C++ tree it replaces is preserved in this repo under `src/` and `re/` as
the reconstruction record; the Rust port lives under `rust/`.

## What the service is

A 169-method AIDL interface, `@VintfStability`, over the kernel's performance
and diagnostics nodes. The interface is wide but shallow:

- ~90 readers returning a `String` (a `/proc` dump, a node value, a version banner)
- ~20 readers returning an `int` (an enable flag, a status code)
- ~25 writers pushing a string or an int into a knob
- 2 struct-returning readers (`readMemoryByPids` / `readMemoryByUids`)
- the rest are the `getHI*` health-info readers returning `ProcReqHal`, the
  `Acm*` app-compact methods (constant on this kernel), and the `Tmemory*`
  group

The porting problem is the same one the other HALs face: the backend was
written for a Qualcomm SM8xxx kernel. MT6893 does not publish
`/proc/jank_info/cpu_jank_info/*`, `/proc/oplus_healthinfo/*`,
`/proc/task_info/*` or `/dev/memcg/apps`. Every method keeps its original OPlus
node as the primary candidate and adds this device's MTK/Transsion equivalent
as a fallback; the first candidate that answers wins. That makes the HAL
faithful on an OPlus kernel and useful on the MT6893.

`re/kernel_nodes_equiv.md` is the per-method evidence: every MTK path was
confirmed present on the device before it was written down.

## Layout

```
aidl/vendor/oplus/hardware/performance/     the AIDL surface (169 methods, 4 parcelables)
rust/src/lib.rs                 crate root
rust/src/nodes.rs               NODE_TABLE: one Plan per AIDL method, in transaction order
rust/src/nodeconv.rs            the candidate-walk operations (read_first, write_first, ...)
rust/src/core.rs                readMemoryByPids / readMemoryByUids, smaps parsing, specials
rust/src/parcel.rs              a host byte-level parcel codec for the round-trip tests
rust/src/parcelables.rs         the four AIDL parcelables and their serialisers
rust/src/io_shim.rs             absolute-path I/O with a test-root override
rust/src/wire.rs                libbinder_ndk FFI + AParcel helpers
rust/src/hal.rs                 onTransact: the arg shapes and the generic plan executor
rust/src/bin/perf_service.rs    main(): thread pool, addService, joinThreadPool
scripts/build_rust.sh           host tests + aarch64 cross-build + identity verification
scripts/stage_vendor.sh         copy artifact into the staging tree (no repack)
re/                             the reconstruction record: disassembly, node sweeps,
                                the C++ source it replaces, and the equivalences
```

## The single-source-of-truth principle

Two hand-maintained surfaces have to agree with the AIDL file: the node table
(`nodes.rs`, transaction order) and the in-parameter shapes (`hal.rs::read_args`).
Both are checked against the AIDL by tests rather than by review:

- `nodes::tests::table_matches_the_aidl_declaration_order_exactly` re-parses
  the AIDL and compares all 169 names in order;
- `hal::tests::arg_arm_cover_exactly_matches_the_aidl_signatures` re-parses the
  AIDL signatures and checks that the union of `read_args` arms is exactly the
  set of methods that carry in-parameters — in both directions.

That second test caught two real mistakes while it was being written: code 82
(`readIomonitorInfo`) missing from every arm, and code 33 wrongly listed as
taking a `String`. Both would have been silent wire corruption, and neither
was visible by reading the code.

Transaction codes are 1..=169 in AIDL declaration order and **must not be
reordered**: AIDL assigns codes by declaration order, so a shuffle breaks wire
compatibility with the shipped HAL and every prebuilt client.

## The memory walks

`readMemoryByPids` and `readMemoryByUids` are the two methods that do real
work rather than a node read.

The OPlus node was a per-uid memcg file. This kernel's memcg is per-app-group
(`/dev/memcg/apps`), not per-uid, so the honest per-process source is
`/proc/<pid>/smaps_rollup`, falling back to `/proc/<pid>/smaps`, falling back
to `/proc/<pid>/statm`. The last fallback is not theoretical: the HAL runs in a
generic `hal_*` SELinux domain that is denied the per-process smaps, while
`statm` is world-readable. `statm` carries no PSS, so PSS is reported as 0
rather than imitated from RSS.

`readMemoryByUids` walks `/proc/<pid>/status` once, groups processes by their
real uid (the first value on the `Uid:` line), and sums each group's
smaps_rollup. One walk serves every requested uid.

## The reply size cap

A binder reply is capped at ~1 MiB. Several of the nodes the MTK fallbacks
stand in for are far larger than the OPlus originals — `/proc/vmallocinfo` is
1.15 MB on this kernel — and a reply that size fails the transaction outright
with `EX_TRANSACTION_FAILED` (-129). Reads are capped at 256 KiB
(`io_shim::MAX_READ`), so the call succeeds and returns a truncated but usable
dump rather than failing.

## Build

Pinned toolchain: `rust-toolchain.toml` selects rustc 1.99.1 with the
`aarch64-linux-android` target. Reproducibility only holds at that version.

```sh
bash scripts/build_rust.sh
```

That runs the host tests, cross-builds, renames `perf_service` to its on-device
name, and verifies the identity strings and DT_NEEDED set.

Resulting DT_NEEDED:

```
libdl.so  liblog.so  libc.so  libbinder_ndk.so
```

No `libc++.so`. The one `__cxa_atexit` that trips a naive C++-leak grep
resolves from libc.

`libbinder_ndk.so` is linked from the ROM tree
(`erofs/system/system/lib64/libbinder_ndk.so`), not the NDK stub — the service
needs `AServiceManager_addService` and the `ABinderProcess_*` entry points the
public stub does not export.

Reproducibility: consecutive clean builds produce identical sha256
(`038bc0166ce9e11d187c5ccaa6eb216ebb82e5234de23256d604c0c9580ab7f0`).

## Identity

```
descriptor   vendor.oplus.hardware.performance.IPerformance
instance     vendor.oplus.hardware.performance.IPerformance/default
hash         a1051705ca91eedbba19076e9adacaae43014172
version      1
log tag      OplusPerfHAL
```

Matches `erofs/vendor/etc/init/vendor-oplus-hardware-performance-V1-service.rc`
and the VINTF fragment `erofs/vendor/etc/vintf/manifest/manifest_performance_aidl.xml`.

## Testing without a device

`io_shim` routes every absolute path through a process-global root override.
Tests set it to a temp directory, build a fake `/proc` tree (including process
directories with `status`, `smaps_rollup`, `comm`, `statm`), and exercise the
real code paths against it.

```sh
cd rust && cargo test --offline
```

The parcel round-trip tests use a hand-written byte-level parcel codec
(`parcel.rs`) that mirrors libbinder's layout — i32s, the padded byte for the
`char` field, length-prefixed strings, nullable arrays — so a change to a
parcelable's field order fails on the host before it reaches a device.

## What is not done here

- No repack, no flash. `scripts/stage_vendor.sh` copies the artifact into the
  staging tree only.
- The init rc, the VINTF fragment and the AIDL file are the stock ones.
- Device-side liveness (does the framework actually bind and call it) waits on
  the next vendor repack.