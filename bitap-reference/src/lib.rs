use colored::*;
use std::cmp;
use std::collections::HashMap;
use std::mem;

#[cfg(test)]
extern crate quickcheck;
#[cfg(test)]
#[macro_use(quickcheck)]
extern crate quickcheck_macros;

pub mod baseline;

#[cfg(test)]
mod test;

/// Match represents a single match of a pattern in some text.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Match {
    /// The edit distance for this match.
    pub distance: usize,
    /// The index of the character (not necessarily byte) that _ends_ this
    /// match. Determining start position isn't possible (unless
    /// max_distance is zero), so this is all I'm returning.
    pub end: usize,
}

pub type BitapResult = Result<Vec<Match>, &'static str>;
pub type FindResult = Result<Vec<usize>, &'static str>;

static ERR_INVALID_PATTERN: &'static str = "invalid pattern";

fn pattern_length_is_valid(pattern_length: usize) -> bool {
    pattern_length > 0 && pattern_length < mem::size_of::<usize>() * 8
}

/// The reference implementation of the bitap algorithm.
pub fn reference(
    pattern: &str,
    text: &str,
    max_distance: usize,
    allow_transpositions: bool,
    debug: bool,
) -> BitapResult {
    let pattern_len = pattern.chars().count();
    if !pattern_length_is_valid(pattern_len) {
        return Err(ERR_INVALID_PATTERN);
    }

    // Clamp max edit distance to the length of the pattern.
    let max_distance = cmp::min(max_distance, pattern_len);

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
        masks
            .entry(c)
            .and_modify(|mask| *mask &= !(1usize << i))
            .or_insert(!(1usize << i));
    }

    // We need to initialize the state with each error level already having
    // that many characters "correct". Without this, partial matches at the
    // beginning of the text wouldn't work. Basically we want r to look like this:
    //
    //   r[0] ...11111110
    //   r[1] ...11111100
    //   r[2] ...11111000
    //   ...(etc)
    //
    // That's what the kinda opaque one-liner below does.
    let mut r = (0..=max_distance).map(|i| !1usize << i).collect::<Vec<_>>();
    let mut trans = vec![!1usize; max_distance];

    let results = text
        .chars()
        .enumerate()
        .filter_map(move |(i, c)| {
            let letter_mask = match masks.get(&c) {
                Some(mask) => *mask,
                None => !0usize,
            };
            let mut prev_parent = r[0];
            r[0] |= letter_mask;
            r[0] <<= 1;

            for j in 1..=max_distance {
                let prev = r[j];
                let current = (prev | letter_mask) << 1;
                let replace = prev_parent << 1;
                let delete = r[j - 1] << 1;
                let insert = prev_parent;
                r[j] = current & insert & delete & replace;

                if allow_transpositions {
                    let transpose = (trans[j - 1] | (letter_mask << 1)) << 1;
                    r[j] &= transpose;

                    // roughly: the current letter matches the _next_ position in
                    // the parent. I couldn't find any reference implementations
                    // of bitap that includes transposition, so this may not be
                    // correct. But I thought about it for a long time?
                    trans[j - 1] = (prev_parent << 1) | letter_mask;
                }

                prev_parent = prev;
            }

            if debug {
                debug_bitap(pattern, text, i, &r);
            }

            for (k, rv) in r.iter().enumerate() {
                if 0 == (rv & (1usize << pattern_len)) {
                    return Some(Match {
                        distance: k,
                        end: i,
                    });
                }
            }
            None
        })
        .collect::<Vec<_>>();
    Ok(results)
}

// Visualizing bitap is really difficult, so this little helper makes it a bit
// easier by printing the internal state after each iteration. I used this a
// lot during development, but not much anymore.
fn debug_bitap(pattern: &str, text: &str, index: usize, state: &[usize]) {
    let pattern_len = pattern.chars().count();
    let text_colored = text
        .chars()
        .enumerate()
        .map(|(i, c)| {
            if i == index {
                format!("{}", c.to_string().green())
            } else {
                c.to_string()
            }
        })
        .collect::<Vec<String>>()
        .concat();
    println!("{: <4}| {}", index, text_colored);
    println!("-------------------");
    println!("    | {}", pattern);
    println!("-------------------");
    for (i, r) in state.iter().enumerate() {
        let bitmask: String = format!("{:b}", r)
            .chars()
            .rev()
            .skip(1)
            .take(pattern_len)
            .collect();
        let is_match = bitmask.ends_with('0');
        if is_match {
            println!("{}   | {}", i, bitmask.green());
        } else {
            println!("{}   | {}", i, bitmask);
        }
    }
    println!("-------------------");
}

pub fn find(pattern: &str, text: &str) -> FindResult {
    reference(pattern, text, 0, false, false).map(|v| {
        let offset = pattern.chars().count() - 1;
        v.iter().map(|m| m.end - offset).collect::<Vec<_>>()
    })
}

pub fn lev(pattern: &str, text: &str, max_distance: usize) -> BitapResult {
    reference(pattern, text, max_distance, false, false)
}

pub fn osa(pattern: &str, text: &str, max_distance: usize) -> BitapResult {
    reference(pattern, text, max_distance, true, false)
}
