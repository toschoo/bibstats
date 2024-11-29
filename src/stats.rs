use std::collections::HashMap;
use std::ffi::OsString;

use pacosso;
use pacosso::{Opts, ParseResult};
use serde_json::json;

use crate::parser;
use crate::parser::BibEntry;

// AuthorStats[author] -> map[title] -> count
pub type AuthorStats = HashMap<String, HashMap<String, u32>>;

pub enum Format {
    Json(bool),
    Tsv,
}

pub fn compute(bib: OsString, files: Vec<OsString>, no_files: bool) -> ParseResult<AuthorStats> {
    let bibmap = bib_to_map(parse_bib_file(&bib)?);

    let mut authostats = HashMap::new();

    if no_files {
        for quote in get_quotes_from_stdin()? {
            match count_up(&quote, &bibmap, &mut authostats) {
                Ok(()) => continue,
                Err(()) => eprintln!("Citekey {} not in database", quote),
            };
        }
    } else {
        for file in files {
            for quote in get_quotes_from_file(&file)? {
                match count_up(&quote, &bibmap, &mut authostats) {
                    Ok(()) => continue,
                    Err(()) => eprintln!("Citekey {} not in database", quote),
                };
            }
        }
    }

    Ok(authostats)
}

pub fn print_stats(m: AuthorStats, f: Format) {
    match f {
        Format::Json(a) => stats_as_json(m, a),
        Format::Tsv => stats_as_tsv(m),
    }
}

fn stats_as_tsv(m: AuthorStats) {
    let mut i = 0;
    for (author, works) in m.into_iter() {
        for (title, count) in works.into_iter() {
            println!("{}\t\"{}\"\t\"{}\"\t{}", i, author, title, count);
            i += 1;
        }
    }
}

fn stats_as_json(m: AuthorStats, with_array: bool) {
    let mut first = true;
    if with_array {
        println!("[");
    }
    for (author, works) in m.into_iter() {
        for (title, count) in works.into_iter() {
            let js = json!({
                "author": author,
                "title": title,
                "count": count
            });

            // print comma if we are in an array
            if !first {
                if with_array {
                    println!(",")
                } else {
                    println!("")
                }
            }

            print!("{}", js.to_string());

            if first {
                first = false;
            }
        }
    }
    println!("");
    if with_array {
        println!("]");
    }
}

fn bib_to_map(works: Vec<BibEntry>) -> HashMap<String, BibEntry> {
    let mut m = HashMap::new();
    for work in works {
        if m.contains_key(&work.key) {
            continue;
        }
        m.insert(work.key.clone(), work);
    }
    m
}

fn count_up(
    citekey: &str,
    bib: &HashMap<String, BibEntry>,
    authors: &mut AuthorStats,
) -> Result<(), ()> {
    if !bib.contains_key(citekey) {
        return Err(());
    }
    let b = &bib[citekey];
    let author = authors.entry(b.author.clone()).or_insert(HashMap::new());
    *author.entry(b.title.clone()).or_insert(0) += 1;
    Ok(())
}

fn parse_bib_file(path: &OsString) -> ParseResult<Vec<BibEntry>> {
    pacosso::parse_file(path.clone(), Opts::default(), parser::parse)
}

fn get_quotes_from_file(path: &OsString) -> ParseResult<Vec<String>> {
    pacosso::parse_file(path.clone(), Opts::default(), parser::collect_cites)
}

fn get_quotes_from_stdin() -> ParseResult<Vec<String>> {
    let mut stdin = std::io::stdin();
    let mut s = pacosso::Stream::new(Opts::default(), &mut stdin);
    s.apply(parser::collect_cites)
}

#[allow(dead_code)]
fn show_works(works: Vec<BibEntry>) {
    for work in works {
        println!("{}({}): {}", work.author, work.date, work.title);
    }
}
