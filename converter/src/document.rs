use super::char::{Char, Display};
use super::source::Expr;
use super::text::Text;
use std::collections::{BTreeSet, HashMap};
use std::error::Error;

pub struct Item<'a, 'b> {
    pub identity: &'a [Char],
    pub name: Option<&'b Text<'a>>,
    pub descs: Vec<&'b Text<'a>>,
    pub groups: BTreeSet<usize>,
}

impl<'a, 'b> Item<'a, 'b> {
    fn from_identity(s: &'a [Char]) -> Item<'a, 'b> {
        Item {
            identity: s,
            name: None,
            descs: Vec::new(),
            groups: BTreeSet::new(),
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
enum CompileError {
    #[error("identity expected before name")]
    NoIdentityBeforeName,
    #[error("identity expected before description")]
    NoIdentityBeforeDesc,
    #[error("duplicate name")]
    DuplicateName,
}

#[derive(thiserror::Error, Debug)]
enum DocumentPrintError {
    #[error("no name")]
    NoName,
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
                    Some(index) => match &ret.items[index].name {
                        Some(prev) => {
                            if name != *prev {
                                eprint!(
                                    "error: duplicate name for `{}`",
                                    Display::from(ret.items[index].identity)
                                );
                                return Err(Box::new(CompileError::DuplicateName));
                            }
                        }
                        None => {
                            ret.items[index].name = Some(name);
                            ret.names.insert(name, index);
                        }
                    },
                    None => {
                        return Err(Box::new(CompileError::NoIdentityBeforeName));
                    }
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
                    None => {
                        return Err(Box::new(CompileError::NoIdentityBeforeDesc));
                    }
                },
            }
        }
        Ok(ret)
    }

    pub fn print<Writer: std::io::Write>(
        &self,
        mut writer: &mut Writer,
    ) -> Result<(), Box<dyn Error>> {
        // <body> 冒頭の <header> タグ内に最終更新日を書くために，
        // ここだけ source に記述せずプログラム内に直接書いている．
        // これを source に書くとしたら
        // \date; のようなコマンド記法を導入することになるが
        // このためだけにわざわざそこまでするのもなんだかなぁという感じ．
        write!(
            writer,
            "<!DOCTYPE html>\
            <html>\
                <head>\
                    <meta charset=\"utf-8\">\
                    <title>化合物から見る代謝経路</title>\
                    <link rel=\"stylesheet\" type=\"text/css\" href=\"style.css\">\
                </head>\
                <body>\
                    <header>\
                        <h1>化合物から見る代謝経路</h1>\
                        <p>最終更新日：{}</p>\
                    </header>",
            chrono::Utc::now()
                .with_timezone(&chrono::offset::FixedOffset::east(9 * 3600))
                .format("%Y/%m/%d"),
        )?;
        for (tag, text) in &self.headers {
            write!(writer, "<{}>", Display::from(tag))?;
            text.print(&mut writer, &self)?;
            write!(writer, "</{}>", Display::from(tag))?;
        }
        for item in &self.items {
            write!(
                writer,
                "<div class=\"item\"><div class=\"head\"><p class=\"name\" id=\"{}\">",
                Display::from(item.identity)
            )?;
            match item.name {
                Some(name) => {
                    name.print(&mut writer, &self)?;
                }
                None => {
                    eprintln!(
                        "error: name of `{}` not provided",
                        Display::from(item.identity)
                    );
                    return Err(Box::new(DocumentPrintError::NoName));
                }
            }
            write!(writer, "</p><p class=\"group\">")?;
            for (i, &group) in item.groups.iter().enumerate() {
                if i != 0 {
                    write!(writer, "・")?;
                }
                write!(writer, "{}", Display::from(self.groups[group]))?;
            }
            write!(writer, "</p></div><div class=\"descs\">")?;
            for desc in &item.descs {
                write!(writer, "<p class=\"desc\">")?;
                desc.print(&mut writer, &self)?;
                write!(writer, "</p>")?;
            }
            write!(writer, "</div></div>")?;
        }
        write!(writer, "</body>")?;
        Ok(())
    }
}
