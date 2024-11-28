use once_cell::sync::Lazy;

mod cli;
mod files;
mod parser;
mod stats;

fn main() {
    Lazy::force(&cli::PARSED_COMMANDS);

    if cli::PARSED_COMMANDS.version {
        println!(env!("CARGO_PKG_VERSION"));
        std::process::exit(1);
    }

    let b = files::get_bib_file(&cli::PARSED_COMMANDS.bib);
    if b.is_err() {
        eprintln!("No bib file found. I give up.");
        std::process::exit(1);
    }
    let b = b.unwrap();

    let fs = files::get_all_files(&cli::PARSED_COMMANDS.files, &cli::PARSED_COMMANDS.dirs);
    if fs.is_err() {
        eprintln!("Error: {:?}", fs);
        std::process::exit(1);
    }
    let fs = fs.unwrap();

    match stats::compute(b, fs) {
        Ok(authors) => stats::print_stats(
            authors,
            if cli::PARSED_COMMANDS.tsv {
                stats::Format::Tsv
            } else {
                stats::Format::Json(cli::PARSED_COMMANDS.jsonarray)
            },
        ),
        Err(e) => eprintln!("Error: {:?}", e),
    }
}
