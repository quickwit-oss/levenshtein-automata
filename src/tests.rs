extern crate levenshtein;

use std::collections::HashSet;
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
        let lev = LevenshteinNFA::levenshtein(m, false);
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


#[test]
fn test_dead_state() {
    let nfa = LevenshteinNFA::levenshtein(2, false);
    let parametric_dfa = ParametricDFA::from_nfa(&nfa);
    let dfa = parametric_dfa.build_dfa("abcdefghijklmnop");
    let mut state = dfa.initial_state();
    state = dfa.transition(state, b'X');
    assert!(state != 0);
    state = dfa.transition(state, b'X');
    assert!(state != 0);
    state = dfa.transition(state, b'X');
    assert_eq!(state, 0);
}

#[test]
#[ignore]
fn test_levenshtein_nfa_slow() {
    let test_sample = TestSample::with_num_chars(5, "abcdef", true);
    test_sample.each(test_levenshtein_nfa_util);
}

#[test]
#[ignore]
fn test_levenshtein_dfa_slow() {
    let test_sample = TestSample::with_num_chars(5, "あbぃaえ", false);
    let parametric_dfas: Vec<ParametricDFA> = (0u8..4u8)
        .map(|m| {
            let lev = LevenshteinNFA::levenshtein(m, false);
            ParametricDFA::from_nfa(&lev)
        })
        .collect();

    for left in test_sample.lefts() {
        for m in 0..4u8 {
            let dfa = parametric_dfas[m as usize].build_dfa(&left);
            for right in test_sample.rights() {
                let expected = levenshtein::levenshtein(&left, &right) as u8;
                let expected_distance = make_distance(expected, m);
                let result_distance = dfa.eval(&right);
                assert_eq!(expected_distance, result_distance);
            }
        }
    }
}

#[test]
#[ignore]
fn test_levenshtein_parametric_dfa_slow() {
    let parametric_dfas: Vec<ParametricDFA> = (0u8..4u8)
        .map(|m| {
            let lev = LevenshteinNFA::levenshtein(m, false);
            ParametricDFA::from_nfa(&lev)
        })
        .collect();
    let test_sample = TestSample::with_num_chars(5, "abcdef", true);
    test_sample.each(|left, right| {
        let expected = levenshtein::levenshtein(left, right) as u8;
        for m in 0u8..4u8 {
            let result_distance = parametric_dfas[m as usize].compute_distance(left, right);
            let expected_distance = make_distance(expected, m);
            assert_eq!(expected_distance, result_distance);
        }
    });
}

#[test]
fn test_levenshtein_parametric_dfa_long() {
    let lev = LevenshteinNFA::levenshtein(2, true);
    let param_dfa = ParametricDFA::from_nfa(&lev);
    let test_str = "abcdefghijlmnopqrstuvwxyz\
                    abcdefghijlmnopqrstuvwxyz\
                    abcdefghijlmnopqrstuvwxyz\
                    abcdefghijlmnopqrstuvwxyz";
    let dfa = param_dfa.build_dfa(test_str);
    {
        let result_distance = dfa.eval(test_str);
        assert_eq!(result_distance, Distance::Exact(0));
    }
    {
        let test_str = "abcdefghijlmnopqrstuvwxyz\
                    abcdefghijlnopqrstuvwxyz\
                    abcdefghijlmnopqrstuvwxyz\
                    abcdefghijlmnopqrstuvwxyz";
        let result_distance = dfa.eval(test_str);
        assert_eq!(result_distance, Distance::Exact(1));
    }
    {
        let test_str = "abcdefghijlmnopqrstuvwxyz\
                    abcdefghijlnopqrstuvwxyz\
                    abcdefghijlmnoprqstuvwxyz\
                    abcdefghijlmnopqrstuvwxyz";
        let result_distance = dfa.eval(test_str);
        assert_eq!(result_distance, Distance::Exact(2));
    }

}

fn combinations(alphabet: &[char], len: usize) -> Vec<String> {
    let mut result = vec![];
    let mut prev: Vec<String> = vec![String::from("")];
    for _ in 0..len {
        prev = alphabet
            .iter()
            .cloned()
            .flat_map(|letter: char| {
                prev.iter().map(
                    move |prefix| format!("{}{}", prefix, letter),
                )
            })
            .collect();
        result.extend_from_slice(&prev[..]);
    }
    result
}

struct TestSample {
    lefts: Vec<String>,
    rights: Vec<String>,
}

impl TestSample {
    fn with_num_chars(num_chars: usize, letters: &str, dedup_pattern: bool) -> TestSample {
        let alphabet: Vec<char> = letters.chars().collect();
        let strings = combinations(&alphabet, num_chars);
        let sorted_strings: Vec<String> = strings
            .iter()
            .filter(|s| {
                if !dedup_pattern {
                    return true;
                }
                let mut v = HashSet::new();
                for c in s.chars() {
                    if !v.contains(&c) {
                        if c != alphabet[v.len()] {
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
            rights: strings,
        }
    }

    fn lefts(&self) -> &[String] {
        &self.lefts
    }

    fn rights(&self) -> &[String] {
        &self.rights
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
    let nfa = LevenshteinNFA::levenshtein(2, true);
    test_symmetric(&nfa, "abc", "abc", Distance::Exact(0));
    test_symmetric(&nfa, "abc", "abcd", Distance::Exact(1));
    test_symmetric(&nfa, "abcdef", "abddef", Distance::Exact(1));
    test_symmetric(&nfa, "abcdef", "abdcef", Distance::Exact(1));
}

#[test]
fn test_levenshtein_dfa() {
    let nfa = LevenshteinNFA::levenshtein(2, false);
    let parametric_dfa = ParametricDFA::from_nfa(&nfa);
    let dfa = parametric_dfa.build_dfa("abcabcaaabc");
    assert_eq!(dfa.num_states(), 273);
}

#[test]
fn test_utf8_simple() {
    let nfa = LevenshteinNFA::levenshtein(1, false);
    let parametric_dfa = ParametricDFA::from_nfa(&nfa);
    let dfa = parametric_dfa.build_dfa("あ");
    assert_eq!(dfa.eval("あ"), Distance::Exact(0u8));
    assert_eq!(dfa.eval("ぃ"), Distance::Exact(1u8));
}

#[test]
fn test_simple() {
    let q: &str = "abcdef";
    let nfa = LevenshteinNFA::levenshtein(2, false);
    let parametric_dfa = ParametricDFA::from_nfa(&nfa);
    let dfa = parametric_dfa.build_dfa(q);
    assert_eq!(dfa.eval(q), Distance::Exact(0u8));
    assert_eq!(dfa.eval("abcdf"), Distance::Exact(1u8));
    assert_eq!(dfa.eval("abcdgf"), Distance::Exact(1u8));
    assert_eq!(dfa.eval("abccdef"), Distance::Exact(1u8));
}

#[test]
fn test_jp() {
    let q: &str = "寿司は焦げられない";
    let nfa = LevenshteinNFA::levenshtein(2, false);
    let parametric_dfa = ParametricDFA::from_nfa(&nfa);
    let dfa = parametric_dfa.build_dfa(q);
    assert_eq!(dfa.eval(q), Distance::Exact(0u8));
    assert_eq!(dfa.eval("寿司は焦げられな"), Distance::Exact(1u8));
    assert_eq!(dfa.eval("寿司は焦げられなI"), Distance::Exact(1u8));
    assert_eq!(
        dfa.eval("寿司は焦げられなIい"),
        Distance::Exact(1u8)
    );
}

#[test]
fn test_jp2() {
    let q: &str = "寿a";
    let nfa = LevenshteinNFA::levenshtein(1, false);
    let parametric_dfa = ParametricDFA::from_nfa(&nfa);
    let dfa = parametric_dfa.build_dfa(q);
    assert_eq!(dfa.eval(q), Distance::Exact(0u8));
}
