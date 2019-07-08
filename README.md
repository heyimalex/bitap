# bitap

Implementation of the [bitap algorithm](https://en.wikipedia.org/wiki/Bitap_algorithm) in Rust. The code _may be_ kinda rough because this is some of the first Rust I've written!

## Usage

```rust
use bitap;

fn main() -> Result<(), &'static str>{
  // Compile the pattern you want to search for.
  let pattern = Pattern::new("hello")?;
  // Run one of the search functions.
  let text = "heelo world";
  let max_distance = 1;
  let first_match = pattern
        .lev(&text, max_distance)
        .next()
        .ok_or("no results")?;
  assert_eq!(first_match, Match{distance: 1, end: 4});
  Ok(())
}
```

## Unicode

When you think of "edit distance", you usually think in terms of "characters". But a unicode code-point doesn't map to a single "character". A character could be "a" or it could be "ă̘̙̤̪̹̰͔͒̃̃͐̂͘". A single character could be ten dads. Under the 1-char-per-character rule, "alyx" is technically _two_ edits away from "aléx" (where é is e + &#x301; ) when you really expect it to be one.

This is, for better or worse, how the `Pattern` struct works internally. If you need more nuanced behavior, you're free to use iterator adapters described in the section below. Maybe something with [graphemes](https://docs.rs/unicode-segmentation/1.3.0/unicode_segmentation/). You could also normalize text to remove extraneous decorations, which may be what your users want anyway.

## Adapters

The `Pattern` struct handles most common usages of bitap; fuzzy searching for unicode patterns in unicode text. But it's not perfect, and bitap itself is generalizable to a much broader range of problems.

Luckily, the _delicious caramel core_ of bitap is actually representable in a way that _doesn't care_ about whether you're dealing with code points or graphemes or even nucleotides, and I can punt all those concerns to someone who cares!

They key insight is that `pattern` and `text`, whatever they are, are ultimately coalesced down to an iterator of bit-masks. Bitap can then be implemented as a iterator adapter that takes in `Iterator<Item = usize>` and returns an iterator of matches. That's what the top level `find`, `levenshtein` and `optimal_string_alignment` functions are. You write the code that makes the bit-mask iterator, they find the matches.

### Static Variants

There are a couple of static versions of the iterator adapters, `levenshtein_static` and `optimal_string_alignment_static`. What's that about?

According to [wikipedia](https://en.wikipedia.org/wiki/Damerau%E2%80%93Levenshtein_distance):

> In his seminal paper, Damerau stated that more than 80% of all human misspellings can be expressed by a single error of one of the four types.

Also, I did a lot of toying with [algolia](https://www.algolia.com/) while thinking about fuzzy search, and [they only allow up to two typos](https://www.algolia.com/doc/guides/managing-results/optimize-search-results/typo-tolerance/). So allowing up to two errors is probably a common enough case to optimize for, and we can eek out \~15% more performance by avoiding an allocation!

## Match Highlighting

The algorithm can unfortunately only tell you what index a match _ends_ on. When edit distance is zero, the beginning of the match is trivially `match.end - pattern_length`, but with edits it's `+- match.distance`. This makes _perfectly accurate_ highlighting a pain, but here are some strategies that have worked for me.

- You almost certainly want filter your matches into local-minima; every zero distance match is sandwiched by two one edit matches, those by two edit matches, those by three edit matches, and so on. By filtering out those wrapping matches, you save yourself a lot of work.
- If matches are relatively rare and `max_distance` is low, it's probably fine to use something like the [strsim](https://github.com/dguo/strsim-rs) crate to brute-force the beginning of the match by checking all substrings between and returning when one has the appropriate edit distance.
- Highlighting _around_ insertions, ie "hello" highlighting **hel**x**lo**", is difficult and I haven't come up with an easy way to do it. Just highlight the whole thing and the humans reading it will understand.
- In general, people care more about the _start_ of a match than the end. If you run bitap in _reverse_, with a reversed pattern over a reversed string, `match.end` is actually the beginning! You can then highlight `pattern_length` characters ahead, skipping leading and trailing whitespace, and it's probably good enough.

As I think about this more some it may be added into the crate.

Also, I should note that while _I_ haven't figured out how to recover the beginning of the match from the internal bitap state, that doesn't mean it's impossible. Interested to see if anyone can come up with something!
