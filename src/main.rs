use std::io::{self, Write};
use std::path::PathBuf;
use std::process;

use clap::Parser;
use monk::lexer::Lexer;

/// Compilador da linguagem MiniLang.
#[derive(Parser, Debug)]
#[command(name = "monk", about = "Compilador da linguagem MiniLang", version)]
struct Cli {
    /// Arquivo-fonte MiniLang (.mk)
    arquivo: PathBuf,

    /// Imprime os tokens do arquivo, um por linha (`linha:coluna TIPO 'lexema'`)
    #[arg(long)]
    tokens: bool,
}

/// Imprime um token por linha no formato `{linha}:{coluna} {tipo} '{lexema}'`.
fn imprimir_tokens(fonte: &str) -> io::Result<()> {
    let tokens = Lexer::new(fonte).tokens();
    let mut saida = io::stdout().lock();
    for t in &tokens {
        writeln!(saida, "{}:{} {} '{}'", t.linha, t.coluna, t.tipo, t.lexema)?;
    }
    saida.flush()
}

fn main() {
    let cli = Cli::parse();

    let fonte = match std::fs::read_to_string(&cli.arquivo) {
        Ok(fonte) => fonte,
        Err(e) => {
            eprintln!(
                "erro: não foi possível ler '{}': {}",
                cli.arquivo.display(),
                e
            );
            process::exit(1);
        }
    };

    let resultado = if cli.tokens {
        imprimir_tokens(&fonte)
    } else {
        write!(io::stdout().lock(), "{fonte}")
    };

    match resultado {
        Ok(()) => {}
        // Saída fechada (ex.: `| head`): não é erro do programa.
        Err(e) if e.kind() == io::ErrorKind::BrokenPipe => {}
        Err(e) => {
            eprintln!("erro: falha ao escrever a saída: {e}");
            process::exit(1);
        }
    }
}
