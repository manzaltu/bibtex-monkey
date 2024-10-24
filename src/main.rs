mod crossref;
mod csv;
mod record;

use std::{fs, path::PathBuf};

use anyhow::Result;
use clap::{Parser, Subcommand};
use crossref::CrossRef;
use csv::CsvParser;
use record::RecordParser;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
    output: PathBuf,
}

#[derive(Subcommand)]
enum Commands {
    Csv { path: PathBuf },
}

fn main() -> Result<()> {
    let args = Args::parse();

    let parser: &mut dyn RecordParser = match args.command {
        Commands::Csv { path } => &mut CsvParser::new(path)?,
    };

    let cross_ref = CrossRef::new();

    fs::create_dir_all(&args.output)?;

    for res in parser.parse()? {
        if let Ok(work) = cross_ref.query_work(&res.author, &res.title) {
            let bibtex_path = args.output.join(sanitize_filename::sanitize(format!(
                "{}_{}.bib",
                res.author, res.title
            )));
            cross_ref.download_work_bibtex(&work.doi, bibtex_path)?;
        };
    }

    Ok(())
}
