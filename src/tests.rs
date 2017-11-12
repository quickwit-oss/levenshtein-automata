extern crate levenshtein;

use std::collections::HashSet;
use std::mem;
use {ParametricDFA, LevenshteinNFA, Distance};

fn make_distance(n: u8, max_distance: u8) -> Distance {
    if n > max_distance {
        Distance::AtLeast(max_distance + 1u8)
    } else {
        Distance::Exact(n)
    }
}

fn test_levenshtein_nfa_util(left: &str, right: &str) {
    let expected = levenshtein::levenshtein(left, right) as u8;
    for m in 0u8..4u8 {
        let expected_distance = make_distance(expected, m);
        let lev = LevenshteinNFA::levenshtein(m);
        test_symmetric(&lev, left, right, expected_distance);
    }
}


fn test_symmetric(lev: &LevenshteinNFA, left: &str, right: &str, expected: Distance) {
    assert_eq!(lev.compute_distance(left, right), expected);
    assert_eq!(lev.compute_distance(right, left), expected);
}

#[test]
fn test_levenshtein() {
    test_levenshtein_nfa_util("abc", "abc");
    test_levenshtein_nfa_util("abc", "abcd");
    test_levenshtein_nfa_util("aab", "ab");
}

fn combinations(alphabet: &[char], len: usize) -> Vec<String> {
    let mut result = vec![];
    let mut prev: Vec<String> = vec![String::from("")];
    for _ in 0..len {
        prev = alphabet
            .iter()
            .cloned()
            .flat_map(|letter: char| {
                prev
                    .iter()
                    .map(move |prefix| format!("{}{}", prefix, letter))
            })
            .collect();
        result.extend_from_slice(&prev[..]);
    }
    result
}

#[test]
fn test_levenshtein_nfa_slow() {
    let test_sample = TestSample::with_num_chars(5);
    test_sample.each(test_levenshtein_nfa_util);
}


#[test]
fn test_levenshtein_parametric_dfa_slow() {
    let parametric_dfas: Vec<ParametricDFA> = (0u8..4u8)
        .map(|m| {
            let lev = LevenshteinNFA::levenshtein(m);
            ParametricDFA::from_nfa(&lev)
        })
        .collect();
    let test_sample = TestSample::with_num_chars(6);
    test_sample.each(|left, right| {
        let expected = levenshtein::levenshtein(left, right) as u8;
        for m in 0u8..4u8 {
            let result_distance = parametric_dfas[m as usize].compute_distance(left, right);
            let expected_distance = make_distance(expected, m);
            assert_eq!(expected_distance, result_distance);
        }
    });
}

struct TestSample {
    lefts: Vec<String>,
    rights: Vec<String>
}

impl TestSample {
    fn with_num_chars(num_chars: usize) -> TestSample {
        let alphabet = vec!['a', 'b', 'c', 'd', 'e'];
        let strings = combinations(&alphabet, num_chars);
        let sorted_strings: Vec<String> = strings
            .iter()
            .filter(|s| {
                let mut v = HashSet::new();
                for c in s.as_bytes() {
                    if !v.contains(c) {
                        let diff = (c - 97) as usize;
                        if diff != v.len() {
                            return false;
                        } else {
                            v.insert(c);
                        }
                    }
                }
                true
            })
            .cloned()
            .collect();
        TestSample {
            lefts: sorted_strings,
            rights: strings
        }
    }

    fn each<F: Fn(&str, &str)>(&self, f: F) {
        for left in &self.lefts {
            for right in &self.rights {
                if left <= right {
                    f(left, right)
                }
            }
        }
    }
}

#[test]
fn test_damerau() {
    let nfa = LevenshteinNFA::damerau_levenshtein(2);
    test_symmetric(&nfa, "abc", "abc", Distance::Exact(0));
    test_symmetric(&nfa, "abc", "abcd", Distance::Exact(1));
    test_symmetric(&nfa, "abcdef", "abddef", Distance::Exact(1));
    test_symmetric(&nfa, "abcdef", "abdcef", Distance::Exact(1));
}
