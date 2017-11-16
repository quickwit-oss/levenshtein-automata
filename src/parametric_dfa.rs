use super::dfa::{DFA, DFABuilder};
use super::Index;
use super::levenshtein_nfa::Distance;
use super::levenshtein_nfa::{LevenshteinNFA, MultiState};
use super::alphabet::Alphabet;


#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ParametricState {
    shape_id: u32,
    offset: u32,
}

impl ParametricState {
    fn empty() -> ParametricState {
        ParametricState {
            shape_id: 0u32,
            offset: 0u32
        }
    }
    fn is_dead_end(&self) -> bool {
        self.shape_id == 0
    }
}


#[derive(Clone, Copy)]
pub struct Transition {
    dest_shape_id: u32,
    delta_offset: u32,
}

impl Transition {
    fn apply(&self, state: ParametricState) -> ParametricState {
        ParametricState {
            shape_id: self.dest_shape_id,
            offset: state.offset + self.delta_offset
        }
    }
}

pub struct ParametricDFA {
    distance: Vec<u8>,
    transitions: Vec<Transition>,
    max_distance: u8,
    transition_stride: usize,
    diameter: usize,
}

impl ParametricDFA {

    pub fn initial_state() -> ParametricState {
        ParametricState {
            shape_id: 1,
            offset: 0,
        }
    }

    pub fn num_states(&self) -> usize {
        self.transitions.len() / self.transition_stride
    }

    pub fn build_dfa(&self, query: &str) -> DFA {
        let num_states = self.num_states() * (query.len() + 1);
        let mut state_index: Index<ParametricState> = Index::with_capacity(num_states);

        let query_chars: Vec<char> = query.chars().collect();
        let query_len = query_chars.len();
        let alphabet = Alphabet::for_query_chars(query_chars);

        state_index.get_or_allocate(&ParametricState::empty());
        let initial_state_id = state_index.get_or_allocate(&ParametricDFA::initial_state());

        let mut dfa_builder = DFABuilder::with_capacity(num_states);
        let mask = (1 << self.diameter) - 1;

        for state_id in 0.. {
            if state_id == state_index.len() {
                break;
            }
            let state = *state_index.get_from_id(state_id);
            let default_successor = self.transition(state, 0u32).apply(state);
            let default_successor_id = state_index.get_or_allocate(&default_successor);
            let distance = self.distance(state, query_len);

            let mut state_builder = dfa_builder.add_state(state_id, distance, default_successor_id);
            for &(chr, characteristic_vec) in alphabet.iter() {
                let chi = characteristic_vec.shift_and_mask(state.offset as usize, mask);
                let dest_state: ParametricState = self.transition(state, chi).apply(state);
                let dest_state_id = state_index.get_or_allocate(&dest_state);
                state_builder.add_transition(chr, dest_state_id);
            }
        }

        dfa_builder.set_initial_state(initial_state_id);
        dfa_builder.build()
    }

    // only for debug
    #[cfg(test)]
    pub fn compute_distance(&self, left: &str, right: &str) -> Distance {
        use super::levenshtein_nfa::compute_characteristic_vector;
        use std::cmp;
        let mut state = Self::initial_state();
        let left_chars: Vec<char> = left.chars().collect();
        for chr in right.chars() {
            let start = state.offset as usize;
            let stop = cmp::min(start + self.diameter, left_chars.len());
            let chi = compute_characteristic_vector(&left_chars[start..stop], chr) as u32;
            state = self.transition(state, chi).apply(state);
            if state.is_dead_end() {
                return Distance::AtLeast(self.max_distance + 1u8);
            }
        }
        self.distance(state, left.len())
    }

    pub fn distance(&self, state: ParametricState, query_len: usize) -> Distance {
        let remaining_offset: usize = query_len - state.offset as usize;
        if state.is_dead_end() || remaining_offset >= self.diameter {
            Distance::AtLeast(self.max_distance + 1u8)
        } else {
            let d = self.distance[(self.diameter * state.shape_id as usize) + remaining_offset];
            if d > self.max_distance {
                Distance::AtLeast(d)
            } else {
                Distance::Exact(d)
            }
        }
    }

    pub fn transition(&self, state: ParametricState, chi: u32) -> Transition {
        assert!((chi as usize) < self.transition_stride);
        self.transitions[self.transition_stride * state.shape_id as usize + chi as usize]
    }


    pub fn from_nfa(nfa: &LevenshteinNFA) -> ParametricDFA {

        let mut index: Index<MultiState> = Index::new();
        index.get_or_allocate(&MultiState::empty());
        let initial_state = nfa.initial_states();
        index.get_or_allocate(&initial_state);

        let max_distance = nfa.max_distance();
        let multistate_diameter = nfa.multistate_diameter();
        let mut transitions: Vec<Transition> = vec![];

        let num_chi = 1 << multistate_diameter;
        let chi_values: Vec<u64> = (0..num_chi).collect();

        let mut dest_multistate = MultiState::empty();

        for state_id in 0.. {
            if state_id == index.len() {
                break;
            }
            for &chi in &chi_values {
                {
                    let multistate: &MultiState = index.get_from_id(state_id);
                    nfa.transition(multistate, &mut dest_multistate, chi);
                }
                let translation = dest_multistate.normalize();
                let dest_id = index.get_or_allocate(&dest_multistate);
                transitions.push(Transition {
                    dest_shape_id: dest_id,
                    delta_offset: translation
                });
            }
        }

        let num_states = index.len();
        let multistate_diameter = multistate_diameter as usize;
        let mut distance: Vec<u8> = Vec::with_capacity(multistate_diameter * num_states as usize);

        for state_id in 0..num_states {
            let multistate = index.get_from_id(state_id);
            for offset in 0..multistate_diameter {
                let dist = nfa.multistate_distance(multistate, offset as u32).to_u8();
                distance.push(dist);
            }
        }

        ParametricDFA {
            transition_stride: num_chi as usize,
            distance: distance,
            max_distance: max_distance,
            transitions: transitions,
            diameter: multistate_diameter as usize,
        }
    }

}
