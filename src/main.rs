mod crossref;
mod csv;
mod record;

use std::{fs, path::PathBuf};

use anyhow::Result;
use clap::{crate_name, crate_version, Parser, Subcommand};
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

    println!(
        "{} {}",
        Style::new().magenta().bold().apply_to(crate_name!()),
        crate_version!()
    );

    let cross_ref = CrossRef::new()?;

    fs::create_dir_all(&args.output)?;
    println!(
        "Saving to files to {:?}",
        Style::new()
            .blue()
            .bold()
            .apply_to(fs::canonicalize(&args.output)?),
    );

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

    let mut download_counter = 0;
    let mut fail_counter = 0;

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
            if cross_ref
                .download_work_bibtex(&work.doi, bibtex_path)
                .is_err()
            {
                fail_counter += 1;
                pb.println(format!(
                    "{:>12} {}: {} ({})",
                    red_bold.apply_to("Failed D/L"),
                    res.author,
                    res.title,
                    work.doi
                ));
            } else {
                download_counter += 1;
            }
        } else {
            fail_counter += 1;
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

    println!(
        "\n\nDone! Downloaded: {} Failed: {}",
        green_bold.apply_to(download_counter),
        red_bold.apply_to(fail_counter)
    );

    Ok(())
}
