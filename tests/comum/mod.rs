//! Helpers compartilhados pelos testes de integração.
//!
//! Cada arquivo de `tests/` que usa este módulo declara `mod comum;`. Nem todo
//! arquivo usa todos os helpers, daí o `allow(dead_code)`.
#![allow(dead_code)]

use std::fs;
use std::path::{Path, PathBuf};

use monk::ast::Programa;
use monk::lexer::Lexer;
use monk::parser::{ErroSintatico, Parser};

/// Lexer + parser. Erros léxicos não entram aqui: veja [`erros_lexicos`].
pub fn parse(fonte: &str) -> Result<Programa, ErroSintatico> {
    Parser::new(Lexer::new(fonte).tokens()).programa()
}

/// Erros léxicos da fonte (vazio se ela for lexicamente válida).
pub fn erros_lexicos(fonte: &str) -> Vec<String> {
    let mut lexer = Lexer::new(fonte);
    lexer.tokens();
    lexer.erros().to_vec()
}

/// Caminho a partir da raiz do crate, independente do diretório de onde o teste roda.
pub fn caminho(relativo: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(relativo)
}

/// Arquivos `*.mk` de um diretório (relativo à raiz do crate), em ordem alfabética.
/// Entra em pânico se o diretório não existir ou não tiver nenhum `.mk`, para que
/// um teste nunca passe sem testar nada.
pub fn arquivos_mk(dir: &str) -> Vec<PathBuf> {
    let dir = caminho(dir);
    let entradas =
        fs::read_dir(&dir).unwrap_or_else(|e| panic!("não consegui ler {}: {e}", dir.display()));
    let mut arquivos: Vec<PathBuf> = entradas
        .map(|e| e.expect("entrada de diretório ilegível").path())
        .filter(|p| p.extension().is_some_and(|ext| ext == "mk"))
        .collect();
    arquivos.sort();
    assert!(
        !arquivos.is_empty(),
        "nenhum arquivo .mk em {}",
        dir.display()
    );
    arquivos
}

/// Conteúdo do arquivo.
pub fn ler(caminho: &Path) -> String {
    fs::read_to_string(caminho)
        .unwrap_or_else(|e| panic!("não consegui ler {}: {e}", caminho.display()))
}

/// Mensagem esperada de um caso inválido: a 1ª linha sem o prefixo `// ERRO: `.
/// `None` se a 1ª linha não seguir a convenção.
pub fn erro_esperado(fonte: &str) -> Option<&str> {
    fonte
        .lines()
        .next()?
        .strip_prefix("// ERRO: ")
        .map(str::trim_end)
        .filter(|msg| !msg.is_empty())
}

/// Nome do arquivo para mensagens de falha.
pub fn nome(caminho: &Path) -> String {
    caminho
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| caminho.display().to_string())
}
