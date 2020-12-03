use std::collections::BTreeMap;
use std::error::Error;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn Error>> {
    let files = search_dir("test")?;
    let source = read(&files)?;

    match Source::from(&source).parse() {
        Ok(result) => {
            for expr in &result {
                match expr {
                    Expr::Tag(s) => {
                        print!("Tag: ");
                        for i in *s {
                            print!("{}", i.value);
                        }
                        println!("");
                    }
                    _ => {}
                }
            }
        }
        Err(err) => println!("{}", err),
    }

    Ok(())
}

#[derive(thiserror::Error, Debug)]
enum SearchDirError {
    #[error("duplicate key (`{0}` and `{1}`)")]
    DuplicateKey(PathBuf, PathBuf),
}

fn search_dir(path: &str) -> Result<BTreeMap<usize, PathBuf>, Box<dyn Error>> {
    let mut ret = BTreeMap::new();

    for entry in std::fs::read_dir(path)? {
        let path = entry?.path();
        if path.is_file() {
            let file_name = path.file_name().ok_or("")?.to_str().ok_or("")?;
            let num_part = if let Some(index) = file_name.find(|c: char| !c.is_digit(10)) {
                &file_name[..index]
            } else {
                file_name
            };
            let key = num_part.parse()?;
            if let Some(prev) = ret.remove(&key) {
                return Err(Box::new(SearchDirError::DuplicateKey(prev, path)));
            } else {
                ret.insert(key, path);
            }
        }
    }

    Ok(ret)
}

#[derive(Debug)]
struct Char {
    value: char,
    file: usize,
    line: usize,
    pos: usize,
}

impl Char {
    fn clone(&self) -> Char {
        Char {
            value: self.value,
            file: self.file,
            line: self.line,
            pos: self.pos,
        }
    }
}

fn read(paths: &BTreeMap<usize, PathBuf>) -> Result<Vec<Char>, Box<dyn Error>> {
    use std::io::prelude::*;
    let mut ret = Vec::new();
    for (&i, path) in paths {
        let file = std::fs::File::open(path)?;
        for (j, line) in std::io::BufReader::new(file).lines().enumerate() {
            let mut count = 0usize;
            for (k, c) in line?.chars().enumerate() {
                ret.push(Char {
                    value: c,
                    file: i,
                    line: j + 1,
                    pos: k + 1,
                });
                count += 1;
            }
            ret.push(Char{ value: '\n', file: i, line: j + 1, pos: count + 1 });
        }
    }
    Ok(ret)
}

#[derive(Debug)]
enum Expr<'a> {
    Tag(&'a [Char]),
    Name(Text<'a>),
    Head(&'a [Char], Text<'a>),
    Desc(&'a [Char], Text<'a>),
}

#[derive(Debug)]
enum Text<'a> {
    Str(&'a [Char]),
    Block(Vec<Text<'a>>),
    Link(Vec<Text<'a>>),
}

struct Source<'a> {
    source: &'a [Char],
    iter: std::iter::Enumerate<std::slice::Iter<'a, Char>>,
}

#[derive(thiserror::Error, Debug)]
enum ParseError {
    #[error("at {0:?}, unexpected character")]
    UnexpectedCharacter(Char),
    #[error("at {0:?}, brackets do not match")]
    BracketsDoNotMatch(Char),
    #[error("unexpected end of file")]
    UnexpectedEndOfFile,
}

impl<'a> Source<'a> {
    fn from(source: &'a [Char]) -> Source<'a> {
        Source {
            source: source,
            iter: source.iter().enumerate(),
        }
    }

    fn parse(&mut self) -> Result<Vec<Expr<'a>>, Box<dyn Error>> {
        let mut ret = Vec::new();
        enum State<'a> {
            Space,
            Tag(usize),
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
                        Text::Block(self.parse_block('}')?),
                    )),
                    _ => continue,
                },
                State::Head(index) => match c.value {
                    '{' => State::Elem(Expr::Head(
                        &self.source[index + 1..i],
                        Text::Block(self.parse_block('}')?),
                    )),
                    _ => continue,
                },
                _ => match c.value {
                    '+' => State::Desc(i),
                    '\\' => State::Head(i),
                    '[' => State::Elem(Expr::Name(Text::Link(self.parse_block(']')?))),
                    c if c.is_whitespace() => State::Space,
                    c if c.is_alphanumeric() => State::Tag(i),
                    _ => return Err(Box::new(ParseError::UnexpectedCharacter(c.clone()))),
                },
            };
            match prev {
                State::Tag(index) => match next {
                    State::Tag(_) => continue,
                    _ => ret.push(Expr::Tag(&self.source[index..i])),
                },
                State::Elem(elem) => ret.push(elem),
                _ => {}
            }
            prev = next;
        }
        match prev {
            State::Space => {}
            State::Tag(index) => ret.push(Expr::Tag(&self.source[index..])),
            State::Desc(_) | State::Head(_) => {
                return Err(Box::new(ParseError::UnexpectedEndOfFile))
            }
            State::Elem(elem) => ret.push(elem),
        }
        Ok(ret)
    }

    fn parse_block(&mut self, delim: char) -> Result<Vec<Text<'a>>, Box<dyn Error>> {
        let mut ret = Vec::new();
        enum State<'a> {
            None,
            Str(usize),
            Elem(Text<'a>),
        }
        let mut prev = State::None;
        while let Some((i, c)) = self.iter.next() {
            let next = match c.value {
                '{' => State::Elem(Text::Block(self.parse_block('}')?)),
                '[' => State::Elem(Text::Link(self.parse_block(']')?)),
                c if c == delim => {
                    match prev {
                        State::None => {}
                        State::Str(index) => ret.push(Text::Str(&self.source[index..i])),
                        State::Elem(elem) => ret.push(elem),
                    }
                    return Ok(ret);
                }
                '}' | ']' => return Err(Box::new(ParseError::BracketsDoNotMatch(c.clone()))),
                _ => State::Str(i),
            };
            match prev {
                State::None => {}
                State::Str(index) => match next {
                    State::Str(_) => continue,
                    _ => ret.push(Text::Str(&self.source[index..i])),
                },
                State::Elem(elem) => ret.push(elem),
            }
            prev = next;
        }
        Err(Box::new(ParseError::UnexpectedEndOfFile))
    }
}
