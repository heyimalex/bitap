use super::*;
use quickcheck::TestResult;

extern crate bitap_reference as bref;

fn find_test(ctx: &str, p: &str, t: &str) {
    let base = bref::find(p, t).unwrap();
    let actual = Pattern::new(p).unwrap().find(t).collect::<Vec<_>>();
    assert_eq!(base, actual, "{}: find({:?}, {:?})", ctx, p, t);
}

fn try_static_max_distance(k: usize) -> Option<StaticMaxDistance> {
    match k {
        1 => Some(StaticMaxDistance::One),
        2 => Some(StaticMaxDistance::Two),
        _ => None,
    }
}

fn levenshtein_test(ctx: &str, p: &str, t: &str, k: usize) {
    let base = ref_result_convert(bref::lev(p, t, k)).unwrap();
    let actual = Pattern::new(p).unwrap().lev(t, k).collect::<Vec<_>>();
    assert_eq!(base, actual, "{}: lev({:?}, {:?}, {})", ctx, p, t, k);
    if let Some(d) = try_static_max_distance(k) {
        let actual_static = Pattern::new(p)
            .unwrap()
            .lev_static(t, d)
            .collect::<Vec<_>>();
        assert_eq!(
            base, actual_static,
            "{}: lev_static({:?}, {:?}, {})",
            ctx, p, t, k
        );
    }
}

fn optimal_string_alignment_test(ctx: &str, p: &str, t: &str, k: usize) {
    let base = ref_result_convert(bref::osa(p, t, k)).unwrap();
    let actual = Pattern::new(p).unwrap().osa(t, k).collect::<Vec<_>>();
    assert_eq!(base, actual, "{}: osa({:?}, {:?}, {})", ctx, p, t, k);
    if let Some(d) = try_static_max_distance(k) {
        let actual_static = Pattern::new(p)
            .unwrap()
            .osa_static(t, d)
            .collect::<Vec<_>>();
        assert_eq!(
            base, actual_static,
            "{}: osa_static({:?}, {:?}, {})",
            ctx, p, t, k
        );
    }
}

lazy_static! {
    static ref CORPUS: Vec<(&'static str, &'static str)> = {
        vec![
            ("alex", "hey im alex, how are you?"),
            ("aba", "abababababa"),
            ("alex", "aelx"),
            ("abcde", "bcde abde abccde abzde abdce"),
            ("abac", "acb"),
        ]
    };
}

#[test]
fn test_find() {
    for (i, (p, t)) in CORPUS.iter().enumerate() {
        let ctx = format!("case {}", i);
        find_test(&ctx, p, t);
    }
}

#[test]
fn test_levenshtein() {
    for (i, (p, t)) in CORPUS.iter().enumerate() {
        let ctx = format!("case {}", i);
        let max_k = p.chars().count() + 2; // +2 for good measure
        for k in 0..=max_k {
            levenshtein_test(&ctx, p, t, k);
        }
    }
}

#[test]
fn test_optimal_string_alignment() {
    for (i, (p, t)) in CORPUS.iter().enumerate() {
        let ctx = format!("case {}", i);
        let max_k = p.chars().count() + 2; // +2 for good measure
        for k in 0..=max_k {
            optimal_string_alignment_test(&ctx, p, t, k);
        }
    }
}

#[quickcheck]
fn qc_find(pattern: String, text: String) -> TestResult {
    if !pattern_length_is_valid(pattern.chars().count()) {
        return TestResult::discard();
    }
    let a = Pattern::new(&pattern)
        .unwrap()
        .find(&text)
        .collect::<Vec<_>>();
    let b = bref::find(&pattern, &text).unwrap();
    TestResult::from_bool(a == b)
}

#[quickcheck]
fn qc_lev(pattern: String, text: String, k: usize) -> TestResult {
    if !pattern_length_is_valid(pattern.chars().count()) {
        return TestResult::discard();
    }
    let a = Pattern::new(&pattern)
        .unwrap()
        .lev(&text, k)
        .collect::<Vec<_>>();
    let b = ref_result_convert(bref::lev(&pattern, &text, k)).unwrap();
    TestResult::from_bool(a == b)
}

#[quickcheck]
fn qc_osa(pattern: String, text: String, k: usize) -> TestResult {
    if !pattern_length_is_valid(pattern.chars().count()) {
        return TestResult::discard();
    }
    let a = Pattern::new(&pattern)
        .unwrap()
        .osa(&text, k)
        .collect::<Vec<_>>();
    let b = ref_result_convert(bref::osa(&pattern, &text, k)).unwrap();
    TestResult::from_bool(a == b)
}

fn ref_result_convert(r: bref::BitapResult) -> Result<Vec<Match>, &'static str> {
    r.map(|v| {
        v.into_iter()
            .map(|m| Match {
                distance: m.distance,
                end: m.end,
            })
            .collect::<Vec<_>>()
    })
}
