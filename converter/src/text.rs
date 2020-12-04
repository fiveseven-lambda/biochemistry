use std::error::Error;
use super::char::{Char, Display};
use super::document::Document;

pub struct Text<'a> {
    pub text: Vec<Token<'a>>,
}

pub enum Token<'a> {
    Char(&'a Char),
    Block(Text<'a>),
    Link(Text<'a>),
    Paren(Text<'a>),
}

#[derive(thiserror::Error, Debug)]
enum TextPrintError{
    #[error("no text after `{0}` at {0:b}")]
    NoDecorationTarget(Char),
}

impl<'a> Text<'a> {
    pub fn print<Writer: std::io::Write>(&self, writer: &mut Writer, document: &Document) -> Result<(), Box<dyn Error>> {
        let mut decorations = Vec::<&Char>::new();
        for token in &self.text {
            match token {
                Token::Char(c) => {
                    match c.value {
                        '^' => {
                            write!(writer, "<sup>")?;
                            decorations.push(c);
                            continue;
                        }
                        '_' => {
                            write!(writer, "<sub>")?;
                            decorations.push(c);
                            continue;
                        }
                        _ => {
                            write!(writer, "{}", c)?;
                        }
                    }
                }
                Token::Block(text) => {
                    text.print(writer, document)?;
                }
                Token::Paren(text) => {
                    write!(writer, "(")?;
                    text.print(writer, document)?;
                    write!(writer, ")")?;
                }
                Token::Link(text) => {
                    match document.names.get(&text) {
                        Some(&index) => {
                            write!(writer, "<a href=\"#{}\">", Display::from(document.items[index].identity))?;
                            text.print(writer, document)?;
                            write!(writer, "</a>")?;
                        }
                        None => {
                            eprint!("Warning: '");
                            text.print(&mut std::io::BufWriter::new(std::io::stderr()), document)?;
                            eprintln!("' not found");
                            write!(writer, "<span class=\"no_link\">")?;
                            text.print(writer, document)?;
                            write!(writer, "</span>")?;
                        }
                    }
                }
            }
            for decoration in decorations.iter().rev() {
                match decoration.value {
                   '^' => write!(writer, "</sup>")?,
                   '_' => write!(writer, "</sub>")?,
                   _ => unreachable!()
                }
            }
            decorations.clear();
        }
        match decorations.first() {
            Some(&c) => Err(Box::new(TextPrintError::NoDecorationTarget(c.clone()))),
            None => Ok(())
        }
    }
}

impl<'a> PartialEq for Text<'a> {
    fn eq(&self, other: &Self) -> bool {
        if self.text.len() == other.text.len() {
            for (left, right) in self.text.iter().zip(other.text.iter()) {
                if left != right {
                    return false;
                }
            }
            true
        } else {
            false
        }
    }
}
impl<'a> Eq for Text<'a> {}

impl<'a> PartialEq for Token<'a> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Token::Char(left), Token::Char(right)) => left == right,
            (Token::Block(left), Token::Block(right)) | (Token::Link(left), Token::Link(right)) | (Token::Paren(left), Token::Paren(right)) => {
                left == right
            }
            _ => false,
        }
    }
}
impl<'a> Eq for Token<'a> {}

use std::hash;

impl<'a> hash::Hash for Text<'a> {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        for token in &self.text {
            token.hash(state);
        }
    }
}

impl<'a> hash::Hash for Token<'a> {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        match self {
            Token::Char(c) => {
                c.hash(state);
            }
            Token::Block(text) | Token::Link(text) | Token::Paren(text) => {
                text.hash(state);
            }
        }
    }
}
