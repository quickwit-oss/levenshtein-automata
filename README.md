# Levenshtein-automaton

This crate makes it fast and simple to implement Levenshtein Automata.

# Example

```rust
    extern crate levenshtein_automaton;

    use levenshtein_automaton::{LevenshteinAutomatonBuilder, Distance};

    fn main() {

        let lev_automaton_builder = LevenshteinAutomatonBuilder::new(2, true);

        // We can now build an entire dfa.
        let dfa = lev_automaton_builder.build_dfa("saucisson sec");

        let mut state = dfa.initial_state();
            for &b in "saucissonsec".as_bytes() {
            state = dfa.transition(state, b);
        }
        assert_eq!(dfa.distance(state), Distance::Exact(1));
    }
```

The implementation is based on the following paper
**Fast String Correction with Levenshtein-Automata (2002)** by by Klaus Schulz and Stoyan Mihov.
I also tried to explain it in the following [blog post](https://fulmicoton.com/posts/levenshtein/).


# Bench


The time taken by the construction a Levenshtein DFA
strongly depends on the max distance it can measure and the length of the input string.

Here is the time spent to build a Levenshtein DFA for the string "Levenshtein"


```ignore
test bench::bench_build_dfa_distance1_no_transpose   ... bench:     115,580 ns/iter (+/- 38,866)
test bench::bench_build_dfa_distance1_with_transpose ... bench:     112,540 ns/iter (+/- 29,627)
test bench::bench_build_dfa_distance2_no_transpose   ... bench:     308,064 ns/iter (+/- 141,326)
test bench::bench_build_dfa_distance2_with_transpose ... bench:     301,771 ns/iter (+/- 117,123)
test bench::bench_build_dfa_distance3_no_transpose   ... bench:   1,459,171 ns/iter (+/- 267,558)
test bench::bench_build_dfa_distance3_with_transpose ... bench:   2,123,324 ns/iter (+/- 1,559,343)
test bench::bench_build_dfa_distance4_no_transpose   ... bench:   6,114,126 ns/iter (+/- 654,160)
test bench::bench_build_dfa_distance4_with_transpose ... bench:  10,313,151 ns/iter (+/- 1,283,931)
```
