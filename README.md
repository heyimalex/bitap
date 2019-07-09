# Bitap

[![Crates.io](https://img.shields.io/crates/v/bitap.svg)](https://crates.io/crates/bitap)

Implementation of the [bitap algorithm](https://en.wikipedia.org/wiki/Bitap_algorithm) for fuzzy string search in Rust.

If you have a bunch of text to search through and a substring that you're looking for, bitap can efficiently find all places in the source text that are at most _k_ edits away from the search string, where "edits" are in terms of [Levenshtein distance](https://en.wikipedia.org/wiki/Levenshtein_distance) or [optimal string alignment distance](https://en.wikipedia.org/wiki/Damerau%E2%80%93Levenshtein_distance#Optimal_string_alignment_distance). There's a small upfront cost, but the runtime is O(_nk_) in practice, which is pretty solid if that's what you need.

## Usage

Compile a `Pattern` and then use it to search.

```rust
use bitap::{Pattern,Match};

// Compile the pattern you're searching for.
let pattern = Pattern::new("wxrld")?;

// Run one of the search functions:
// - pattern.lev for levenshtein distance matching
// - pattern.osa for optimal string alignment distance matching
// The second parameter determines the maximum edit distance
// that you want to return matches for.
let max_distance = 1;
let matches = pattern.lev("hello world", max_distance);

// Horray!
assert_eq!(matches.next(), Some(Match{ distance: 1, end: 10 }));

```

## Limitations

- Pattern size is limited to system word size (`mem::size_of::<usize>() - 1`), so you can't search for anything longer than 31/63 characters, depending on architecture. This is a fundamental limitation of the algorithm. This _seems_ like a pretty bad limitation, but for fuzzy search at least you're probably going to split up your query into tokens and run bitap _n_ times. `Antidisestablishmentarianism` is only 28 characters after all.

- Bitap can tell you where a match ends, but not where it begins. The section on match highlighting goes into more detail about this.

- Unicode is weird. When you think of "edit distance", you usually think in terms of "characters". But a Unicode code-point doesn't map to a single "character". A character could be "a" or it could be "ă̘̙̤̪̹̰͔͒̃̃͐̂͘". A single character could be ten dads. Under the 1-char-per-character rule, "alyx" is technically _two_ edits away from "aléx" (where é is e + &#x301; ) when you really expect it to be one. The `Pattern` struct works this way internally, where one `char` equals one character. If you need more nuanced behavior, you're free to use iterator adapters described in the section below. You could also normalize text to remove extraneous decorations, which may be what your users want anyway.

## Adapters

The `Pattern` struct handles most common usages of bitap; fuzzy searching for unicode patterns in unicode text. But it's not perfect, and bitap itself is generalizable to a much broader range of problems.

Luckily, the _core_ of bitap is actually representable in a way that _doesn't care_ about whether you're dealing with code points or graphemes or even nucleotides, and I can punt all those concerns to someone who cares!

They key insight is that the main algorithm works on an iterator of pattern masks. Bitap can then be implemented as a iterator adapter that takes in `Iterator<Item = usize>` and returns an iterator of matches. That's what the top level `find`, `levenshtein` and `optimal_string_alignment` functions are; you write the code that makes the pattern-mask iterator, they find the matches.

### Static Variants

There are a couple of static versions of the iterator adapters, `levenshtein_static` and `optimal_string_alignment_static`. What's that about?

According to [wikipedia](https://en.wikipedia.org/wiki/Damerau%E2%80%93Levenshtein_distance):

> In his seminal paper, Damerau stated that more than 80% of all human misspellings can be expressed by a single error of one of the four types.

Also, I did a lot of toying with [algolia](https://www.algolia.com/) while thinking about fuzzy search, and [they only allow up to two typos](https://www.algolia.com/doc/guides/managing-results/optimize-search-results/typo-tolerance/). So allowing up to two errors is probably a common enough case to optimize for, and we can eek out \~15% more performance by avoiding an allocation!

## Match Highlighting

Bitap can unfortunately only tell you what index a match _ends_ on. When edit distance is zero, the beginning of the match is trivially `match.end - pattern_length + 1`, but with edits it's that `+- match.distance`. This makes _perfectly accurate_ highlighting a pain, but here are some strategies that have worked for me.

- You almost certainly want filter your matches into local-minima; every zero distance match is sandwiched by two one edit matches, those by two edit matches, those by three edit matches, and so on. By filtering out those wrapping matches, you save yourself a lot of work.

- If matches are relatively rare and `max_distance` is low, it's probably fine to use something like the [strsim](https://github.com/dguo/strsim-rs) crate to brute-force the beginning of the match by checking all substrings between and returning when one has the appropriate edit distance.

- Highlighting _around_ insertions, ie "hello" highlighting "**hel**x**lo**", is difficult and I haven't come up with an easy way to do it. Just highlight the whole thing and the humans reading it will understand.

- In general, people care more about the _start_ of a match than the end. If you run bitap in _reverse_, with a reversed pattern over a reversed string, `match.end` is actually the beginning! You can then highlight `pattern_length` characters ahead, skipping leading and trailing whitespace, and it's probably good enough.

As I think about this more some it may be added into the crate.

Also, I should note that while _I_ haven't figured out how to recover the beginning of the match from the internal bitap state, that doesn't mean it's impossible. Interested to see if anyone can come up with something!
