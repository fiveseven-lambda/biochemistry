use std::collections::BTreeMap;
use std::error::Error;
use std::fs::read_dir;
use std::path::{Path, PathBuf};

#[derive(thiserror::Error, Debug)]
enum SearchDirError {
    #[error("duplicate key (`{0}` and `{1}`)")]
    DuplicateKey(PathBuf, PathBuf),
    #[error("`{0}` is not a directory")]
    NotADirectory(PathBuf),
}

// str には AsRef<Path> が impl されているため，この関数は &str も受け取れる
// 指定されたディレクトリ内のファイルを走査
// ファイル名の先頭の番号を読んで，ファイルを数字の小さい順に並べる
pub fn search_dir<P: AsRef<Path>>(path: P) -> Result<BTreeMap<usize, PathBuf>, Box<dyn Error>> {
    let mut ret = BTreeMap::new();

    for entry in read_dir(path)? {
        let path = entry?.path();
        if path.is_file() {
            let file_name = path.file_name().ok_or("")?.to_str().ok_or("")?;
            let num = {
                // 名前の先頭に数字が付いていないファイルは 0 番
                let mut num = 0usize;
                for c in file_name.chars() {
                    match c.to_digit(10) {
                        Some(d) => num = num * 10 + d as usize,
                        None => break,
                    }
                }
                num
            };
            if let Some(prev) = ret.remove(&num) {
                return Err(Box::new(SearchDirError::DuplicateKey(prev, path)));
            } else {
                ret.insert(num, path);
            }
        } else {
            // 同じ番号のファイルが複数あると DuplicateKey エラー
            return Err(Box::new(SearchDirError::NotADirectory(path)))
        }
    }

    Ok(ret)
}
