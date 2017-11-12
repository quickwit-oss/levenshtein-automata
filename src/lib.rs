mod parametric_dfa;
mod alphabet;
mod levenshtein_nfa;
use self::levenshtein_nfa::{LevenshteinNFA, Distance};
use self::parametric_dfa::ParametricDFA;

#[cfg(test)]
mod tests;