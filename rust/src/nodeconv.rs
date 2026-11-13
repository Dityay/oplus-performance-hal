// The candidate-list operations, ported from the C++ helpers in PerfHal.cpp.
//
// Every one of them shares the same contract: walk the candidates in order and
// stop at the first that answers. "Answers" means non-empty output for a read,
// an accepted write for a write, existence for a stat. An empty read is
// indistinguishable from a missing node and is treated as such.

use crate::io_shim;
use crate::nodes::subst;

/// First candidate that yields non-empty content.
pub fn read_first(cands: &[&str]) -> String {
    for cand in cands {
        let text = io_shim::read(cand);
        if !text.is_empty() {
            return text;
        }
    }
    String::new()
}

/// First candidate that yields a parseable leading integer, else 0. This is
/// the C++ `strtol(s.c_str(), nullptr, 10)` behaviour: non-numeric prefixes
/// yield 0, and a candidate that parses to 0 still wins the race because the
/// read succeeded.
pub fn read_int_first(cands: &[&str]) -> i32 {
    for cand in cands {
        let text = io_shim::read(cand);
        if text.is_empty() {
            continue;
        }
        return parse_leading_int(&text);
    }
    0
}

/// First integer appearing anywhere in the text. Needed for nodes that wrap
/// their value ("UFS Host state=2", "enable=1"): a plain leading parse would
/// return 0 for all of them.
pub fn read_int_scan_first(cands: &[&str]) -> i32 {
    for cand in cands {
        let text = io_shim::read(cand);
        if text.is_empty() {
            continue;
        }
        if let Some(value) = scan_first_int(&text) {
            return value;
        }
    }
    0
}

/// Does any candidate exist?
pub fn path_exists_first(cands: &[&str]) -> bool {
    cands.iter().any(|cand| io_shim::exists(cand))
}

/// First candidate that accepts the write. Returns whether one did.
pub fn write_first(cands: &[&str], value: &str) -> bool {
    for cand in cands {
        if io_shim::write(cand, value) {
            return true;
        }
    }
    false
}

pub fn write_int_first(cands: &[&str], value: i32) -> bool {
    write_first(cands, &value.to_string())
}

/// Directory listing, first candidate that yields anything.
pub fn read_dir_first(cands: &[&str]) -> String {
    for cand in cands {
        let listing = io_shim::list_dir(cand);
        if !listing.is_empty() {
            return listing;
        }
    }
    String::new()
}

/// `read_first` with the string argument substituted into `%s`.
pub fn read_first_sub(cands: &[&str], arg: &str) -> String {
    for cand in cands {
        let text = io_shim::read(&subst(cand, arg));
        if !text.is_empty() {
            return text;
        }
    }
    String::new()
}

/// The shipped service reports the kernel as major*100 + minor from the
/// "Linux version 5.10.270..." banner, so 5.10 becomes 510.
pub fn kernel_version(cands: &[&str]) -> i32 {
    for cand in cands {
        let text = io_shim::read(cand);
        let Some(pos) = text.find("version") else {
            continue;
        };
        let rest = &text[pos + "version".len()..];
        let rest = rest.trim_start();
        let (major, minor) = match parse_major_minor(rest) {
            Some(v) => v,
            None => continue,
        };
        return major * 100 + minor;
    }
    0
}

/// Parse a leading decimal integer, C `strtol` style: optional sign, leading
/// whitespace, stop at the first non-digit. Returns 0 when no digits follow.
pub fn parse_leading_int(text: &str) -> i32 {
    let trimmed = text.trim_start();
    let mut chars = trimmed.char_indices().peekable();
    let mut sign: i64 = 1;
    if let Some((_, c)) = chars.peek().copied() {
        if c == '-' {
            sign = -1;
            chars.next();
        } else if c == '+' {
            chars.next();
        }
    }
    let mut value: i64 = 0;
    let mut any = false;
    for (_, c) in chars {
        let Some(digit) = c.to_digit(10) else {
            break;
        };
        any = true;
        value = value.saturating_mul(10).saturating_add(digit as i64);
        if value > i32::MAX as i64 {
            value = i32::MAX as i64;
        }
    }
    if !any {
        return 0;
    }
    (value * sign) as i32
}

/// The first run of digits anywhere in the text.
fn scan_first_int(text: &str) -> Option<i32> {
    let start = text.find(|c: char| c.is_ascii_digit())?;
    let mut value: i64 = 0;
    for c in text[start..].chars() {
        let Some(digit) = c.to_digit(10) else {
            break;
        };
        value = value.saturating_mul(10).saturating_add(digit as i64);
        if value > i32::MAX as i64 {
            value = i32::MAX as i64;
        }
    }
    Some(value as i32)
}

/// "5.10.270-StargazerKernel" -> (5, 10). Requires both components.
fn parse_major_minor(text: &str) -> Option<(i32, i32)> {
    let mut parts = text.split(|c: char| !c.is_ascii_digit());
    let major: i32 = parts.next()?.parse().ok()?;
    let minor: i32 = parts.next()?.parse().ok()?;
    Some((major, minor))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn leading_int_matches_strtol() {
        assert_eq!(parse_leading_int("42\n"), 42);
        assert_eq!(parse_leading_int("  -7"), -7);
        assert_eq!(parse_leading_int("0x10"), 0);
        assert_eq!(parse_leading_int("garbage"), 0);
        assert_eq!(parse_leading_int("-"), 0);
        assert_eq!(parse_leading_int(""), 0);
    }

    #[test]
    fn scan_finds_a_value_inside_a_sentence() {
        assert_eq!(scan_first_int("UFS Host state=2"), Some(2));
        assert_eq!(scan_first_int("enable=1"), Some(1));
        assert_eq!(scan_first_int("no digits here"), None);
    }

    #[test]
    fn kernel_version_is_major_times_a_hundred_plus_minor() {
        let _g = io_shim::test_lock();
        let root = "/tmp/perf_nodes_kver";
        let _ = std::fs::remove_dir_all(root);
        std::fs::create_dir_all(format!("{}/proc", root)).unwrap();
        io_shim::set_test_root(root);
        std::fs::write(
            format!("{}/proc/version", root),
            "Linux version 5.10.270-StargazerKernel-CelestialSphere (builder@x) #1 SMP\n",
        )
        .unwrap();
        assert_eq!(kernel_version(&["/proc/version"]), 510);
        io_shim::clear_test_root();
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn kernel_version_of_a_missing_node_is_zero() {
        let _g = io_shim::test_lock();
        let root = "/tmp/perf_nodes_kver_missing";
        let _ = std::fs::remove_dir_all(root);
        std::fs::create_dir_all(root).unwrap();
        io_shim::set_test_root(root);
        assert_eq!(kernel_version(&["/proc/version"]), 0);
        io_shim::clear_test_root();
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn read_first_falls_through_an_empty_candidate() {
        let _g = io_shim::test_lock();
        let root = "/tmp/perf_nodes_fallthrough";
        let _ = std::fs::remove_dir_all(root);
        std::fs::create_dir_all(format!("{}/a", root)).unwrap();
        std::fs::create_dir_all(format!("{}/b", root)).unwrap();
        io_shim::set_test_root(root);
        std::fs::write(format!("{}/a/empty", root), "").unwrap();
        std::fs::write(format!("{}/b/full", root), "fallback").unwrap();
        assert_eq!(read_first(&["/a/empty", "/b/full"]), "fallback");
        io_shim::clear_test_root();
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn the_original_node_wins_when_present() {
        let _g = io_shim::test_lock();
        let root = "/tmp/perf_nodes_priority";
        let _ = std::fs::remove_dir_all(root);
        std::fs::create_dir_all(format!("{}/orig", root)).unwrap();
        std::fs::create_dir_all(format!("{}/fallback", root)).unwrap();
        io_shim::set_test_root(root);
        std::fs::write(format!("{}/orig/node", root), "oplus").unwrap();
        std::fs::write(format!("{}/fallback/node", root), "mtk").unwrap();
        assert_eq!(read_first(&["/orig/node", "/fallback/node"]), "oplus");
        io_shim::clear_test_root();
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn write_int_first_formats_decimal() {
        let _g = io_shim::test_lock();
        let root = "/tmp/perf_nodes_writeint";
        let _ = std::fs::remove_dir_all(root);
        std::fs::create_dir_all(format!("{}/proc", root)).unwrap();
        io_shim::set_test_root(root);
        std::fs::write(format!("{}/proc/node", root), "").unwrap();
        assert!(write_int_first(&["/proc/node"], -3));
        assert_eq!(
            std::fs::read_to_string(format!("{}/proc/node", root)).unwrap(),
            "-3"
        );
        io_shim::clear_test_root();
        let _ = std::fs::remove_dir_all(root);
    }
}