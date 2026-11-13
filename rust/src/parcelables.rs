// The four AIDL parcelables, with their wire serialisers.
//
// Field ORDER and TYPES are what the wire format encodes; names are inferred
// from the shipped binary and are irrelevant to compatibility. Each struct here
// carries its own `write_to` / `read_from` rather than deriving, because two of
// them have hand-written C++ serialisers that do not match a naive field-by-field
// emission and have to be reproduced exactly.

use crate::parcel::Parcel;

/// `ProcReqHal`: readInt32, readString, readParcelableArray.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ProcReqHal {
    pub req_id: i32,
    pub target: String,
    pub stats: Vec<ProcMemStat>,
}

/// `ProcMemStat`: int32, String, bool, char, 16x int32, then one trailing
/// int32 that the original writer finalises via
/// getDataPosition/setDataPosition — a nullable int. `reserved` is that
/// trailing value: `None` is the null case, and it must be written as -1.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ProcMemStat {
    pub pid: i32,
    pub comm: String,
    pub is_foreground: bool,
    pub oom_adj: i8,
    pub rss: i32,
    pub swap: i32,
    pub pss: i32,
    pub uss: i32,
    pub shared_clean: i32,
    pub shared_dirty: i32,
    pub private_clean: i32,
    pub private_dirty: i32,
    pub swap_pss: i32,
    pub anon_rss: i32,
    pub file_rss: i32,
    pub shmem_rss: i32,
    pub gpu: i32,
    pub kernel: i32,
    pub dmabuf: i32,
    pub reserved: Option<i32>,
}

/// `ProcMemStatRet`: readInt32, readInt32, readParcelableArray.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ProcMemStatRet {
    pub status: i32,
    pub count: i32,
    pub stats: Vec<ProcMemStat>,
}

/// `OsvelteVersionRet`: 4x readInt32.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct OsvelteVersionRet {
    pub version: i32,
    pub major: i32,
    pub minor: i32,
    pub patch: i32,
}

// AIDL writes a null parcelable array as -1, and a null String likewise.
#[allow(dead_code)]
const NULL_LENGTH: i32 = -1;

impl ProcMemStat {
    pub fn write_to(&self, out: &mut Parcel) {
        out.write_i32(self.pid);
        out.write_string(&self.comm);
        out.write_bool(self.is_foreground);
        out.write_byte(self.oom_adj as u8);
        out.write_i32(self.rss);
        out.write_i32(self.swap);
        out.write_i32(self.pss);
        out.write_i32(self.uss);
        out.write_i32(self.shared_clean);
        out.write_i32(self.shared_dirty);
        out.write_i32(self.private_clean);
        out.write_i32(self.private_dirty);
        out.write_i32(self.swap_pss);
        out.write_i32(self.anon_rss);
        out.write_i32(self.file_rss);
        out.write_i32(self.shmem_rss);
        out.write_i32(self.gpu);
        out.write_i32(self.kernel);
        out.write_i32(self.dmabuf);
        out.write_i32(self.reserved.unwrap_or(0));
        out.write_bool(self.reserved.is_some());
    }

    pub fn read_from(input: &mut Parcel) -> Option<Self> {
        let mut stat = ProcMemStat::default();
        stat.pid = input.read_i32()?;
        stat.comm = input.read_string()?;
        stat.is_foreground = input.read_bool()?;
        stat.oom_adj = input.read_byte()? as i8;
        stat.rss = input.read_i32()?;
        stat.swap = input.read_i32()?;
        stat.pss = input.read_i32()?;
        stat.uss = input.read_i32()?;
        stat.shared_clean = input.read_i32()?;
        stat.shared_dirty = input.read_i32()?;
        stat.private_clean = input.read_i32()?;
        stat.private_dirty = input.read_i32()?;
        stat.swap_pss = input.read_i32()?;
        stat.anon_rss = input.read_i32()?;
        stat.file_rss = input.read_i32()?;
        stat.shmem_rss = input.read_i32()?;
        stat.gpu = input.read_i32()?;
        stat.kernel = input.read_i32()?;
        stat.dmabuf = input.read_i32()?;
        let value = input.read_i32()?;
        let present = input.read_bool()?;
        stat.reserved = if present { Some(value) } else { None };
        Some(stat)
    }
}

impl ProcReqHal {
    pub fn write_to(&self, out: &mut Parcel) {
        out.write_i32(self.req_id);
        out.write_string(&self.target);
        write_stat_array(out, &self.stats);
    }

    pub fn read_from(input: &mut Parcel) -> Option<Self> {
        let req_id = input.read_i32()?;
        let target = input.read_string()?;
        let stats = read_stat_array(input)?;
        Some(ProcReqHal {
            req_id,
            target,
            stats,
        })
    }
}

impl ProcMemStatRet {
    pub fn write_to(&self, out: &mut Parcel) {
        out.write_i32(self.status);
        out.write_i32(self.count);
        write_stat_array(out, &self.stats);
    }

    pub fn read_from(input: &mut Parcel) -> Option<Self> {
        let status = input.read_i32()?;
        let count = input.read_i32()?;
        let stats = read_stat_array(input)?;
        Some(ProcMemStatRet {
            status,
            count,
            stats,
        })
    }
}

impl OsvelteVersionRet {
    pub fn write_to(&self, out: &mut Parcel) {
        out.write_i32(self.version);
        out.write_i32(self.major);
        out.write_i32(self.minor);
        out.write_i32(self.patch);
    }

    pub fn read_from(input: &mut Parcel) -> Option<Self> {
        let version = input.read_i32()?;
        let major = input.read_i32()?;
        let minor = input.read_i32()?;
        let patch = input.read_i32()?;
        Some(OsvelteVersionRet {
            version,
            major,
            minor,
            patch,
        })
    }
}

fn write_stat_array(out: &mut Parcel, stats: &[ProcMemStat]) {
    // A null array would be NULL_LENGTH; this HAL always writes a real one.
    out.write_i32(stats.len() as i32);
    for stat in stats {
        stat.write_to(out);
    }
}

fn read_stat_array(input: &mut Parcel) -> Option<Vec<ProcMemStat>> {
    let count = input.read_i32()?;
    if count < 0 {
        // A null array. Distinct from an empty one, but every caller of this
        // HAL wants "no records" either way.
        return Some(Vec::new());
    }
    if count as usize > input.remaining() / 4 {
        // The length prefix cannot exceed the bytes actually in the parcel.
        // Guarding here keeps a malformed count from pre-allocating wildly.
        return None;
    }
    let mut stats = Vec::with_capacity(count as usize);
    for _ in 0..count {
        stats.push(ProcMemStat::read_from(input)?);
    }
    Some(stats)
}

/// The shipped defaults, kept as named constructors so the method bodies read
/// declaratively rather than repeating literals.
pub fn empty_req(req_id: i32, target: String) -> ProcReqHal {
    ProcReqHal {
        req_id,
        target,
        stats: Vec::new(),
    }
}

pub fn stat_ret(status: i32, stats: Vec<ProcMemStat>) -> ProcMemStatRet {
    ProcMemStatRet {
        status,
        count: stats.len() as i32,
        stats,
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
            is_foreground: true,
            oom_adj: -900i16 as i8,
            rss: 1024,
            swap: 64,
            pss: 900,
            uss: 800,
            shared_clean: 10,
            shared_dirty: 11,
            private_clean: 12,
            private_dirty: 13,
            swap_pss: 14,
            anon_rss: 15,
            file_rss: 16,
            shmem_rss: 17,
            gpu: 18,
            kernel: 19,
            dmabuf: 20,
            reserved: Some(21),
        };
        let mut out = Parcel::new();
        stat.write_to(&mut out);
        let bytes = out.into_bytes();
        let back = ProcMemStat::read_from(&mut Parcel::from_bytes(bytes)).unwrap();
        assert_eq!(back, stat);
    }

    #[test]
    fn nullable_reserved_survives_the_round_trip() {
        let stat = ProcMemStat {
            reserved: None,
            ..Default::default()
        };
        let mut out = Parcel::new();
        stat.write_to(&mut out);
        let back = ProcMemStat::read_from(&mut Parcel::from_bytes(out.into_bytes())).unwrap();
        assert_eq!(back.reserved, None);
    }

    #[test]
    fn proc_req_hal_round_trips_with_records() {
        let req = ProcReqHal {
            req_id: 7,
            target: "Some target text".into(),
            stats: vec![
                ProcMemStat { pid: 1, rss: 100, ..Default::default() },
                ProcMemStat { pid: 2, rss: 200, ..Default::default() },
            ],
        };
        let mut out = Parcel::new();
        req.write_to(&mut out);
        let back = ProcReqHal::read_from(&mut Parcel::from_bytes(out.into_bytes())).unwrap();
        assert_eq!(back, req);
    }

    #[test]
    fn proc_mem_stat_ret_reports_its_own_count() {
        let ret = stat_ret(1, vec![ProcMemStat::default(), ProcMemStat::default()]);
        assert_eq!(ret.count, 2);
        let mut out = Parcel::new();
        ret.write_to(&mut out);
        let back = ProcMemStatRet::read_from(&mut Parcel::from_bytes(out.into_bytes())).unwrap();
        assert_eq!(back, ret);
    }

    #[test]
    fn null_stat_array_reads_as_empty() {
        let mut out = Parcel::new();
        out.write_i32(1);
        out.write_i32(0);
        out.write_i32(NULL_LENGTH);
        let ret = ProcMemStatRet::read_from(&mut Parcel::from_bytes(out.into_bytes())).unwrap();
        assert!(ret.stats.is_empty());
    }

    #[test]
    fn absurd_array_length_is_rejected_not_allocated() {
        let mut out = Parcel::new();
        out.write_i32(1);
        out.write_i32(0);
        out.write_i32(1_000_000);
        let parsed = ProcMemStatRet::read_from(&mut Parcel::from_bytes(out.into_bytes()));
        assert!(parsed.is_none());
    }

    #[test]
    fn osvelte_is_four_ints() {
        let v = OsvelteVersionRet {
            version: 1,
            major: 2,
            minor: 3,
            patch: 4,
        };
        let mut out = Parcel::new();
        v.write_to(&mut out);
        assert_eq!(out.into_bytes().len(), 16);
        let back = OsvelteVersionRet::read_from(&mut Parcel::from_bytes(v_bytes(&v))).unwrap();
        assert_eq!(back, v);
    }

    fn v_bytes(v: &OsvelteVersionRet) -> Vec<u8> {
        let mut out = Parcel::new();
        v.write_to(&mut out);
        out.into_bytes()
    }
}