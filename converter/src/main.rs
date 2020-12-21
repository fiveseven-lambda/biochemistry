mod char;

mod search_dir;
use search_dir::search_dir;

mod read_files;
use read_files::read_files;

mod source;
mod text;
use source::Source;

mod document;
use document::Document;

fn main() {
    // source ディレクトリ内のファイルを走査
    match search_dir("source") {
        // ファイルを番号順に連結して一つの文字列にする
        Ok(files) => match read_files(&files) {
            Ok(text) => {
                // Source に渡してからパースする
                // （Source は文字列以外にイテレータも持つ）
                let mut source = Source::from(&text);
                match source.parse() {
                    Ok(source) => {
                        // Document に変換
                        // "glucose [グルコース]" と書いてあったときに
                        // "glucose" と "グルコース" を紐付けるような作業は
                        // ここで行われる
                        match Document::from_source(&source) {
                            Ok(document) => {
                                match std::fs::File::create("index.html") {
                                    Ok(out) => {
                                        // index.html に書き出し．
                                        // 文中の[グルコース]をリンクにしたり
                                        // ^ や _ を <sup> や <sub> に変えたりする作業は
                                        // ここで行われる
                                        let mut buf = std::io::BufWriter::new(out);
                                        match document.print(&mut buf) {
                                            Ok(()) => {
                                                println!("output written to index.html");
                                            }
                                            Err(err) => {
                                                // print error でも途中まで書き出されてしまう……
                                                // （直すべき？）
                                                eprintln!("print error: {}", err);
                                            }
                                        }
                                    }
                                    Err(err) => {
                                        eprintln!("failed to open output file: {}", err);
                                    }
                                }
                            }
                            Err(err) => {
                                eprintln!("compile error: {}", err);
                            }
                        }
                    }
                    Err(err) => {
                        eprintln!("parse error: {}", err);
                    }
                }
            }
            Err(err) => {
                eprintln!("error while reading files: {}", err);
            }
        },
        Err(err) => {
            eprintln!("error while searching directory: {}", err);
        }
    }
}
