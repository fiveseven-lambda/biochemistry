use super::char::Char;
use super::text::Text;
use super::text::Token;

// パースのために使う．
pub struct Source<'a> {
    source: &'a [Char],
    iter: std::iter::Enumerate<std::slice::Iter<'a, Char>>,
}

pub enum Expr<'a> {
    // アルファベット，数字， '-' ， ',' で構成された文字列．
    // リンクの名前になる．
    Identity(&'a [Char]),
    // 角括弧 [ ] で囲まれた部分．
    Name(Text<'a>),
    // たとえば \p{ 〜 } は
    // index.html の冒頭で
    // <p> 〜 </p> になる．
    Head(&'a [Char], Text<'a>),
    // 説明文．
    // +解糖系{ グルコースは酸化されてピルビン酸になる }
    // の形式で書かれる．
    Desc(&'a [Char], Text<'a>),
}

#[derive(thiserror::Error, Debug)]
enum ParseError {
    #[error("unexpected character `{0}` at {0:b}")]
    UnexpectedCharacter(Char),
    #[error("no closing bracket to match `{0}` at {0:b}")]
    NoClosingBracket(Char),
    #[error("brackets `{0}` at {0:b} and `{1}` at {1:b} does not match")]
    BracketsDoesNotMatch(Char, Char),
    #[error("unexpected end of file")]
    UnexpectedEndOfFile,
}

use std::error::Error;

impl<'a> Source<'a> {
    // イテレータをもっておく．
    // parse() から parse_block()
    // parse_block() から parse_block()
    // を呼び出したときに，
    // イテレータを引数として渡す必要がない．
    // （ある意味，グローバル変数のような使い方）
    pub fn from(source: &'a [Char]) -> Source<'a> {
        Source {
            source: source,
            iter: source.iter().enumerate(),
        }
    }

    pub fn parse(&mut self) -> Result<Vec<Expr<'a>>, Box<dyn Error>> {
        let mut ret = Vec::new();
        enum State<'a> {
            Space,
            Identity(usize),
            Desc(usize),
            Head(usize),
            Elem(Expr<'a>),
        }
        let mut prev = State::Space;
        while let Some((i, c)) = self.iter.next() {
            let next = match prev {
                State::Desc(index) => match c.value {
                    '{' => State::Elem(Expr::Desc(
                        &self.source[index + 1..i],
                        self.parse_block(c, '}')?,
                    )),
                    _ => continue,
                },
                State::Head(index) => match c.value {
                    '{' => State::Elem(Expr::Head(
                        &self.source[index + 1..i],
                        self.parse_block(c, '}')?,
                    )),
                    _ => continue,
                },
                _ => match c.value {
                    '+' => State::Desc(i),
                    '\\' => State::Head(i),
                    '[' => State::Elem(Expr::Name(self.parse_block(c, ']')?)),
                    c if c.is_whitespace() => State::Space,
                    '-' | ',' => State::Identity(i),
                    c if c.is_alphanumeric() => State::Identity(i),
                    _ => return Err(Box::new(ParseError::UnexpectedCharacter(c.clone()))),
                },
            };
            match prev {
                State::Identity(index) => match next {
                    State::Identity(_) => continue,
                    _ => ret.push(Expr::Identity(&self.source[index..i])),
                },
                State::Elem(elem) => ret.push(elem),
                _ => {}
            }
            prev = next;
        }
        match prev {
            State::Space => {}
            State::Identity(index) => ret.push(Expr::Identity(&self.source[index..])),
            State::Desc(_) | State::Head(_) => {
                return Err(Box::new(ParseError::UnexpectedEndOfFile))
            }
            State::Elem(elem) => ret.push(elem),
        }
        Ok(ret)
    }

    fn parse_block(&mut self, start: &Char, delim: char) -> Result<Text<'a>, Box<dyn Error>> {
        let mut ret = Text { text: Vec::new() };
        let mut escaped = false;
        while let Some((_, c)) = self.iter.next() {
            if escaped {
                ret.text.push(Token::EscapedChar(c));
                escaped = false;
            } else {
                match c.value {
                    '\\' => escaped = true,
                    '{' => ret.text.push(Token::Block(self.parse_block(c, '}')?)),
                    '[' => ret.text.push(Token::Link(self.parse_block(c, ']')?)),
                    '(' => ret.text.push(Token::Paren(self.parse_block(c, ')')?)),
                    c if c == delim => return Ok(ret),
                    '}' | ']' | ')' => return Err(Box::new(ParseError::BracketsDoesNotMatch(start.clone(), c.clone()))),
                    _ => ret.text.push(Token::Char(c)),
                }
            }
        }
        return Err(Box::new(ParseError::NoClosingBracket(start.clone())));
    }
}
