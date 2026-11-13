// The cross-platform half of the OPlus Performance HAL port.
//
// The node tables, the candidate-walk operations, the two memory walks and the
// parcelable serialisers live here so they are unit-testable on the host
// against one fake /proc tree. The binder-facing half (wire.rs / hal.rs) is an
// Android-only binary in src/bin/perf_service.rs.

pub mod core;
pub mod io_shim;
pub mod nodeconv;
pub mod nodes;
pub mod parcel;
pub mod parcelables;