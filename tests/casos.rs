//! M13 — roda o banco de casos sintáticos de `tests/casos/`.
//!
//! Cada teste acumula as falhas e só no fim faz um único `assert!`, listando todos
//! os arquivos que falharam.

mod comum;

use comum::{arquivos_mk, erro_esperado, erros_lexicos, ler, nome, parse};

fn relatar(falhas: &[String], total: usize) {
    assert!(
        falhas.is_empty(),
        "{} de {} casos falharam:\n  {}",
        falhas.len(),
        total,
        falhas.join("\n  ")
    );
}

#[test]
fn validos() {
    let arquivos = arquivos_mk("tests/casos/validos");
    let mut falhas = Vec::new();
    for arquivo in &arquivos {
        let fonte = ler(arquivo);
        let lexicos = erros_lexicos(&fonte);
        if !lexicos.is_empty() {
            falhas.push(format!(
                "{}: erro léxico: {}",
                nome(arquivo),
                lexicos.join("; ")
            ));
            continue;
        }
        if let Err(e) = parse(&fonte) {
            falhas.push(format!("{}: deveria parsear, mas deu: {e}", nome(arquivo)));
        }
    }
    relatar(&falhas, arquivos.len());
}

#[test]
fn invalidos() {
    let arquivos = arquivos_mk("tests/casos/invalidos");
    let mut falhas = Vec::new();
    for arquivo in &arquivos {
        let fonte = ler(arquivo);
        let Some(esperado) = erro_esperado(&fonte) else {
            falhas.push(format!(
                "{}: 1ª linha não segue o formato '// ERRO: <mensagem>'",
                nome(arquivo)
            ));
            continue;
        };
        let lexicos = erros_lexicos(&fonte);
        if !lexicos.is_empty() {
            falhas.push(format!(
                "{}: deveria ser lexicamente válido, mas deu: {}",
                nome(arquivo),
                lexicos.join("; ")
            ));
            continue;
        }
        match parse(&fonte) {
            Ok(_) => falhas.push(format!(
                "{}: deveria falhar com '{esperado}', mas parseou",
                nome(arquivo)
            )),
            Err(e) if !e.mensagem.contains(esperado) => falhas.push(format!(
                "{}: esperado erro contendo '{esperado}', mas deu: {e}",
                nome(arquivo)
            )),
            Err(_) => {}
        }
    }
    relatar(&falhas, arquivos.len());
}
