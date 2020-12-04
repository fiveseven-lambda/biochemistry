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
    match search_dir("source") {
        Ok(files) => match read_files(&files) {
            Ok(text) => {
                let mut source = Source::from(&text);
                match source.parse() {
                    Ok(source) => {
                        match Document::from_source(&source) {
                            Ok(document) => {
                                match std::fs::File::create("index.html") {
                                    Ok(out) => {
                                        let mut buf = std::io::BufWriter::new(out);
                                        match document.print(&mut buf) {
                                            Ok(()) => {
                                            }
                                            Err(err) => {
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
