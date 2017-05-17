use std::ascii::AsciiExt;

pub trait StringCase {
    type Owned;

    fn to_capitalized(&self) -> Self::Owned;
}

impl StringCase for str {
    type Owned = String;

    fn to_capitalized(&self) -> Self::Owned {
        let mut result = String::with_capacity(self.len());

        for (i, c) in self.chars().enumerate() {
            result.push(if i == 0 { c.to_ascii_uppercase() } else { c });
        }

        result
    }
}
