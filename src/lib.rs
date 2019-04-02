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

/// Match represents a single match of a pattern.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Match {
    /// The edit distance for this match. Zero means it was an exact match,
    /// one means a single edit, etc.
    pub distance: usize,
    /// The index that this match ends on. Determining start position isn't
    /// possible (unless max_distance is zero), so this is all I'm returning.
    pub end: usize,
}

static ERR_INVALID_PATTERN: &'static str = "invalid pattern length";

/// Because of bitap's implementation details, patterns can only be as long as
/// the system word size. This is used internally in all of the iterator
/// adapters.
#[inline]
pub fn pattern_length_is_valid(pattern_length: usize) -> bool {
    pattern_length > 0 && pattern_length < mem::size_of::<usize>() * 8
}

pub fn find<I: Iterator<Item = usize>>(
    mask_iter: I,
    pattern_length: usize,
) -> Result<impl Iterator<Item = usize>, &'static str> {
    if !pattern_length_is_valid(pattern_length) {
        return Err(ERR_INVALID_PATTERN);
    }
    let mut r = !1usize;
    let matches = mask_iter.enumerate().filter_map(move |(i, mask)| {
        r |= mask;
        r <<= 1;
        if 0 == (r & (1usize << pattern_length)) {
            return Some(i);
        }
        None
    });
    Ok(matches)
}

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

pub struct Pattern {
    length: usize,
    masks: HashMap<char, usize>,
}

impl Pattern {

    pub fn new(pattern: &str) -> Result<Pattern, &'static str> {
        let mut length = 0;
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

    pub fn find<'a>(&'a self, text: &'a str) -> impl Iterator<Item = usize> + 'a {
        find(self.mask_iter(text), self.len()).unwrap()
    }

    pub fn lev<'a>(&'a self, text: &'a str, k: usize) -> impl Iterator<Item = Match> + 'a {
        levenshtein(self.mask_iter(text), self.len(), k).unwrap()
    }

    pub fn osa<'a>(&'a self, text: &'a str, k: usize) -> impl Iterator<Item = Match> + 'a {
        optimal_string_alignment(self.mask_iter(text), self.len(), k).unwrap()
    }

    pub fn lev_static<'a>(
        &'a self,
        text: &'a str,
        k: StaticMaxDistance,
    ) -> impl Iterator<Item = Match> + 'a {
        levenshtein_static(self.mask_iter(text), self.len(), k).unwrap()
    }

    pub fn osa_static<'a>(
        &'a self,
        text: &'a str,
        k: StaticMaxDistance,
    ) -> impl Iterator<Item = Match> + 'a {
        optimal_string_alignment_static(self.mask_iter(text), self.len(), k).unwrap()
    }
}

pub struct MaskIterator<'a> {
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
