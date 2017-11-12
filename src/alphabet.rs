



pub struct Alphabet {
    chars: Vec<char>,
}

impl Alphabet {

    pub fn len(&self) -> usize {
        self.chars.len() + 1
    }

    pub fn char_id(&self, chr: char) -> usize {
        self.chars[..]
            .binary_search(&chr)
            .map(|pos| pos + 1)
            .unwrap_or(0)
    }

    pub fn for_query(q: &str) -> Alphabet {
        let mut chars: Vec<char> = q.chars().collect();
        chars.sort();
        chars.dedup();
        Alphabet {
            chars: chars
        }
    }
}


#[cfg(test)]
mod tests {
    use super::Alphabet;

    #[test]
    fn test_alphabet() {
        let alphabet = Alphabet::for_query("happy");
        // [a, h, p, y]
        assert_eq!(alphabet.char_id('0'), 0);
        assert_eq!(alphabet.char_id('a'), 1);
        assert_eq!(alphabet.char_id('b'), 0);
        assert_eq!(alphabet.char_id('h'), 2);
        assert_eq!(alphabet.char_id('p'), 3);
        assert_eq!(alphabet.char_id('y'), 4);
        assert_eq!(alphabet.char_id('z'), 0);
        assert_eq!(alphabet.len(), 5);
    }
}
