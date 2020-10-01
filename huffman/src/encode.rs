use std::collections::HashMap;
use std::hash::Hash;

pub struct Encoder<'a, T: Eq + Hash, I: Iterator<Item = T>> {
    codes: &'a HashMap<T, Box<[bool]>>,
    source: I,
}

impl<'a, T: Eq + Hash, I: Iterator<Item = T>> Encoder<'a, T, I> {
    pub fn new(codes: &'a HashMap<T, Box<[bool]>>, source: I) -> Self {
        Encoder { codes, source }
    }
}

impl<'a, T: Eq + Hash, I: Iterator<Item = T>> Iterator for Encoder<'a, T, I> {
    type Item = &'a [bool];
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(element) = self.source.next() {
            if let Some(code) = self.codes.get(&element) {
                Some(code.as_ref())
            } else {
                // FIXME: Replace panic with something?
                panic!("Element not found in codes table")
            }
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn encode_empty() {
        unimplemented!();
    }
    #[test]
    fn encode_chars() {
        unimplemented!();
    }
    #[test]
    fn encode_not_found() {
        unimplemented!();
    }
}
