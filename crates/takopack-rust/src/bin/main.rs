use anyhow::Result;
use clap::{Parser, Subcommand};
use takopack_rust::cargo::{process::Process, source::crates_io::FromCratesIoBuilder};
use tempfile::TempDir;

#[derive(Debug, Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    CratesIo {
        name: String,
        version: Option<String>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let tempdir = TempDir::with_prefix("takopack-")?;

    let source = match cli.command {
        Commands::CratesIo { name, version } => {
            let mut builder = FromCratesIoBuilder::new(name);
            if let Some(version) = version {
                builder.version(version);
            }

            builder.resolve()?
        }
    };

    let mut process = Process::from(source, &tempdir);
    process.load().unwrap();

    println!("{:?}", process);

    Ok(())
}
