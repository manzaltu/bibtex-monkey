mod crossref;
mod csv;
mod record;

use std::{fs, path::PathBuf};

use anyhow::Result;
use clap::{Parser, Subcommand};
use console::{Style, Term};
use crossref::CrossRef;
use csv::CsvParser;
use indicatif::{ProgressBar, ProgressStyle};
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

    let results = parser.parse()?;
    let pb = ProgressBar::new(results.len() as u64);
    pb.set_style(
        ProgressStyle::with_template(if Term::stdout().size().1 > 80 {
            "{prefix:>12.cyan.bold} [{bar:57}] {pos}/{len} {wide_msg}"
        } else {
            "{prefix:>12.cyan.bold} [{bar:57}] {pos}/{len}"
        })
        .unwrap()
        .progress_chars("=> "),
    );
    pb.set_prefix("Downloading");

    let green_bold = Style::new().green().bold();
    let red_bold = Style::new().red().bold();

    for res in results {
        pb.println(format!(
            "{:>12} {}: {}",
            green_bold.apply_to("Searching"),
            res.author,
            res.title
        ));

        if let Ok(work) = cross_ref.query_work(&res.author, &res.title) {
            let bibtex_path = args.output.join(sanitize_filename::sanitize(format!(
                "{}_{}.bib",
                res.author, res.title
            )));
            pb.set_message(format!("DOI: {}", work.doi));
            cross_ref.download_work_bibtex(&work.doi, bibtex_path)?;
        } else {
            pb.set_message("Not found");
            pb.println(format!(
                "{:>12} {}: {}",
                red_bold.apply_to("Not found"),
                res.author,
                res.title
            ));
        }

        pb.inc(1);
    }

    pb.finish_and_clear();

    Ok(())
}
