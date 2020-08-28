use std::collections::{HashMap, HashSet};
use std::io::{Write, BufWriter};

fn main() -> Result<(), std::io::Error> {
    let mut args = std::env::args();
    args.next();
    if let Some(filename) = args.next() {
        match std::fs::read_to_string(filename) {
            Ok(source) => {
                enum State {
                    Init,
                    Tag,
                    TagContent,
                    Identity,
                    ToName,
                    Name,
                    Group,
                    Description,
                }
                let mut state = State::Init;
                let mut line = 1;
                let mut pos = 1;
                let mut header = Vec::new();
                let mut desc = Vec::<(String, String, HashSet<String>, Vec<String>)>::new();
                let mut identities = HashMap::<String, String>::new();
                let mut indices = HashMap::<String, usize>::new();
                let mut cur : Option<usize> = None;
                let mut cur_identity = None;
                let mut cur_name = None;
                let mut cur_group = None;
                let mut cur_desc = None;
                for c in source.chars() {
                    match state {
                        State::Init => {
                            if !c.is_whitespace() {
                                match c {
                                    '\\' => {
                                        state = State::Tag;
                                        header.push((String::new(), String::new()));
                                    }
                                    '+' => {
                                        match cur {
                                            Some(cur) => {
                                                cur_desc = Some(desc[cur].3.len());
                                                desc[cur].3.push(String::new());
                                            }
                                            None => {
                                                println!("line {}, pos {}: ", line, pos);
                                            }
                                        }
                                        cur_group = Some(String::new());
                                        state = State::Group;
                                    }
                                    _ => {
                                        cur_identity = Some(c.to_string());
                                        state = State::Identity;
                                    }
                                }
                            }
                        }
                        State::Tag => {
                            if c == '{' {
                                state = State::TagContent;
                            } else {
                                if let Some(last) = header.last_mut() {
                                    last.0.push(c);
                                }
                            }
                        }
                        State::TagContent => {
                            if c == '}' {
                                state = State::Init;
                            } else {
                                if let Some(last) = header.last_mut(){
                                    last.1.push(c);
                                }
                            }
                        }
                        State::Identity => {
                            if c.is_whitespace() {
                                if let Some(ref cur_identity) = cur_identity {
                                    match indices.get(cur_identity) {
                                        Some(index) => {
                                            cur = Some(*index);
                                            state = State::Init;
                                        }
                                        None => {
                                            let len = indices.len();
                                            indices.insert(cur_identity.to_string(), len);
                                            cur = Some(len);
                                            state = State::ToName;
                                        }
                                    }
                                }
                            } else {
                                if let Some(cur_identity) = &mut cur_identity {
                                    cur_identity.push(c);
                                }
                            }
                        }
                        State::ToName => {
                            if !c.is_whitespace() {
                                cur_name = Some(c.to_string());
                                state = State::Name;
                            }
                        }
                        State::Name => {
                            if c.is_whitespace() {
                                if let Some(cur_identity) = &cur_identity {
                                    if let Some(cur_name) = &cur_name {
                                        desc.push((cur_identity.to_string(), cur_name.to_string(), HashSet::new(), Vec::new()));
                                        identities.insert(cur_name.to_string(), cur_identity.to_string());
                                    }
                                }
                                state = State::Init;
                            } else {
                                if let Some(cur_name) = &mut cur_name {
                                    cur_name.push(c);
                                }
                            }
                        }
                        State::Group => {
                            if c == '{' || c.is_whitespace() {
                                if let Some(cur_group) = &mut cur_group {
                                    if !cur_group.is_empty(){
                                        if let Some(cur) = cur {
                                            desc[cur].2.insert(cur_group.to_string());
                                        }
                                    }
                                    if c != '{' {
                                        cur_group.clear();
                                    }
                                }
                                if c == '{' {
                                    state = State::Description;
                                }
                            } else {
                                if let Some(cur_group) = &mut cur_group {
                                    cur_group.push(c);
                                }
                            }
                        }
                        State::Description => {
                            if c == '}' {
                                state = State::Init;
                            } else {
                                if let Some(cur) = cur {
                                    if let Some(cur_desc) = cur_desc {
                                        desc[cur].3[cur_desc].push(c);
                                    }
                                }
                            }
                        }
                    }
                    if c == '\n' {
                        line += 1;
                        pos = 1;
                    } else {
                        pos += 1;
                    }
                }
                match std::fs::File::create("index.html") {
                    Ok(out) => {
                        let mut writer = BufWriter::new(out);
                        write!(writer, "<!DOCTYPE html><html><head><meta charset=\"utf-8\"><title>化合物から見る代謝経路</title><link rel=\"stylesheet\" type=\"text/css\" href=\"style.css\"></head><body>")?;
                        for (tag, content) in header {
                            write!(writer, "<{}>{}</{}>", tag, content, tag)?;
                        }
                        for (identity, name, group, desc) in desc {
                            write!(writer, "<p class=\"name\" id=\"{}\">{}</p><p class=\"group\">", identity, name)?;
                            for (i, g) in group.iter().enumerate() {
                                if !g.is_empty() {
                                    if i != 0 {
                                        write!(writer, "・")?;
                                    }
                                    write!(writer, "{}", g)?;
                                }
                            }
                            write!(writer, "</p><div class=\"desc\">")?;
                            for d in desc {
                                write!(writer, "<p>")?;
                                enum State {
                                    Init,
                                    Escape,
                                    Link,
                                    Sub
                                }
                                let mut state = State::Init;
                                let mut link = None;
                                for c in d.chars() {
                                    match state {
                                        State::Init => {
                                            if c == '$' {
                                                state = State::Escape;
                                            } else if !c.is_whitespace() {
                                                write!(writer, "{}", c)?;
                                            }
                                        }
                                        State::Escape => {
                                            if c == '[' {
                                                link = Some(String::new());
                                                state = State::Link;
                                            } else if c == '(' {
                                                state = State::Sub;
                                                write!(writer, "<sub>(")?;
                                            }
                                        }
                                        State::Link => {
                                            if c == ']' {
                                                if let Some(ref link) = link {
                                                    match identities.get(link) {
                                                        Some(identity) => {
                                                            write!(writer, "<a href=\"#{}\">{}</a>", identity, link)?;
                                                        }
                                                        None => {
                                                            write!(writer, "{}", link)?;
                                                            println!("{}?", link);
                                                        }
                                                    }
                                                }
                                                state = State::Init;
                                            } else {
                                                if let Some(link) = &mut link {
                                                    link.push(c);
                                                }
                                            }
                                        }
                                        State::Sub => {
                                            if c == ')' {
                                                write!(writer, ")</sub>")?;
                                                state = State::Init;
                                            } else {
                                                write!(writer, "{}", c)?;
                                            }
                                        }
                                    }
                                }
                                write!(writer, "</p>")?;
                            }
                            write!(writer, "</div>")?;
                        }
                        write!(writer, "</body>")?;
                        Ok(())
                    }
                    Err(err) => Err(err)
                }
            }
            Err(err) => Err(err)
        }
    } else {
        Ok(())
    }
}
