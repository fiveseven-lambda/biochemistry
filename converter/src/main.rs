use std::collections::{BTreeMap, HashMap, HashSet};
use std::error::Error;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn Error>> {
    let files = search_dir("test")?;
    let source = read(&files)?;

    match Source::from(&source).parse() {
        Ok(result) => {
            let document = Document::from_source(&result)?;
            let mut writer = std::io::BufWriter::new(std::io::stdout());
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

impl PartialEq for Char {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}
impl Eq for Char {}
impl std::hash::Hash for Char {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.value.hash(state);
    }
}

fn char_to_string(s: &[Char]) -> String {
    let mut ret = String::new();
    for i in s {
        ret.push(i.value);
    }
    ret
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
    Identity(&'a [Char]),
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

/* これさ
 * struct Text {
 *     text: Vec<Token>,
 * }
 *
 * enum Token {
 *     Str(&[Char]),
 *     Block(Text),
 *     Link(Text),
 * }
 *
 * の方が良かった……
 */

impl<'a> PartialEq for Text<'a> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Text::Str(left), Text::Str(right)) => {
                if left.len() == right.len() {
                    for (l, r) in left.iter().zip(right.iter()) {
                        if l.value != r.value {
                            return false;
                        }
                    }
                    true
                } else {
                    false
                }
            }
            (Text::Block(left), Text::Block(right))
                | (Text::Link(left), Text::Link(right))
                => {
                if left.len() == right.len() {
                    for (l, r) in left.iter().zip(right.iter()) {
                        if l != r {
                            return false;
                        }
                    }
                    true
                } else {
                    false
                }
            }
            _ => false
        }
    }
}
impl<'a> Eq for Text<'a> {}

impl<'a> std::hash::Hash for Text<'a> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            Text::Str(chars) => {
                for c in *chars {
                    c.value.hash(state);
                }
            }
            Text::Block(vec) | Text::Link(vec) => {
                for text in vec {
                    text.hash(state);
                }
            }
        }
    }
}

fn print_text<Writer: std::io::Write>(text: &Vec<Text>, writer: &mut Writer, document: &Document) -> Result<(), Box<dyn Error>> {
    let mut sub = false;
    let mut sup = false;
    for t in text {
        match t {
            Text::Str(s) => {
                for c in *s {
                    match c.value {
                        '^' => {
                            sup = true;
                        }
                        '_' => {
                            sub = true;
                        }
                        _ => {
                            writer.write(c.value.to_string().as_bytes());
                            if sup {
                                sup = false;
                            }
                            if sub {
                                sub = false;
                            }
                        }
                    }
                }
            }
            Text::Block(text) => {
                print_text(text, writer, document);
            }
            Text::Link(text) => {
            }
        }
    }
    Ok(())
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

struct Item<'a, 'b> {
    identity: &'a [Char],
    name: Option<&'b Text<'a>>,
    descs: Vec<&'b Text<'a>>,
    groups: HashSet<usize>
}

impl<'a, 'b> Item<'a, 'b> {
    fn from_identity(s: &'a [Char]) -> Item<'a, 'b> {
        Item{
            identity: s,
            name: None,
            descs: Vec::new(),
            groups: HashSet::new()
        }
    }
}

#[derive(Default)]
struct Document<'a, 'b> {
    heads: Vec<(&'a [Char], &'b Text<'a>)>,
    items: Vec<Item<'a, 'b>>,
    groups: Vec<&'a [Char]>,
    names: HashMap<&'b Text<'a>, usize>
}

impl<'a, 'b> Document<'a, 'b> {
    fn from_source(source: &'b Vec<Expr<'a>>) -> Result<Document<'a, 'b>, Box<dyn Error>> {
        let mut ret : Document = Default::default();
        let mut groups = HashMap::<&[Char], usize>::new();
        let mut identities = HashMap::<&[Char], usize>::new();
        let mut index = None;
        for expr in source {
            match expr {
                Expr::Identity(identity) => {
                    match identities.get(identity) {
                        Some(value) => {
                            index = Some(*value);
                        }
                        None => {
                            let len = identities.len();
                            identities.insert(identity, len);
                            ret.items.push(Item::from_identity(identity));
                            index = Some(len);
                        }
                    }
                }
                Expr::Name(name) => {
                    match index {
                        Some(index) => {
                            let target = &mut ret.items[index].name;
                            match target {
                                Some(_) => {
                                }
                                None => {
                                    *target = Some(name);
                                    ret.names.insert(name, index);
                                }
                            }
                        }
                        None => {}
                    }
                }
                Expr::Head(tag, text) => {
                    ret.heads.push((tag, text));
                }
                Expr::Desc(group, text) => {
                    match index {
                        Some(index) => {
                            ret.items[index].descs.push(text);
                            match groups.get(group) {
                                Some(value) => {
                                    ret.items[index].groups.insert(*value);
                                }
                                None => {
                                    let len = groups.len();
                                    groups.insert(group, len);
                                    ret.groups.push(group);
                                    ret.items[index].groups.insert(len);
                                }
                            }
                        }
                        None => {}
                    }
                }
            }
        }
        Ok(ret)
    }
}
