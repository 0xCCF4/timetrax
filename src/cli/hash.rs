use crate::data::manager::Manager;
use std::collections::HashMap;
use std::io::IsTerminal;

/// For every hash in `hashes`, compute the minimum number of leading characters
/// that uniquely distinguish it from every other hash in the slice.
///
/// A hash that shares no leading character with any other gets a prefix length
/// of 1.  Longer shared prefixes push the required length up accordingly.
#[must_use]
pub fn unique_prefix_map(hashes: &[String]) -> HashMap<String, usize> {
    let mut map: HashMap<String, usize> =
        hashes.iter().map(|h| (h.clone(), 1usize)).collect();

    for i in 0..hashes.len() {
        for j in (i + 1)..hashes.len() {
            let shared = hashes[i]
                .chars()
                .zip(hashes[j].chars())
                .take_while(|(a, b)| a == b)
                .count();
            let needed = shared + 1;
            *map.entry(hashes[i].clone()).or_insert(1) =
                map[&hashes[i]].max(needed);
            *map.entry(hashes[j].clone()).or_insert(1) =
                map[&hashes[j]].max(needed);
        }
    }
    map
}

/// Render a hash for terminal display.
/// The first `unique_len` chars are **bold**; the next chars (up to `total_len`)
/// are dim.  Falls back to plain text when `color` is false.
#[must_use]
pub fn render_hash(hash: &str, unique_len: usize, total_len: usize, color: bool) -> String {
    let show = total_len.min(hash.len());
    let cut = unique_len.min(show);
    let bold = &hash[..cut];
    let dim = &hash[cut..show];
    if color {
        format!("\x1b[1m{bold}\x1b[0m\x1b[2m{dim}\x1b[0m")
    } else {
        format!("{bold}{dim}")
    }
}

/// Collect every activity hash across the whole database (for global uniqueness).
#[must_use]
pub fn all_hashes(manager: &Manager) -> Vec<String> {
    manager
        .days
        .values()
        .flat_map(|d| d.inner().activities.iter().map(super::super::az_hash::AZHash::az_hash_sha512))
        .collect()
}

/// Whether stdout supports ANSI colour codes.
#[must_use]
pub fn stdout_color() -> bool {
    std::io::stdout().is_terminal()
}

/// Whether stderr supports ANSI colour codes.
#[must_use]
pub fn stderr_color() -> bool {
    std::io::stderr().is_terminal()
}
