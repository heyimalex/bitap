use super::*;

use std::cmp;
use strsim;

pub enum DistanceFn {
    Levenshtein,
    DamerauLevenshtein,
    OptimalStringAlignment,
}

/// The baseline functions should be functionally the same as the reference
/// functions, but internally use a much slower algorithm. It's a reference
/// for the reference so to speak. Handy because early iterations of the
/// reference function had pretty serious logic flaws that would not have been
/// caught without this!
pub fn baseline(
    pattern: &str,
    text: &str,
    max_distance: usize,
    distance_fn: DistanceFn,
) -> super::BitapResult {
    // The results of bitap (the algorithm) should be the same as taking the edit
    // distance between the pattern and _every possible_ substring of the text.
    // This function doesn't need to be fast, it just needs to be right, so
    // that's exactly what it does! This should prevent a class of issues that
    // came up in the past where my "reference" impl had the same flaws as my
    // production version.

    // Enforce bitap-specific pattern size limits.
    let pattern_len = pattern.chars().count();
    if !pattern_length_is_valid(pattern_len) {
        return Err(ERR_INVALID_PATTERN);
    }

    // Clamp max edit distance to the length of the pattern.
    let max_distance = cmp::min(max_distance, pattern_len);

    let text_chars = text.chars().collect::<Vec<_>>();

    let mut results = Vec::new();

    for i in 0..text_chars.len() {
        // Clamp the range of substrings we test; we know that to achieve
        // max_distance, the difference in length between the pattern and the
        // substring can be at most max_distance.
        let max_diff = max_distance + pattern_len;
        let start = if i > max_diff { i - max_diff } else { 0 };

        let mut best_distance: usize = max_distance + 1;
        for j in start..=i {
            let sub_text: String = text_chars[j..=i].iter().collect();
            let distance = match distance_fn {
                DistanceFn::Levenshtein => strsim::levenshtein(pattern, &sub_text),
                DistanceFn::DamerauLevenshtein => strsim::damerau_levenshtein(pattern, &sub_text),
                DistanceFn::OptimalStringAlignment => strsim::osa_distance(pattern, &sub_text),
            };
            if distance < best_distance {
                best_distance = distance;
            }
            if best_distance == 0 {
                break;
            }
        }
        if best_distance <= max_distance {
            results.push(Match {
                distance: best_distance,
                end: i,
            })
        }
    }
    Ok(results)
}

pub fn find(pattern: &str, text: &str) -> FindResult {
    baseline(pattern, text, 0, DistanceFn::Levenshtein).map(|v| {
        let offset = pattern.chars().count() - 1;
        v.iter().map(|m| m.end - offset).collect::<Vec<_>>()
    })
}

pub fn lev(pattern: &str, text: &str, k: usize) -> BitapResult {
    baseline(pattern, text, k, DistanceFn::Levenshtein)
}

pub fn damerau(pattern: &str, text: &str, k: usize) -> BitapResult {
    baseline(pattern, text, k, DistanceFn::DamerauLevenshtein)
}

pub fn osa(pattern: &str, text: &str, k: usize) -> BitapResult {
    baseline(pattern, text, k, DistanceFn::OptimalStringAlignment)
}
