// The AIDL parcelables, with their wire serialisers.
//
// Field order and types come from the framework's generated `readFromParcel`
// (RE/framework_itc/smali_classes5/vendor/oplus/hardware/performance/), not
// from the AIDL names: the shipped interface declares these parcelables in a
// stub file, so the generated Java is the only authority on what the client
// actually parses.
//
// Every parcelable on the wire is versioned: a non-null marker, the byte size
// of the body (including the size field itself), then the fields. The client
// reads the size first and skips to `start + size` afterwards, so a body that
// does not declare its own length is rejected with BadParcelableException --
// and because the caller is often system_server, that rejection kills it.

use crate::parcel::Parcel;

/// `ProcMemStat`, in the order the framework reads it.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ProcMemStat {
    pub pid: i32,
    pub comm: String,
    pub is_32bit: bool,
    pub oom_score_adj: u16,
    pub nr_fds: i32,
    pub uid: i32,
    pub ppid: i32,
    pub anon: i32,
    pub file: i32,
    pub shmem: i32,
    pub swap: i32,
    pub vss: i32,
    pub pss: i32,
    pub swap_rss: i32,
    pub javaheap: i32,
    pub nativeheap: i32,
    pub ashmem: i32,
    pub dmabuf: i32,
    pub gpu: i32,
}

/// `ProcReqItemHal`: String, String, int.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ProcReqItemHal {
    pub key: String,
    pub label: String,
    pub upload_type: i32,
}

/// `ProcReqHal`: String, then a parcelable array of `ProcReqItemHal`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ProcReqHal {
    pub proc: String,
    pub items: Vec<ProcReqItemHal>,
}

/// `ProcMemStatRet`: int, then a parcelable array of `ProcMemStat`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ProcMemStatRet {
    pub status: i32,
    pub arr_ms: Vec<ProcMemStat>,
}

/// `OsvelteVersionRet`: three ints. `status` is not an error code -- the
/// Athena memory releaser branches on it and on the version pair.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct OsvelteVersionRet {
    pub status: i32,
    pub major: i32,
    pub minor: i32,
}

/// Writes a versioned parcelable body and patches its leading size field.
fn write_framed(out: &mut Parcel, fields: impl FnOnce(&mut Parcel)) {
    out.write_i32(1); // non-null marker
    let size_at = out.write_pos();
    out.write_i32(0); // patched below
    fields(out);
    let size = (out.write_pos() - size_at) as i32;
    out.patch_i32_at(size_at, size);
}

/// Reads the non-null marker and size field of a versioned parcelable and
/// returns where its body ends, so a parser never runs into the next element
/// of a containing array.
fn read_frame(input: &mut Parcel) -> Option<usize> {
    let marker = input.read_i32()?;
    if marker == 0 {
        return None;
    }
    let start = input.pos();
    let size = input.read_i32()?;
    if size < 4 {
        return None;
    }
    Some(start + size as usize)
}

impl ProcMemStat {
    pub fn write_to(&self, out: &mut Parcel) {
        write_framed(out, |out| {
            out.write_string(&self.comm);
            out.write_bool(self.is_32bit);
            out.write_i32(self.oom_score_adj as i32);
            out.write_i32(self.nr_fds);
            out.write_i32(self.uid);
            out.write_i32(self.pid);
            out.write_i32(self.ppid);
            out.write_i32(self.anon);
            out.write_i32(self.file);
            out.write_i32(self.shmem);
            out.write_i32(self.swap);
            out.write_i32(self.vss);
            out.write_i32(self.pss);
            out.write_i32(self.swap_rss);
            out.write_i32(self.javaheap);
            out.write_i32(self.nativeheap);
            out.write_i32(self.ashmem);
            out.write_i32(self.dmabuf);
            out.write_i32(self.gpu);
        });
    }

    pub fn read_from(input: &mut Parcel) -> Option<Self> {
        let end = read_frame(input)?;
        let stat = ProcMemStat {
            comm: input.read_string()?,
            is_32bit: input.read_bool()?,
            oom_score_adj: input.read_i32()?.clamp(0, u16::MAX as i32) as u16,
            nr_fds: input.read_i32()?,
            uid: input.read_i32()?,
            pid: input.read_i32()?,
            ppid: input.read_i32()?,
            anon: input.read_i32()?,
            file: input.read_i32()?,
            shmem: input.read_i32()?,
            swap: input.read_i32()?,
            vss: input.read_i32()?,
            pss: input.read_i32()?,
            swap_rss: input.read_i32()?,
            javaheap: input.read_i32()?,
            nativeheap: input.read_i32()?,
            ashmem: input.read_i32()?,
            dmabuf: input.read_i32()?,
            gpu: input.read_i32()?,
        };
        input.seek(end)?;
        Some(stat)
    }
}

impl ProcReqItemHal {
    pub fn write_to(&self, out: &mut Parcel) {
        write_framed(out, |out| {
            out.write_string(&self.key);
            out.write_string(&self.label);
            out.write_i32(self.upload_type);
        });
    }

    pub fn read_from(input: &mut Parcel) -> Option<Self> {
        let end = read_frame(input)?;
        let item = ProcReqItemHal {
            key: input.read_string()?,
            label: input.read_string()?,
            upload_type: input.read_i32()?,
        };
        input.seek(end)?;
        Some(item)
    }
}

impl ProcReqHal {
    pub fn write_to(&self, out: &mut Parcel) {
        write_framed(out, |out| {
            out.write_string(&self.proc);
            write_parcelable_array(out, &self.items, ProcReqItemHal::write_to);
        });
    }

    pub fn read_from(input: &mut Parcel) -> Option<Self> {
        let end = read_frame(input)?;
        let value = ProcReqHal {
            proc: input.read_string()?,
            items: read_parcelable_array(input, ProcReqItemHal::read_from)?,
        };
        input.seek(end)?;
        Some(value)
    }
}

impl ProcMemStatRet {
    pub fn write_to(&self, out: &mut Parcel) {
        write_framed(out, |out| {
            out.write_i32(self.status);
            write_parcelable_array(out, &self.arr_ms, ProcMemStat::write_to);
        });
    }

    pub fn read_from(input: &mut Parcel) -> Option<Self> {
        let end = read_frame(input)?;
        let value = ProcMemStatRet {
            status: input.read_i32()?,
            arr_ms: read_parcelable_array(input, ProcMemStat::read_from)?,
        };
        input.seek(end)?;
        Some(value)
    }
}

impl OsvelteVersionRet {
    pub fn write_to(&self, out: &mut Parcel) {
        write_framed(out, |out| {
            out.write_i32(self.status);
            out.write_i32(self.major);
            out.write_i32(self.minor);
        });
    }

    pub fn read_from(input: &mut Parcel) -> Option<Self> {
        let end = read_frame(input)?;
        let value = OsvelteVersionRet {
            status: input.read_i32()?,
            major: input.read_i32()?,
            minor: input.read_i32()?,
        };
        input.seek(end)?;
        Some(value)
    }
}

fn write_parcelable_array<T>(out: &mut Parcel, items: &[T], element: fn(&T, &mut Parcel)) {
    out.write_i32(items.len() as i32);
    for item in items {
        element(item, out);
    }
}

fn read_parcelable_array<T>(
    input: &mut Parcel,
    element: fn(&mut Parcel) -> Option<T>,
) -> Option<Vec<T>> {
    let count = input.read_i32()?;
    // -1 is the null array. Distinct from an empty one on the wire, but every
    // caller of this HAL wants "no records" for both.
    if count < 0 {
        return Some(Vec::new());
    }
    if count as usize > input.remaining() / 4 {
        // A count larger than the bytes that remain cannot be honest, and
        // reserving it would let a malformed parcel drive the allocation.
        return None;
    }
    let mut items = Vec::with_capacity(count as usize);
    for _ in 0..count {
        items.push(element(input)?);
    }
    Some(items)
}

/// Reply helpers, so the method bodies in hal.rs and the specials in core.rs
/// do not repeat constructors.
pub fn empty_req(proc_name: String) -> ProcReqHal {
    ProcReqHal {
        proc: proc_name,
        items: Vec::new(),
    }
}

pub fn stat_ret(status: i32, arr_ms: Vec<ProcMemStat>) -> ProcMemStatRet {
    ProcMemStatRet { status, arr_ms }
}

pub fn osvelte_version(status: i32, major: i32, minor: i32) -> OsvelteVersionRet {
    OsvelteVersionRet {
        status,
        major,
        minor,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn proc_mem_stat_round_trips() {
        let stat = ProcMemStat {
            pid: 4321,
            comm: "com.example.app".into(),
            is_32bit: true,
            oom_score_adj: 1000,
            nr_fds: 42,
            uid: 10_123,
            ppid: 1,
            anon: 100,
            file: 200,
            shmem: 300,
            swap: 400,
            vss: 500,
            pss: 600,
            swap_rss: 700,
            javaheap: 800,
            nativeheap: 900,
            ashmem: 1000,
            dmabuf: 1100,
            gpu: 1200,
        };
        let mut out = Parcel::new();
        stat.write_to(&mut out);
        let back = ProcMemStat::read_from(&mut Parcel::from_bytes(out.into_bytes())).unwrap();
        assert_eq!(back, stat);
    }

    #[test]
    fn every_parcelable_carries_its_own_size() {
        // The client rejects a body whose leading size is smaller than 4 with
        // "Parcelable too small", so the marker and a real size must lead.
        let value = osvelte_version(1, 0, 0);
        let mut out = Parcel::new();
        value.write_to(&mut out);
        let bytes = out.into_bytes();
        assert_eq!(&bytes[0..4], &1i32.to_le_bytes(), "non-null marker");
        let size = i32::from_le_bytes(bytes[4..8].try_into().unwrap());
        assert_eq!(size as usize, bytes.len() - 4, "size covers size-field-to-end");
        assert!(size > 4);
    }

    #[test]
    fn osvelte_matches_the_framework_field_count() {
        // status, major, minor -> three ints after a 4-byte size field.
        let mut out = Parcel::new();
        osvelte_version(1, 2, 3).write_to(&mut out);
        assert_eq!(out.into_bytes().len(), 4 + 4 + 12);
    }

    #[test]
    fn an_unframed_body_is_rejected() {
        // What the port shipped before the fix: fields with no size field.
        let mut out = Parcel::new();
        out.write_i32(1);
        out.write_i32(0);
        out.write_i32(0);
        assert!(OsvelteVersionRet::read_from(&mut Parcel::from_bytes(out.into_bytes())).is_none());
    }

    #[test]
    fn proc_req_hal_round_trips_with_items() {
        let req = ProcReqHal {
            proc: "meminfo".into(),
            items: vec![
                ProcReqItemHal { key: "a".into(), label: "A".into(), upload_type: 1 },
                ProcReqItemHal { key: "b".into(), label: "B".into(), upload_type: 2 },
            ],
        };
        let mut out = Parcel::new();
        req.write_to(&mut out);
        let back = ProcReqHal::read_from(&mut Parcel::from_bytes(out.into_bytes())).unwrap();
        assert_eq!(back, req);
    }

    #[test]
    fn proc_mem_stat_ret_round_trips_records() {
        let ret = stat_ret(
            1,
            vec![
                ProcMemStat { pid: 1, pss: 100, ..Default::default() },
                ProcMemStat { pid: 2, pss: 200, ..Default::default() },
            ],
        );
        let mut out = Parcel::new();
        ret.write_to(&mut out);
        let back = ProcMemStatRet::read_from(&mut Parcel::from_bytes(out.into_bytes())).unwrap();
        assert_eq!(back, ret);
    }

    #[test]
    fn a_null_array_reads_as_empty() {
        let ret = ProcMemStatRet {
            status: 1,
            arr_ms: Vec::new(),
        };
        let mut out = Parcel::new();
        ret.write_to(&mut out);
        let mut bytes = out.into_bytes();
        // Replace the element count with the null marker.
        let count_at = bytes.len() - 4;
        bytes[count_at..].copy_from_slice(&(-1i32).to_le_bytes());
        let back = ProcMemStatRet::read_from(&mut Parcel::from_bytes(bytes)).unwrap();
        assert!(back.arr_ms.is_empty());
    }

    #[test]
    fn absurd_array_length_is_rejected_not_allocated() {
        let mut out = Parcel::new();
        write_framed(&mut out, |out| {
            out.write_i32(1);
            out.write_i32(1_000_000);
        });
        assert!(ProcMemStatRet::read_from(&mut Parcel::from_bytes(out.into_bytes())).is_none());
    }
}
