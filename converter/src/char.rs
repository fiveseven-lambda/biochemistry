#[derive(Debug, Clone)]
pub struct Char {
    pub value: char,
    pub file: usize,
    pub line: usize,
    pub pos: usize,
}

impl PartialEq for Char {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl Eq for Char {}

use std::hash;

impl hash::Hash for Char {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.value.hash(state);
    }
}

use std::fmt;

impl fmt::Display for Char {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl fmt::Binary for Char {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}, {}, {})", self.file, self.line, self.pos)
    }
}

pub struct Display<'a> {
    text: &'a [Char]
}

impl<'a> Display<'a>{
    pub fn from(text: &'a [Char]) -> Display<'a> {
        Display{ text: text }
    }
}

impl<'a> std::fmt::Display for Display<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for c in self.text {
            c.fmt(f)?;
        }
        Ok(())
    }
}
