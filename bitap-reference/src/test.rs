use super::*;
use quickcheck::TestResult;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TestCase {
    pub pattern: String,
    pub text: String,
    pub max_distance: usize,
    pub expected_matches: Vec<(usize, usize)>,
}

impl TestCase {
    pub fn new(pattern: &str, text: &str, match_str: &str, distance: usize) -> TestCase {
        // Make sure the match_str is the exact same length as the text length.
        let match_str_len = match_str.chars().count();
        let text_len = text.chars().count();
        if match_str_len != text_len {
            panic!("invalid match string format: text and match string were different lengths");
        }

        // Make sure the match str is all spaces or numbers.
        if !match_str.chars().all(|c| c == ' ' || c.is_ascii_digit()) {
            panic!(
                "invalid match string format: contained characters that were not numbers or spaces"
            );
        }

        TestCase {
            pattern: pattern.to_string(),
            text: text.to_string(),
            max_distance: distance,
            expected_matches: match_str_to_matches(match_str),
        }
    }
}

// Helper that makes creating test cases a little easier. It takes a string of
// digits and spaces and converts it roughly like this:
//
// "7 8 9" => vec![(7,0),(8,2),(9,4)]
//                  ^ ^
//                  | |
//                  | index of the original char
//                  char converted to usize
//
// It lets you write the expected matches directly below the text to search,
// and is generally easier than manually writing (usize,usize) pairs. The only
// downside to this is it can't represent error levels above 9, but that
// should be rare rare.
fn match_str_to_matches(s: &str) -> Vec<(usize, usize)> {
    s.chars()
        .enumerate()
        .filter_map(|(i, c)| match c.to_digit(10) {
            Some(d) => Some((d as usize, i)),
            None => None,
        })
        .collect()
}

#[test]
fn test_match_str_to_matches() {
    let cases = vec![
        ("1 1 1 1", vec![(1, 0), (1, 2), (1, 4), (1, 6)]),
        ("1 2 3 4", vec![(1, 0), (2, 2), (3, 4), (4, 6)]),
        (" 1 2 3 4 ", vec![(1, 1), (2, 3), (3, 5), (4, 7)]),
    ];
    for case in cases.iter() {
        let actual = match_str_to_matches(case.0);
        assert_eq!(actual, case.1);
    }
}

type FindImpl = fn(&str, &str) -> Result<Vec<usize>, &'static str>;

#[test]
fn test_find() {
    let impls: Vec<FindImpl> = vec![find, baseline::find];
    #[rustfmt::skip]
    let cases: Vec<TestCase> = vec![
        TestCase::new(
            "alex",
            "hey im alex, how are you?",
            "       0                 ",
            0,
        ),
        TestCase::new(
            "aba",
            "abababababa",
            "0 0 0 0 0  ",
            0,
        ),
        TestCase::new(
            "alex",
            "hey im alex",
            "       0   ",
            2,
        ),
        TestCase::new(
            "alex",
            "hey im aelx",
            "           ",
            2,
        ),
        TestCase::new(
            "abcde",
            "bcde abde abccde abzde abdce",
            "                            ",
            1,
        ),
        TestCase::new(
            "abcde",
            "bcde",
            "    ",
            1,
        ),
    ];

    for f in impls.iter() {
        for case in cases.iter() {
            let result = f(&case.pattern, &case.text).unwrap();
            assert_eq!(
                result,
                case.expected_matches
                    .iter()
                    .filter_map(|m| if m.0 == 0 { Some(m.1) } else { None })
                    .collect::<Vec<_>>(),
                "pattern: {:?}, text: {:?}",
                case.pattern,
                case.text,
            );
        }
    }
}

type BitapImpl = fn(&str, &str, usize) -> BitapResult;

#[test]
fn test_levenshtein() {
    let impls: Vec<BitapImpl> = vec![lev, baseline::lev];
    #[rustfmt::skip]
    let cases: Vec<TestCase> = vec![
        TestCase::new(
            "alex",
            "hey im alex, how are you?",
            "          0              ",
            0,
        ),
        TestCase::new(
            "aba",
            "abababababa",
            "  0 0 0 0 0",
            0,
        ),
        TestCase::new(
            "alex",
            "hey im alex",
            "        210",
            2,
        ),
        TestCase::new(
            "alex",
            "hey im aelx",
            "        222",
            2,
        ),
        TestCase::new(
            "abcde",
            "bcde abde abccde abzde abdce",
            "   1    1      1     1      ",
            1,
        ),
        TestCase::new(
            "abcde",
            "bcde",
            "   1",
            1,
        ),
    ];

    for f in impls.iter() {
        for case in cases.iter() {
            let result = f(&case.pattern, &case.text, case.max_distance)
                .unwrap()
                .iter()
                .map(|m| (m.distance, m.end))
                .collect::<Vec<_>>();
            assert_eq!(result, case.expected_matches);
        }
    }
}

#[test]
fn test_damerau_levenshtein() {
    #[rustfmt::skip]
    let cases: Vec<TestCase> = vec![
        TestCase::new(
            "abac",
            "acb",
            "322",
            3,
        ),
    ];
    // There's no bitap implementation of _real_ damerau-levenshtein, so only
    // test the baseline version.
    for case in cases.iter() {
        let result = baseline::damerau(&case.pattern, &case.text, case.max_distance)
            .unwrap()
            .iter()
            .map(|m| (m.distance, m.end))
            .collect::<Vec<_>>();
        assert_eq!(result, case.expected_matches);
    }
}

#[test]
fn test_osa() {
    let impls: Vec<BitapImpl> = vec![osa, baseline::osa];
    #[rustfmt::skip]
    let cases: Vec<TestCase> = vec![
        TestCase::new(
            "alex",
            "aelx",
            "   1",
            1,
        ),
        TestCase::new(
            "abcde",
            "bcde abde abccde abzde abdce",
            "   1    1      1     1     1",
            1,
        ),
        // This case is interesting because under damerau-levenshtein distance
        // the results are slightly different.
        TestCase::new(
            "abac",
            "acb",
            "323",
            3,
        ),
    ];

    for f in impls.iter() {
        for case in cases.iter() {
            let result = f(&case.pattern, &case.text, case.max_distance)
                .unwrap()
                .iter()
                .map(|m| (m.distance, m.end))
                .collect::<Vec<_>>();
            assert_eq!(result, case.expected_matches);
        }
    }
}

#[quickcheck]
fn qc_find(pattern: String, text: String) -> TestResult {
    if !pattern_length_is_valid(pattern.chars().count()) {
        return TestResult::discard();
    }
    let a = baseline::find(&pattern, &text).unwrap();
    let b = find(&pattern, &text).unwrap();
    TestResult::from_bool(a == b)
}

#[quickcheck]
fn qc_lev(pattern: String, text: String, max_distance: usize) -> TestResult {
    if !pattern_length_is_valid(pattern.chars().count()) {
        return TestResult::discard();
    }
    let a = baseline::lev(&pattern, &text, max_distance).unwrap();
    let b = lev(&pattern, &text, max_distance).unwrap();
    TestResult::from_bool(a == b)
}

#[quickcheck]
fn qc_osa(pattern: String, text: String, max_distance: usize) -> TestResult {
    if !pattern_length_is_valid(pattern.chars().count()) {
        return TestResult::discard();
    }
    let a = baseline::osa(&pattern, &text, max_distance).unwrap();
    let b = osa(&pattern, &text, max_distance).unwrap();
    TestResult::from_bool(a == b)
}
