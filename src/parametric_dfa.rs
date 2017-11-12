use std::collections::HashMap;
use std::hash::Hash;
use super::levenshtein_nfa::Distance;
use super::levenshtein_nfa::{LevenshteinNFA, MultiState};

#[derive(Clone, Copy, Debug)]
pub struct ParametricState {
    shape_id: u32,
    offset: u32,
}

impl ParametricState {
    fn is_dead_end(&self) -> bool {
        self.shape_id == 0
    }
}

pub fn compute_characteristic_vector(query: &[char], c: char) -> u64 {
    let mut chi = 0u64;
    for i in 0..query.len() {
        if query[i] == c {
            chi |= 1u64 << i;
        }
    }
    chi
}

#[derive(Clone, Copy)]
struct Transition {
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
    transition_stride: usize,
    accept_stride: usize,
    distance: Vec<u8>,
    max_distance: u8,
    transitions: Vec<Transition>,
    chi_mask: u64,
}

impl ParametricDFA {

    pub fn initial_state() -> ParametricState {
        ParametricState {
            shape_id: 1,
            offset: 0,
        }
    }

    // only for debug
    pub fn compute_distance(&self, left: &str, right: &str) -> Distance {
        let mut state = Self::initial_state();
        let left_chars: Vec<char> = left.chars().collect();
        for chr in right.chars() {
            let chi = compute_characteristic_vector(&left_chars[state.offset as usize..], chr);
            state = self.transition(state, chi).apply(state);
            if state.is_dead_end() {
                return Distance::AtLeast(self.max_distance + 1u8);
            }
        }
        self.distance(state, left.len())
    }

    pub fn distance(&self, state: ParametricState, query_len: usize) -> Distance {
        let remaining_offset: usize = query_len - state.offset as usize;
        if state.is_dead_end() || remaining_offset >= self.accept_stride {
            Distance::AtLeast(self.max_distance + 1u8)
        } else {
            let d = self.distance[(self.accept_stride * state.shape_id as usize) + remaining_offset];
            if d > self.max_distance {
                Distance::AtLeast(d)
            } else {
                Distance::Exact(d)
            }
        }
    }

    pub fn transition(&self, state: ParametricState, chi: u64) -> Transition {
        let chi = chi & self.chi_mask;
        self.transitions[self.transition_stride * state.shape_id as usize + chi as usize]
    }

    pub fn accept(&mut self, state: ParametricState, chi: u64) -> ParametricState {
        let transition = self.transition(state, chi);
        transition.apply(state)
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
                    let multistate: &MultiState = index.from_id(state_id);
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
        let accept_stride = multistate_diameter as usize;
        let mut distance: Vec<u8> = Vec::with_capacity(accept_stride * num_states as usize);

        for state_id in 0..num_states {
            let multistate = index.from_id(state_id);
            for offset in 0u8..multistate_diameter {
                let accept_val = max_distance + 1;
                let dist = nfa.multistate_distance(multistate, offset as u32).to_u8();
                distance.push(dist);
            }
        }

        ParametricDFA {
            transition_stride: num_chi as usize,
            accept_stride: accept_stride,
            distance: distance,
            max_distance: max_distance,
            transitions: transitions,
            chi_mask: (1 << multistate_diameter) - 1,
        }
    }

}

struct Index<I: Eq + Hash + Clone> {
    index: HashMap<I, u32>,
    items: Vec<I>,
}

impl<I: Eq + Hash + Clone> Index<I> {

    fn new() -> Index<I> {
        Index {
            index: HashMap::new(),
            items: Vec::new()
        }
    }

    fn get_or_allocate(&mut self, item: &I) -> u32 {
        let index_len: u32 = self.len();
        let item_index = *self.index
            .entry(item.clone())
            .or_insert(index_len);
        if item_index == index_len {
            self.items.push(item.clone());
        }
        item_index as u32
    }

    fn len(&self) -> u32 {
        self.items.len() as u32
    }

    fn from_id(&self, id: u32) -> &I {
        &self.items[id as usize]
    }
}

