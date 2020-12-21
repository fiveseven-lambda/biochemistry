use std::collections::BTreeMap;
use std::error::Error;
use std::path::PathBuf;

use std::fs::File;
use std::io::{BufRead, BufReader};

use super::char::Char;

pub fn read_files(paths: &BTreeMap<usize, PathBuf>) -> Result<Vec<Char>, Box<dyn Error>> {
    let mut ret = Vec::new();
    // paths のキーは，ファイル名先頭の番号
    for (&i, path) in paths {
        for (j, line) in BufReader::new(File::open(path)?).lines().enumerate() {
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
            // 改行は，その行の最後の文字とする
            ret.push(Char {
                value: '\n',
                file: i,
                line: j + 1,
                pos: count + 1,
            });
        }
    }
    Ok(ret)
}
