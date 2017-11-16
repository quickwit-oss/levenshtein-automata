use super::Index;
use std::cmp;
use super::Distance;

/// Implementation of a Deterministic Finite Automaton for
/// a Levenshtein Automaton targeting UTF-8 encoded strings.
pub struct DFA {
    transitions: Vec<[u32; 256]>,
    distances: Vec<Distance>,
    initial_state: u32,
}

impl DFA {

    /// Returns the initial state
    ///
    /// The initial state can be anything.
    pub fn initial_state(&self) -> u32 {
        self.initial_state
    }

    /// Returns the resulting distance of
    pub fn eval<B: AsRef<[u8]>>(&self, text: B) -> Distance {
        let mut state = self.initial_state();
        for &b in text.as_ref() {
            state = self.transition(state, b);
        }
        self.distance(state)
    }

    /// Returns the output distance associated to the current state
    pub fn distance(&self, state_id: u32) -> Distance {
        self.distances[state_id as usize]
    }


    /// Returns the number of states in the `DFA`.
    pub fn num_states(&self) -> usize {
        self.transitions.len()
    }

    /// Returns the destination state reached
    /// after consuming a given byte.
    pub fn transition(&self, from_state_id: u32, b: u8) -> u32 {
        self.transitions[from_state_id as usize][b as usize]
    }
}

pub struct DFABuilder {
    index: Index<Utf8State>,
    distances: Vec<Distance>,
    transitions: Vec<[u32; 256]>,
    initial_state: u32,
    default_transitions: Vec<u32>
}


pub fn extract_utf8_len_from_first_byte(b: u8) -> usize {
    cmp::min(4, cmp::max(1, (!b).leading_zeros() as usize))
}

#[derive(Eq, PartialEq, Hash, Clone)]
enum Utf8State {
    Original(u32),
    Successor(u32, u8), // successor of state after a byte.
    Predecessor(u32, u8), // predecessor after n-bytes.
}

impl DFABuilder {

    pub fn with_capacity(capacity: usize) -> DFABuilder {
        DFABuilder {
            index:  Index::with_capacity(capacity),
            distances: Vec::with_capacity(capacity),
            transitions: Vec::with_capacity(capacity),
            default_transitions: Vec::with_capacity(capacity),
            initial_state: 0u32,
        }
    }

    pub fn set_initial_state(&mut self, initial_state: u32) {
        let state_id_decoded = self.index.get_or_allocate(&Utf8State::Original(initial_state));
        self.initial_state = state_id_decoded
    }

    fn set_all_successors(&mut self, from_state_id: u32, to_state_id: u32) {
        self.transitions[from_state_id as usize]
            .iter_mut()
            .for_each(|v| *v = to_state_id);
    }

    fn add_transition_id(&mut self, state_id: u32, byte: u8, new_state_id: u32) {
        self.transitions[state_id as usize][byte as usize] = new_state_id;
    }

    pub fn add_transition(&mut self, from_state_id: u32, chr: char, to_state_id: u32) {
        let mut from_state_id_decoded = self.get_or_allocate(&Utf8State::Original(from_state_id));
        let mut buffer = [0u8; 4];
        let bytes: &[u8] = chr.encode_utf8(&mut buffer).as_bytes();
        for &b in &bytes[..bytes.len() - 1] {
            let intermediary_state_id_decoded: u32 = self.get_or_allocate(&Utf8State::Successor(from_state_id_decoded, b));
            self.add_transition_id(from_state_id_decoded, b, intermediary_state_id_decoded);
            from_state_id_decoded = intermediary_state_id_decoded;
        }
        let to_state_id_decoded = self.get_or_allocate(&Utf8State::Original(to_state_id));
        self.add_transition_id(from_state_id_decoded, bytes[bytes.len() - 1], to_state_id_decoded);
    }

    fn get_or_allocate(&mut self, state: &Utf8State) -> u32 {
        let state_id = self.index.get_or_allocate(state);
        let state_id_usize = state_id as usize;
        if self.distances.len() <= state_id_usize {
            self.transitions.resize(state_id_usize + 1, [0u32; 256]);
            self.distances.resize(state_id_usize + 1, Distance::AtLeast(255u8));
            self.default_transitions.resize(state_id_usize + 1, 0u32);
        }
        state_id
    }


    pub fn set_default_transition(&mut self, from_state: u32, to_state: u32) {
        let predecessor_states = [to_state; 4];
        for num_bytes in 1..4 {
            let predecessor_state = Utf8State::Predecessor(to_state, num_bytes as u8);
            let predecessor_state_id: u32 = self.get_or_allocate(&predecessor_state);
            self.set_all_successors(predecessor_state_id, predecessor_states[num_bytes - 1]);
        }
        for b in 128..256 {
            let total_num_bytes = extract_utf8_len_from_first_byte(b as u8);
            self.add_transition_id(from_state, b as u8, predecessor_states[total_num_bytes - 1]);
        }
        self.transitions[from_state as usize]
            .iter_mut()
            .for_each(|v| *v=to_state);

    }

    pub fn add_state(&mut self, state: u32, distance: Distance, default_successor: u32) {
        let state_id = self.get_or_allocate(&Utf8State::Original(state.clone()));
        self.distances[state_id as usize] = distance;
        let default_successor_decoded = self.get_or_allocate(&Utf8State::Original(default_successor));
        self.set_default_transition(state_id, default_successor_decoded);
    }

    pub fn build(self) -> DFA {
        DFA {
            transitions: self.transitions,
            distances: self.distances,
            initial_state: self.initial_state
        }
    }
}

#[cfg(test)]
mod tests {

    use super::extract_utf8_len_from_first_byte;
    use super::DFABuilder;
    use std::char;
    use super::Distance;

    #[test]
    fn test_extract_utf8_len_from_first_byte() {
        let mut buffer = [0u8; 5];
        for codepoint in 0u32..1_114_113u32 {
            if let Some(chr) = char::from_u32(codepoint) {
                let s = chr.encode_utf8(&mut buffer).as_bytes();
                let num_bytes = s.len();
                assert!(num_bytes <= 4);
                assert_eq!(extract_utf8_len_from_first_byte(s[0]), num_bytes);
            }
        }
    }


    #[test]
    fn test_utf8_dfa_builder() {
        let mut dfa_builder = DFABuilder::with_capacity(2);
        dfa_builder.add_state(0, Distance::Exact(1u8), 1);
        dfa_builder.add_state(1, Distance::Exact(0u8), 0);
        dfa_builder.set_initial_state(1u32);
        println!("building");
        let dfa = dfa_builder.build();
        let parity_num_letters = |s: &str| {
            dfa.eval(s).to_u8()
        };
        println!("building2");

        assert_eq!(parity_num_letters("a"), 1u8);
        assert_eq!(parity_num_letters("aあ"), 0u8);
        assert_eq!(parity_num_letters("❤"), 1u8);
        assert_eq!(parity_num_letters("❤❤"), 0u8);
        assert_eq!(parity_num_letters("❤a"), 0u8);
        assert_eq!(parity_num_letters("あ"), 1u8);
        assert_eq!(parity_num_letters("ああ"), 0u8);
    }
}