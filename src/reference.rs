//! This module contains reference implementations of the bitap functions. You
//! probably shouldn't be using it externally.

use std::collections::HashMap;
use std::mem;

use super::Match;

/// This is the reference implementation of the algorithm. It works for all
/// unicode text, and is intentionally straight forward. I'm mostly using it
/// for testing purposes and working out appropriate behavior for edge cases.
/// The functions exported from this module are just specializations of this
/// function.
fn bitap_reference<'a>(
    text: &'a str,
    pattern: &'a str,
    max_edit_distance: usize,
    allow_transpositions: bool,
) -> impl Iterator<Item = Match> + 'a {
    // Make sure that the pattern is valid. It's a limitation of the bitap
    // algorithm, but we can only search for patterns that have less
    // characters than there are bits in the system's word size.
    let m = pattern.chars().count();
    if m == 0 {
        panic!("empty pattern!");
    } else if m > mem::size_of::<usize>() * 8 - 1 {
        panic!("pattern is too long!");
    }

    // Create a mapping from characters to character masks. A "character's
    // mask" in this case is a bitmask where, for every index that character
    // is used in the pattern string, the value is zero.
    //
    // Roughly if the pattern were "abcab" the character masks would be as
    // follows (albeit reversed, so the first character corresponds to the
    // least significant bit). The remaining bits are all set to 1.
    //
    //        abcab abcab
    //   "a": X..X. 01101
    //   "b": .X..X 10110
    //   "c": ..X.. 11011
    //
    let mut masks: HashMap<char, usize> = HashMap::new();
    for (i, c) in pattern.chars().enumerate() {
        match masks.get_mut(&c) {
            Some(mask) => {
                *mask &= !(1usize << i);
            }
            None => {
                masks.insert(c, !0usize & !(1usize << i));
            }
        };
    }

    let mut r = vec![!1usize; max_edit_distance + 1];
    let mut trans = vec![!1usize, max_edit_distance];
    return text.chars().enumerate().filter_map(move |(i, c)| {
        let letter_mask = match masks.get(&c) {
            Some(mask) => *mask,
            None => !0usize,
        };
        let mut prev_parent = r[0];
        r[0] |= letter_mask;
        r[0] <<= 1;

        for j in 1..=max_edit_distance {
            let prev = r[j];
            let current = (prev | letter_mask) << 1;
            let replace = prev_parent << 1;
            let delete = r[j - 1] << 1;
            let insert = prev_parent;
            let transpose = (trans[j - 1] | (letter_mask << 1)) << 1;
            r[j] = current & insert & delete & replace;
            if allow_transpositions {
                r[j] &= transpose;
            }

            // roughly: the current letter matches the _next_ position in the
            // parent. I couldn't find any reference implementations of bitap
            // that includes transposition, so this may not be correct. But I
            // thought about it for a long time?
            trans[j - 1] = (prev_parent << 1) | letter_mask;

            prev_parent = prev;
        }

        for (k, rv) in r.iter().enumerate() {
            if 0 == (rv & (1usize << m)) {
                return Some(Match {
                    distance: k,
                    end: i,
                });
            }
        }
        None
    });
}

pub fn find<'a>(pattern: &'a str, text: &'a str) -> impl Iterator<Item = usize> + 'a {
    let m = pattern.chars().count();
    return bitap_reference(text, pattern, 0, false).map(
        move |Match {
                  distance: _k,
                  end: i,
              }| i + 1 - m,
    );
}

pub fn levenshtein<'a>(
    pattern: &'a str,
    text: &'a str,
    k: usize,
) -> impl Iterator<Item = Match> + 'a {
    return bitap_reference(text, pattern, k, false);
}

pub fn damerau_levenshtein<'a>(
    pattern: &'a str,
    text: &'a str,
    k: usize,
) -> impl Iterator<Item = Match> + 'a {
    return bitap_reference(text, pattern, k, true);
}

/// BitapFast contains an _optimized_ implementation of bitap searching. It's
/// encapsulated in a struct instead of a single function so that you can
/// amortize the cost of mask creation. Only works for ascii patterns and for
/// searching ascii text. Planning to use this as a benchmark baseline for the
/// main package, just because I don't understand the perf implications of
/// everything I'm doing. Currently used in the benchmarks. This is
/// technically public so that it can be called from benchmarks, but you
/// should never really use it.
#[derive(Copy, Clone)]
pub struct BitapFast {
    pattern_length: usize,
    masks: [usize; 256],
}

impl BitapFast {
    pub fn new(pattern: &str) -> BitapFast {
        if !pattern.is_ascii() {
            panic!("pattern must be ascii");
        }
        let m = pattern.len();
        if m == 0 {
            panic!("empty pattern!");
        } else if m > mem::size_of::<usize>() * 8 - 1 {
            panic!("pattern is too long!");
        }
        let mut s = Self {
            pattern_length: m,
            masks: [!0usize; 256],
        };
        for (i, b) in pattern.bytes().enumerate() {
            let m = unsafe { s.masks.get_unchecked_mut(b as usize) };
            *m &= !(1usize << i);
        }
        return s;
    }

    pub fn find(&self, text: &str) -> Option<usize> {
        let mut r = !1usize;
        for (i, b) in text.bytes().enumerate() {
            unsafe {
                r |= self.masks.get_unchecked(b as usize);
            }
            r <<= 1;
            if 0 == (r & (1usize << self.pattern_length)) {
                return Some(i + 1 - self.pattern_length);
            }
        }
        None
    }

    #[inline]
    pub fn find_iter<'a>(&'a self, text: &'a str) -> impl Iterator<Item = usize> + 'a {
        let mut r = !1usize;
        return text.bytes().enumerate().filter_map(move |(i, b)| {
            unsafe {
                r |= self.masks.get_unchecked(b as usize);
            }
            r <<= 1;
            if 0 == (r & (1usize << self.pattern_length)) {
                Some(i + 1 - self.pattern_length)
            } else {
                None
            }
        });
    }
}
