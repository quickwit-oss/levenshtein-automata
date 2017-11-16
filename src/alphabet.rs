use std::slice;

#[derive(Clone, Copy)]
pub struct FullCharacteristicVector(u64);

impl FullCharacteristicVector {
   pub fn shift_and_mask(&self, offset: usize, mask: u32) -> u32 {
        ((self.0 >> offset) as u32) & mask
   }
}

impl From<u64> for FullCharacteristicVector {
    fn from(bits: u64) -> Self {
        FullCharacteristicVector(bits)
    }
}

pub struct Alphabet {
    charset: Vec<(char, FullCharacteristicVector)>
}

impl Alphabet {

    pub fn iter(&self) -> slice::Iter<(char, FullCharacteristicVector)> {
        self.charset.iter()
    }

    pub fn for_query_chars(query_chars: Vec<char>) -> Alphabet {
        // TODO : handle more than 64 chars
        // TODO document this limitation.
        assert!(query_chars.len() < 64, "Only query shorter than 64 chars are supported for the moment.");
        let mut charset: Vec<char> = query_chars.clone();
        charset.sort();
        charset.dedup();
        let charset = charset
            .into_iter()
            .map(|c| {
                let bits = query_chars
                    .iter()
                    .cloned()
                    .enumerate()
                    .filter(|&(_, ref query_char)| *query_char == c)
                    .map(|(i, _)| (1u64 << i))
                    .sum();
                (c, FullCharacteristicVector(bits))
            })
            .collect();
        Alphabet {
            charset: charset
        }
    }
}


#[cfg(test)]
mod tests {
    use super::{Alphabet, FullCharacteristicVector};

    #[test]
    fn test_alphabet() {
        let alphabet = Alphabet::for_query_chars("happy".chars().collect());
        let mut it = alphabet.iter();

        {
            let &(ref c, ref chi) = it.next().unwrap();
            assert_eq!(*c, 'a');
            assert_eq!(chi.0, 2u64);
        }
        {
            let &(ref c, ref chi) = it.next().unwrap();
            assert_eq!(*c, 'h');
            assert_eq!(chi.0, 1u64);
        }
        {
            let &(ref c, ref chi) = it.next().unwrap();
            assert_eq!(*c, 'p');
            assert_eq!(chi.0, 4u64 + 8u64);
        }
        {
            let &(ref c, ref chi) = it.next().unwrap();
            assert_eq!(*c, 'y');
            assert_eq!(chi.0, 16u64);
        }
    }

    #[test]
    fn test_full_characteristic() {
        assert_eq!(FullCharacteristicVector(2u64).shift_and_mask(1, 1u32), 1);
        assert_eq!(FullCharacteristicVector((1u64 << 5) + (1u64 << 10)).shift_and_mask(3, 63u32), 4);
    }
}
