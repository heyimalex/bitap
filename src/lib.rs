pub mod reference;

use std::collections::HashMap;
use std::mem;

/// Match represents a single match of a pattern in a haystack.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Match {
    /// Distance is the edit distance for this match.
    pub distance: usize,
    /// End is the index of the character (not necessarily byte) that _ends_
    /// this match. Determining start position, nevermind the actual indexes
    /// and byte offsets of the matching characters, is much more complex,
    /// so for now this is all I'm returning.
    pub end: usize,
}

/// UnicodePattern represents a Unicode search string that's compiled to
/// search through other Unicode text.
pub struct UnicodePattern {
    length: usize,
    masks: HashMap<char, usize>,
}

impl UnicodePattern {
    /// Compiles the search pattern. An error will be returned if the pattern
    /// is empty, or if the pattern is longer than the system word size minus
    /// one.
    pub fn new(pattern: &str) -> Result<UnicodePattern, &'static str> {
        let mut length = 0;
        let mut masks: HashMap<char, usize> = HashMap::new();
        for (i, c) in pattern.chars().enumerate() {
            length += 1;
            match masks.get_mut(&c) {
                Some(mask) => {
                    *mask &= !(1usize << i);
                }
                None => {
                    masks.insert(c, !0usize & !(1usize << i));
                }
            };
        }
        if length == 0 {
            return Err("pattern must not be empty");
        }
        if length >= mem::size_of::<usize>() * 8 - 1 {
            return Err("invalid pattern length");
        }
        return Ok(UnicodePattern { length, masks });
    }
}

// AsciiPattern represents an ASCII search string that's compiled to search
// through Unicode text.
pub struct AsciiPattern {
    length: usize,
    masks: [usize; 256],
}

impl AsciiPattern {
    /// Compiles the search pattern. An error will be returned if the pattern
    /// is empty, the pattern contains non-ascii characters, or if the pattern
    /// is longer than the system word size minus one.
    pub fn new(pattern: &str) -> Result<AsciiPattern, &'static str> {
        if pattern.len() == 0 {
            return Err("pattern must not be empty");
        }
        if pattern.len() >= mem::size_of::<usize>() * 8 - 1 {
            return Err("invalid pattern length");
        }
        if !pattern.is_ascii() {
            return Err("pattern must be all ascii characters");
        }
        return Ok(Self::new_unchecked(pattern));
    }
    fn new_unchecked(pattern: &str) -> AsciiPattern {
        let mut m = AsciiPattern {
            length: pattern.len(),
            masks: [!0usize; 256],
        };
        for (i, b) in pattern.bytes().enumerate() {
            m.masks[b as usize] &= !(1usize << i);
        }
        return m;
    }

    // Converts this to an AsciiOnlyPattern, which can only be used to search
    // ascii text.
    pub fn to_ascii_only(self) -> AsciiOnlyPattern {
        AsciiOnlyPattern(self)
    }
}

pub struct UnicodeMaskIterator<'a> {
    pattern: &'a UnicodePattern,
    iter: std::str::Chars<'a>,
}

impl<'a> Iterator for UnicodeMaskIterator<'a> {
    type Item = usize;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|c| match self.pattern.masks.get(&c) {
            Some(m) => *m,
            None => !0usize,
        })
    }
}

pub struct AsciiMaskIterator<'a> {
    pattern: &'a AsciiPattern,
    iter: std::str::Chars<'a>,
}

impl<'a> Iterator for AsciiMaskIterator<'a> {
    type Item = usize;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|c| {
            if !c.is_ascii() {
                return !0usize;
            }
            // Truncate to u8. This is taken from std::char::encode_utf8_raw;
            // apparently the lsb of a char is ascii. Hopefully this doesn't
            // change in future releases :)
            let b = c as u8;
            return self.pattern.masks[b as usize];
        })
    }
}

impl<'a> Searcher<'a> for UnicodePattern {
    type MaskIter = UnicodeMaskIterator<'a>;

    #[inline]
    fn len(&self) -> usize {
        self.length
    }

    #[inline]
    fn get_mask_iter(&'a self, text: &'a str) -> Self::MaskIter {
        UnicodeMaskIterator {
            pattern: self,
            iter: text.chars(),
        }
    }
}

pub trait Searcher<'a> {
    type MaskIter: Iterator<Item = usize>;

    fn len(&self) -> usize;
    fn get_mask_iter(&'a self, text: &'a str) -> Self::MaskIter;

    #[inline]
    fn find_iter(&'a self, text: &'a str) -> BitapFind<Self::MaskIter> {
        BitapFind::new(self.get_mask_iter(text), self.len())
    }
    fn find(&'a self, text: &'a str) -> Option<usize> {
        self.find_iter(text).next()
    }

    #[inline]
    fn find_levenshtein_iter(
        &'a self,
        text: &'a str,
        k: usize,
    ) -> BitapLevenshtein<Self::MaskIter> {
        BitapLevenshtein::new(self.get_mask_iter(text), self.len(), k)
    }
    fn find_levenshtein(&'a self, text: &'a str, k: usize) -> Option<Match> {
        self.find_levenshtein_iter(text, k).next()
    }

    #[inline]
    fn find_damerau_levenshtein_iter(
        &'a self,
        text: &'a str,
        k: usize,
    ) -> BitapDamerauLevenshtein<Self::MaskIter> {
        BitapDamerauLevenshtein::new(self.get_mask_iter(text), self.len(), k)
    }
    fn find_damerau_levenshtein(&'a self, text: &'a str, k: usize) -> Option<Match> {
        self.find_damerau_levenshtein_iter(text, k).next()
    }
}

impl<'a> Searcher<'a> for AsciiPattern {
    type MaskIter = AsciiMaskIterator<'a>;

    #[inline]
    fn len(&self) -> usize {
        self.length
    }

    #[inline]
    fn get_mask_iter(&'a self, text: &'a str) -> Self::MaskIter {
        AsciiMaskIterator {
            pattern: self,
            iter: text.chars(),
        }
    }
}

pub struct AsciiOnlyMaskIterator<'a> {
    pattern: &'a AsciiPattern,
    iter: std::str::Bytes<'a>,
}

impl<'a> Iterator for AsciiOnlyMaskIterator<'a> {
    type Item = usize;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|b| self.pattern.masks[b as usize])
    }
}

/// Like AsciiPattern, but can only be used to search ascii text. Using it to
/// search through Unicode text will result in results that are incorrect.
pub struct AsciiOnlyPattern(AsciiPattern);

impl<'a> Searcher<'a> for AsciiOnlyPattern {
    type MaskIter = AsciiOnlyMaskIterator<'a>;

    #[inline]
    fn len(&self) -> usize {
        self.0.length
    }

    #[inline]
    fn get_mask_iter(&'a self, text: &'a str) -> Self::MaskIter {
        AsciiOnlyMaskIterator {
            pattern: &self.0,
            iter: text.bytes(),
        }
    }
}

// Following this point are the core functions. They're implemented as iterator
// adaptors that take an iterator of character masks and return an iterator of
// matches.

pub struct BitapFind<I> {
    iter: std::iter::Enumerate<I>,
    pattern_length: usize,
    r: usize,
}

impl<I: Iterator<Item = usize>> BitapFind<I> {
    #[inline]
    fn new(mask_iter: I, pattern_length: usize) -> BitapFind<I> {
        BitapFind {
            iter: mask_iter.enumerate(),
            pattern_length,
            r: !1usize,
        }
    }
}

impl<I: Iterator<Item = usize>> Iterator for BitapFind<I> {
    type Item = usize;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        for (i, mask) in self.iter.by_ref() {
            self.r |= mask;
            self.r <<= 1;
            if 0 == (self.r & (1usize << self.pattern_length)) {
                return Some(i + 1 - self.pattern_length);
            }
        }
        None
    }
}

pub struct BitapLevenshtein<I> {
    iter: std::iter::Enumerate<I>,
    pattern_length: usize,
    r: Vec<usize>,
}

impl<I: Iterator<Item = usize>> BitapLevenshtein<I> {
    #[inline]
    fn new(mask_iter: I, pattern_length: usize, max_distance: usize) -> BitapLevenshtein<I> {
        BitapLevenshtein {
            iter: mask_iter.enumerate(),
            pattern_length,
            r: vec![!1usize; max_distance + 1],
        }
    }
}

impl<I: Iterator<Item = usize>> Iterator for BitapLevenshtein<I> {
    type Item = Match;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        for (i, mask) in self.iter.by_ref() {
            let mut prev_parent = self.r[0];
            self.r[0] |= mask;
            self.r[0] <<= 1;
            for j in 1..self.r.len() {
                let prev = self.r[j];
                let current = (prev | mask) << 1;
                let replace = prev_parent << 1;
                let delete = self.r[j - 1] << 1;
                let insert = prev_parent;
                self.r[j] = current & insert & delete & replace;
                prev_parent = prev;
            }
            for (k, rv) in self.r.iter().enumerate() {
                if 0 == (rv & (1usize << self.pattern_length)) {
                    return Some(Match {
                        distance: k,
                        end: i,
                    });
                }
            }
        }
        None
    }
}

pub struct BitapDamerauLevenshtein<I> {
    iter: std::iter::Enumerate<I>,
    pattern_length: usize,
    r: Vec<usize>,
    trans: Vec<usize>,
}

impl<I: Iterator<Item = usize>> BitapDamerauLevenshtein<I> {
    #[inline]
    fn new(mask_iter: I, pattern_length: usize, max_distance: usize) -> BitapDamerauLevenshtein<I> {
        BitapDamerauLevenshtein {
            iter: mask_iter.enumerate(),
            pattern_length,
            r: vec![!1usize; max_distance + 1],
            trans: vec![!1usize; max_distance],
        }
    }
}

impl<I: Iterator<Item = usize>> Iterator for BitapDamerauLevenshtein<I> {
    type Item = Match;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        for (i, mask) in self.iter.by_ref() {
            let mut prev_parent = self.r[0];
            self.r[0] |= mask;
            self.r[0] <<= 1;
            for j in 1..self.r.len() {
                let prev = self.r[j];
                let current = (prev | mask) << 1;
                let replace = prev_parent << 1;
                let delete = self.r[j - 1] << 1;
                let insert = prev_parent;
                let transpose = (self.trans[j - 1] | (mask << 1)) << 1;
                self.r[j] = current & insert & delete & replace & transpose;
                // roughly: the current letter matches the _next_ position in the parent.
                self.trans[j - 1] = (prev_parent << 1) | mask;
                prev_parent = prev;
            }
            for (k, rv) in self.r.iter().enumerate() {
                if 0 == (rv & (1usize << self.pattern_length)) {
                    return Some(Match {
                        distance: k,
                        end: i,
                    });
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::reference;
    use super::*;

    #[test]
    fn test_find() {
        let funcs: Vec<Box<Fn(&str, &str) -> Vec<usize>>> = vec![
            Box::new(|pattern, text| reference::find(pattern, text).collect()),
            Box::new(|pattern, text| reference::BitapFast::new(pattern).find_iter(text).collect()),
            Box::new(|pattern, text| {
                AsciiPattern::new(pattern)
                    .unwrap()
                    .find_iter(text)
                    .collect()
            }),
            Box::new(|pattern, text| {
                AsciiPattern::new(pattern)
                    .unwrap()
                    .to_ascii_only()
                    .find_iter(text)
                    .collect()
            }),
            Box::new(|pattern, text| {
                UnicodePattern::new(pattern)
                    .unwrap()
                    .find_iter(text)
                    .collect()
            }),
        ];

        let cases: Vec<(&str, &str, Vec<usize>)> = vec![
            ("abba", "hey im abba", vec![7]),
            ("alex", "alex alex alex", vec![0, 5, 10]),
            ("alex", "nothing to match", vec![]),
        ];

        for case in cases.iter() {
            for func in funcs.iter() {
                assert_eq!(func(case.0, case.1), case.2);
            }
        }
    }

    #[test]
    fn test_levenshtein() {
        let funcs: Vec<Box<Fn(&str, &str, usize) -> Vec<Match>>> = vec![
            Box::new(|pattern, text, k| reference::levenshtein(pattern, text, k).collect()),
            Box::new(|pattern, text, k| {
                AsciiPattern::new(pattern)
                    .unwrap()
                    .find_levenshtein_iter(text, k)
                    .collect()
            }),
            Box::new(|pattern, text, k| {
                AsciiPattern::new(pattern)
                    .unwrap()
                    .to_ascii_only()
                    .find_levenshtein_iter(text, k)
                    .collect()
            }),
            Box::new(|pattern, text, k| {
                UnicodePattern::new(pattern)
                    .unwrap()
                    .find_levenshtein_iter(text, k)
                    .collect()
            }),
        ];

        let cases: Vec<(&str, &str, usize, Vec<_>)> = vec![(
            "alex",
            "hey im aelx",
            2,
            vec![
                Match {
                    distance: 2,
                    end: 8,
                },
                Match {
                    distance: 2,
                    end: 9,
                },
                Match {
                    distance: 2,
                    end: 10,
                },
            ],
        )];

        for case in cases.iter() {
            for func in funcs.iter() {
                assert_eq!(func(case.0, case.1, case.2), case.3);
            }
        }
    }

    #[test]
    fn test_damerau_levenshtein() {
        let funcs: Vec<Box<Fn(&str, &str, usize) -> Vec<Match>>> = vec![
            Box::new(|pattern, text, k| reference::damerau_levenshtein(pattern, text, k).collect()),
            Box::new(|pattern, text, k| {
                AsciiPattern::new(pattern)
                    .unwrap()
                    .find_damerau_levenshtein_iter(text, k)
                    .collect()
            }),
            Box::new(|pattern, text, k| {
                AsciiPattern::new(pattern)
                    .unwrap()
                    .to_ascii_only()
                    .find_damerau_levenshtein_iter(text, k)
                    .collect()
            }),
            Box::new(|pattern, text, k| {
                UnicodePattern::new(pattern)
                    .unwrap()
                    .find_damerau_levenshtein_iter(text, k)
                    .collect()
            }),
        ];

        let cases: Vec<(&str, &str, usize, Vec<_>)> = vec![(
            "alex",
            "hey im aelx",
            2,
            vec![
                Match {
                    distance: 2,
                    end: 8,
                },
                Match {
                    distance: 2,
                    end: 9,
                },
                Match {
                    distance: 1,
                    end: 10,
                },
            ],
        )];

        for case in cases.iter() {
            for func in funcs.iter() {
                assert_eq!(func(case.0, case.1, case.2), case.3);
            }
        }
    }
}
