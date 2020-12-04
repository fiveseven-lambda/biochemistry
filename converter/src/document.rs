use super::char::Char;
use super::source::Expr;
use super::text::Text;
use std::collections::{HashMap, HashSet};
use std::error::Error;

pub struct Item<'a, 'b> {
    pub identity: &'a [Char],
    pub name: Option<&'b Text<'a>>,
    pub descs: Vec<&'b Text<'a>>,
    pub groups: HashSet<usize>,
}

impl<'a, 'b> Item<'a, 'b> {
    fn from_identity(s: &'a [Char]) -> Item<'a, 'b> {
        Item {
            identity: s,
            name: None,
            descs: Vec::new(),
            groups: HashSet::new(),
        }
    }
}

#[derive(Default)]
pub struct Document<'a, 'b> {
    pub headers: Vec<(&'a [Char], &'b Text<'a>)>,
    pub items: Vec<Item<'a, 'b>>,
    pub groups: Vec<&'a [Char]>,
    pub names: HashMap<&'b Text<'a>, usize>,
}

#[derive(thiserror::Error, Debug)]
enum DocumentPrintError {
    #[error("no name")]
    NoName
}

impl<'a, 'b> Document<'a, 'b> {
    pub fn from_source(source: &'b Vec<Expr<'a>>) -> Result<Document<'a, 'b>, Box<dyn Error>> {
        let mut ret: Document = Default::default();
        let mut groups = HashMap::<&[Char], usize>::new();
        let mut identities = HashMap::<&[Char], usize>::new();
        let mut index = None;
        for expr in source {
            match expr {
                Expr::Identity(identity) => match identities.get(identity) {
                    Some(value) => {
                        index = Some(*value);
                    }
                    None => {
                        let len = identities.len();
                        identities.insert(identity, len);
                        ret.items.push(Item::from_identity(identity));
                        index = Some(len);
                    }
                },
                Expr::Name(name) => match index {
                    Some(index) => {
                        let target = &mut ret.items[index].name;
                        match target {
                            Some(_) => {}
                            None => {
                                *target = Some(name);
                                ret.names.insert(name, index);
                            }
                        }
                    }
                    None => {}
                },
                Expr::Head(tag, text) => {
                    ret.headers.push((tag, text));
                }
                Expr::Desc(group, text) => match index {
                    Some(index) => {
                        ret.items[index].descs.push(text);
                        if !group.is_empty() {
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
                    }
                    None => {}
                },
            }
        }
        Ok(ret)
    }

    pub fn print<Writer: std::io::Write>(&self, mut writer: &mut Writer) -> Result<(), Box<dyn Error>> {
        write!(writer, "<!DOCTYPE html><html><head><meta charset=\"utf-8\"><title>化合物から見る代謝経路</title><link rel=\"stylesheet\" type=\"text/css\" href=\"style.css\"></head><body>")?;
        for (tag, text) in &self.headers {
            write!(writer, "<")?;
            super::char::print(tag, &mut writer)?;
            write!(writer, ">")?;
            text.print(&mut writer, &self)?;
            write!(writer, "</")?;
            super::char::print(tag, &mut writer)?;
            write!(writer, ">")?;
        }
        for item in &self.items {
            write!(writer, "<p class=\"name\" id=\"")?;
            super::char::print(item.identity, &mut writer)?;
            write!(writer, "\">")?;
            match item.name {
                Some(name) => {
                    name.print(&mut writer, &self)?;
                }
                None => {
                    eprint!("error: name of \"");
                    super::char::print(item.identity, &mut std::io::BufWriter::new(std::io::stderr()))?;
                    eprintln!("\" not provided");
                    return Err(Box::new(DocumentPrintError::NoName));
                }
            }
            write!(writer, "</p><p class=\"group\">")?;
            for (i, &group) in item.groups.iter().enumerate() {
                if i != 0 {
                    write!(writer, "・")?;
                }
                super::char::print(self.groups[group], writer)?;
            }
            write!(writer, "</p>")?;
            for desc in &item.descs {
                write!(writer, "<p class=\"desc\">")?;
                desc.print(&mut writer, &self)?;
                write!(writer, "</p>")?;
            }
        }
        write!(writer, "</body>")?;
        Ok(())
    }
}
