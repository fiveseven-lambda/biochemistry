use super::char::{Char, Display};
use super::document::Document;
use std::error::Error;

// { } で囲まれた部分が Text となる．
// "\p{ }" のようなヘッダー部分と
// "+解糖系{ グルコースが分解されてピルビン酸になる }" のような説明部分が該当する

// Text は Token の列
pub struct Text<'a> {
    pub text: Vec<Token<'a>>,
}

pub enum Token<'a> {
    Char(&'a Char),        // 普通の文字
    EscapedChar(&'a Char), // バックスラッシュでエスケープされた文字
    Block(Text<'a>),       // 波括弧 { } で囲まれた部分．波括弧自体は出力されない
    Link(Text<'a>),        // 角括弧 [ ] で囲まれた部分．ハイパーリンクになる
    Paren(Text<'a>),       // 丸括弧 ( ) で囲まれた部分．丸括弧も含めて出力される
}

// ^ （上付き）と _ （下付き）は，
// 直後の Token 1 個を修飾する．
// たとえば ^{〜} と書くとブロック全体が上付きになる．

#[derive(thiserror::Error, Debug)]
enum TextPrintError {
    #[error("no text after `{0}` at {0:b}")]
    NoDecorationTarget(Char), // ^ や _ の直後に何も無い場合
}

enum Decoration<'a> {
    Sup(&'a Char),
    Sub(&'a Char),
}

impl<'a> Text<'a> {
    // Text を index.html に出力するときに使う．
    // [ ] をリンクにするために，引数で受け取った Document を参照する．
    pub fn print<Writer: std::io::Write>(
        &self,
        writer: &mut Writer,
        document: &Document,
    ) -> Result<(), Box<dyn Error>> {
        let mut decorations = Vec::new();
        for token in &self.text {
            match token {
                Token::Char(c) => match c.value {
                    '^' => {
                        write!(writer, "<sup>")?;
                        decorations.push(Decoration::Sup(c));
                        continue;
                    }
                    '_' => {
                        write!(writer, "<sub>")?;
                        decorations.push(Decoration::Sub(c));
                        continue;
                    }
                    _ => {
                        write!(writer, "{}", c)?;
                    }
                },
                Token::EscapedChar(c) => {
                    write!(writer, "{}", c)?;
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
                            write!(
                                writer,
                                "<a href=\"#{}\">",
                                Display::from(document.items[index].identity)
                            )?;
                            text.print(writer, document)?;
                            write!(writer, "</a>")?;
                        }
                        None => {
                            // たとえば "[グルコース]" とあるのに
                            // グルコースが見つからなかったとき：
                            // 警告を出した上で，
                            // リンクにする代わりに <span class="no_link"> </span> で囲む．
                            // "no_link" は style.css で赤文字などにする．
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
            // decorations を逆から見て，
            // <sup> と </sup>， <sub> と </sub> が
            // それぞれ対応するようにする．
            // Token が ^ か _ だったときは，
            // continue されるのでここには達しない
            // （次の Token が出力されてからここに来る）
            for decoration in decorations.iter().rev() {
                match decoration {
                    Decoration::Sup(_) => write!(writer, "</sup>")?,
                    Decoration::Sub(_) => write!(writer, "</sub>")?,
                }
            }
            decorations.clear();
        }
        // ここで decoration の中身が残っていたら，
        // Text の最後に ^ か _ があったということ
        match decorations.first() {
            Some(Decoration::Sup(c)) | Some(Decoration::Sub(c)) => {
                Err(Box::new(TextPrintError::NoDecorationTarget((*c).clone())))
            }
            None => Ok(()),
        }
    }
}

// document.rs で HashMap のキーにするので
// Eq と Hash を impl

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
            (Token::Char(left), Token::Char(right))
            | (Token::EscapedChar(left), Token::EscapedChar(right)) => left == right,
            (Token::Block(left), Token::Block(right))
            | (Token::Link(left), Token::Link(right))
            | (Token::Paren(left), Token::Paren(right)) => left == right,
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
            Token::Char(c) | Token::EscapedChar(c) => {
                c.hash(state);
            }
            Token::Block(text) | Token::Link(text) | Token::Paren(text) => {
                text.hash(state);
            }
        }
    }
}
