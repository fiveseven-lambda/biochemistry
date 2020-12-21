// 文字列として， String の代わりに Vec<Char> を， &str の代わりに &[Char] を用いる．

#[derive(Debug, Clone)]
pub struct Char {
    pub value: char,
    pub file: usize, // どの番号のファイル？（ファイル名先頭の数字）
    pub line: usize, // 何行目？
    pub pos: usize, // 何文字目？
}

// document.rs で HashMap のキーとするので
// Eq と Hash を impl

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

// 普通に出力するとき
impl fmt::Display for Char {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

// エラーメッセージの中で，位置（ファイル番号，行数，文字数）を出力したいとき．
// 本来は 2 進法の出力であるフォーマット {:b} を，位置に代用する（Basho の b （苦しい））．
impl fmt::Binary for Char {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}, {}, {})", self.file, self.line, self.pos)
    }
}

// &[Char] を出力したいとき， Display::from に渡してから println! とか write! に渡す．
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
