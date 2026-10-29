use std::path::PathBuf;
use std::process;

use clap::Parser;

/// Compilador da linguagem MiniLang.
#[derive(Parser, Debug)]
#[command(name = "monk", about = "Compilador da linguagem MiniLang", version)]
struct Cli {
    /// Arquivo-fonte MiniLang (.mini)
    arquivo: PathBuf,
}

fn main() {
    let cli = Cli::parse();

    match std::fs::read_to_string(&cli.arquivo) {
        Ok(fonte) => print!("{fonte}"),
        Err(e) => {
            eprintln!(
                "erro: não foi possível ler '{}': {}",
                cli.arquivo.display(),
                e
            );
            process::exit(1);
        }
    }
}
