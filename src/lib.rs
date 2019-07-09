use std::cmp;
use std::collections::HashMap;
use std::mem;

#[cfg(test)]
extern crate quickcheck;
#[cfg(test)]
#[macro_use(quickcheck)]
extern crate quickcheck_macros;

#[cfg(test)]
#[macro_use]
extern crate lazy_static;

#[cfg(test)]
mod test;

/// Match represents a single match of a pattern within a string.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Match {
    /// The edit distance for this match. Zero means it was an exact match,
    /// one means a single edit, etc.
    pub distance: usize,
    /// The index that this match _ends_ on. Determining start position isn't
    /// possible (unless `distance` is zero), so this is all you have access to.
    pub end: usize,
}

static ERR_INVALID_PATTERN: &'static str = "invalid pattern length";

/// Returns whether the passed value is a valid pattern length.
///
/// Because of implementation details of the bitap algorithm itself, patterns
/// can only be as long as the system word size minus one. That's 31/63
/// depending on the architecture you're compiling for. Additionally, patterns
/// with a length of zero are rejected.
#[inline]
pub fn pattern_length_is_valid(pattern_length: usize) -> bool {
    pattern_length > 0 && pattern_length < mem::size_of::<usize>() * 8
}

/// Iterator adapter for implementing bitap find over an iterator of pattern
/// masks.
pub fn find<I: Iterator<Item = usize>>(
    mask_iter: I,
    pattern_length: usize,
) -> Result<impl Iterator<Item = usize>, &'static str> {
    if !pattern_length_is_valid(pattern_length) {
        return Err(ERR_INVALID_PATTERN);
    }
    // In find, unlike the other functions, we want to return the _start_ index of the
    // matches because it's actually possible to recover.
    let offset = pattern_length - 1;
    let mut r = !1usize;
    let matches = mask_iter.enumerate().filter_map(move |(i, mask)| {
        r |= mask;
        r <<= 1;
        if 0 == (r & (1usize << pattern_length)) {
            return Some(i - offset);
        }
        None
    });
    Ok(matches)
}

/// Iterator adapter for implementing bitap for levenshtein distance over an
/// iterator of pattern masks.
pub fn levenshtein<I: Iterator<Item = usize>>(
    mask_iter: I,
    pattern_length: usize,
    max_distance: usize,
) -> Result<impl Iterator<Item = Match>, &'static str> {
    if !pattern_length_is_valid(pattern_length) {
        return Err(ERR_INVALID_PATTERN);
    }
    let max_distance = cmp::min(max_distance, pattern_length);
    let mut r: Vec<usize> = (0..=max_distance).map(|i| !1usize << i).collect();

    let matches = mask_iter.enumerate().filter_map(move |(i, mask)| {
        let mut prev_parent = r[0];
        r[0] |= mask;
        r[0] <<= 1;
        for j in 1..r.len() {
            let prev = r[j];
            let current = (prev | mask) << 1;
            let replace = prev_parent << 1;
            let delete = r[j - 1] << 1;
            let insert = prev_parent;
            r[j] = current & insert & delete & replace;
            prev_parent = prev;
        }
        for (k, rv) in r.iter().enumerate() {
            if 0 == (rv & (1usize << pattern_length)) {
                return Some(Match {
                    distance: k,
                    end: i,
                });
            }
        }
        None
    });
    Ok(matches)
}

/// Iterator adapter for implementing bitap for optimal string alignment
/// distance over an iterator of pattern masks.
pub fn optimal_string_alignment<I: Iterator<Item = usize>>(
    mask_iter: I,
    pattern_length: usize,
    max_distance: usize,
) -> Result<impl Iterator<Item = Match>, &'static str> {
    if !pattern_length_is_valid(pattern_length) {
        return Err(ERR_INVALID_PATTERN);
    }
    let max_distance = cmp::min(max_distance, pattern_length);
    let mut r: Vec<usize> = (0..=max_distance).map(|i| !1usize << i).collect();
    let mut t = vec![!1usize; max_distance];

    let matches = mask_iter.enumerate().filter_map(move |(i, mask)| {
        let mut prev_parent = r[0];
        r[0] |= mask;
        r[0] <<= 1;
        for j in 1..r.len() {
            let prev = r[j];
            let current = (prev | mask) << 1;
            let replace = prev_parent << 1;
            let delete = r[j - 1] << 1;
            let insert = prev_parent;
            let transpose = (t[j - 1] | (mask << 1)) << 1;
            r[j] = current & insert & delete & replace & transpose;
            t[j - 1] = (prev_parent << 1) | mask;
            prev_parent = prev;
        }
        for (k, rv) in r.iter().enumerate() {
            if 0 == (rv & (1usize << pattern_length)) {
                return Some(Match {
                    distance: k,
                    end: i,
                });
            }
        }
        None
    });
    Ok(matches)
}

pub enum StaticMaxDistance {
    One = 1,
    Two = 2,
}

/// Like the levenshtein iterator adapter, but optimized for max_distances of
/// 1-2.
pub fn levenshtein_static<I: Iterator<Item = usize>>(
    mask_iter: I,
    pattern_length: usize,
    max_distance: StaticMaxDistance,
) -> Result<impl Iterator<Item = Match>, &'static str> {
    if !pattern_length_is_valid(pattern_length) {
        return Err(ERR_INVALID_PATTERN);
    }
    let max_distance = cmp::min(max_distance as usize, pattern_length);
    let mut r = [!1usize, !1usize << 1, !1usize << 2];

    let matches = mask_iter.enumerate().filter_map(move |(i, mask)| {
        let mut prev_parent = r[0];
        r[0] |= mask;
        r[0] <<= 1;
        for j in (1..r.len()).take(max_distance) {
            let prev = r[j];
            let current = (prev | mask) << 1;
            let replace = prev_parent << 1;
            let delete = r[j - 1] << 1;
            let insert = prev_parent;
            r[j] = current & insert & delete & replace;
            prev_parent = prev;
        }
        for (k, rv) in r.iter().take(max_distance + 1).enumerate() {
            if 0 == (rv & (1usize << pattern_length)) {
                return Some(Match {
                    distance: k,
                    end: i,
                });
            }
        }
        None
    });
    Ok(matches)
}

/// Like the optimal_string_alignment iterator adapter, but optimized for
/// max_distances of 1-2.
pub fn optimal_string_alignment_static<I: Iterator<Item = usize>>(
    mask_iter: I,
    pattern_length: usize,
    max_distance: StaticMaxDistance,
) -> Result<impl Iterator<Item = Match>, &'static str> {
    if !pattern_length_is_valid(pattern_length) {
        return Err(ERR_INVALID_PATTERN);
    }
    let max_distance = cmp::min(max_distance as usize, pattern_length);
    let mut r = [!1usize, !1usize << 1, !1usize << 2];
    let mut t = [!1usize, !1usize];

    let matches = mask_iter.enumerate().filter_map(move |(i, mask)| {
        let mut prev_parent = r[0];
        r[0] |= mask;
        r[0] <<= 1;
        for j in (1..r.len()).take(max_distance) {
            let prev = r[j];
            let current = (prev | mask) << 1;
            let replace = prev_parent << 1;
            let delete = r[j - 1] << 1;
            let insert = prev_parent;
            let transpose = (t[j - 1] | (mask << 1)) << 1;
            r[j] = current & insert & delete & replace & transpose;
            t[j - 1] = (prev_parent << 1) | mask;
            prev_parent = prev;
        }
        for (k, rv) in r.iter().take(max_distance + 1).enumerate() {
            if 0 == (rv & (1usize << pattern_length)) {
                return Some(Match {
                    distance: k,
                    end: i,
                });
            }
        }
        None
    });
    Ok(matches)
}

/// A compiled pattern string that can be used to search text.
pub struct Pattern {
    length: usize,
    masks: HashMap<char, usize>,
}

impl Pattern {
    /// Compiles and returns a new pattern from the passed string. Will fail
    /// if the passed pattern is empty or longer than the system word size.
    pub fn new(pattern: &str) -> Result<Pattern, &'static str> {
        let mut length = 0;
        // Create a mapping from characters to character masks. A "character's
        // mask" in this case is a bitmask where, for every index that
        // character is used in the pattern string, the value is zero.
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
            length += 1;
            masks
                .entry(c)
                .and_modify(|mask| *mask &= !(1usize << i))
                .or_insert(!(1usize << i));
        }
        if !pattern_length_is_valid(length) {
            return Err(ERR_INVALID_PATTERN);
        }
        Ok(Pattern { length, masks })
    }

    /// Returns the length of the pattern in characters.
    #[inline]
    pub fn len(&self) -> usize {
        self.length
    }

    #[inline]
    fn mask_iter<'a>(&'a self, text: &'a str) -> MaskIterator<'a> {
        MaskIterator {
            masks: &self.masks,
            iter: text.chars(),
        }
    }

    /// Returns an iterator of character indexes where the pattern can be found
    /// within the passed text.
    ///
    /// Unlike `str::matches`, it will find and return overlapping matches.
    ///
    /// ```
    /// use bitap::{Pattern};
    /// let pattern = Pattern::new("world")?;
    /// assert_eq!(pattern.find("hello world").next(), Some(6));
    /// # Ok::<(), &'static str>(())
    /// ```
    pub fn find<'a>(&'a self, text: &'a str) -> impl Iterator<Item = usize> + 'a {
        find(self.mask_iter(text), self.len()).unwrap()
    }

    /// Returns an iterator of matches where the pattern matched the passed
    /// text within a levenshtein distance of `max_distance`.
    ///
    /// ```
    /// use bitap::{Pattern,Match};
    /// let pattern = Pattern::new("wxrld")?;
    /// let m = pattern.lev("hello world", 1).next();
    /// assert_eq!(m, Some(Match{ distance: 1, end: 10 }));
    /// # Ok::<(), &'static str>(())
    /// ```
    pub fn lev<'a>(
        &'a self,
        text: &'a str,
        max_distance: usize,
    ) -> impl Iterator<Item = Match> + 'a {
        levenshtein(self.mask_iter(text), self.len(), max_distance).unwrap()
    }

    /// Returns an iterator of matches where the pattern matched the passed
    /// text within an optimal string alignment distance of `max_distance`.
    ///
    /// ```
    /// use bitap::{Pattern,Match};
    /// let pattern = Pattern::new("wrold")?;
    /// let m = pattern.osa("hello world", 1).next();
    /// assert_eq!(m, Some(Match{ distance: 1, end: 10 }));
    /// # Ok::<(), &'static str>(())
    /// ```
    pub fn osa<'a>(
        &'a self,
        text: &'a str,
        max_distance: usize,
    ) -> impl Iterator<Item = Match> + 'a {
        optimal_string_alignment(self.mask_iter(text), self.len(), max_distance).unwrap()
    }

    /// The same as lev, but optimized for a `max_distance` of 1-2.
    pub fn lev_static<'a>(
        &'a self,
        text: &'a str,
        max_distance: StaticMaxDistance,
    ) -> impl Iterator<Item = Match> + 'a {
        levenshtein_static(self.mask_iter(text), self.len(), max_distance).unwrap()
    }

    /// The same as osa, but optimized for a `max_distance` of 1-2.
    pub fn osa_static<'a>(
        &'a self,
        text: &'a str,
        max_distance: StaticMaxDistance,
    ) -> impl Iterator<Item = Match> + 'a {
        optimal_string_alignment_static(self.mask_iter(text), self.len(), max_distance).unwrap()
    }
}

/// Combines the mask map and an iterator of chars into a stream of pattern masks.
struct MaskIterator<'a> {
    masks: &'a HashMap<char, usize>,
    iter: std::str::Chars<'a>,
}

impl<'a> Iterator for MaskIterator<'a> {
    type Item = usize;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|c| match self.masks.get(&c) {
            Some(m) => *m,
            None => !0usize,
        })
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}
