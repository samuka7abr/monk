//! Testes de integração do CLI: rodam o binário `monk` de verdade.

use std::path::PathBuf;
use std::process::{Command, Output};

fn exemplo(nome: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("exemplos")
        .join(nome)
}

fn monk(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_monk"))
        .args(args)
        .output()
        .expect("falha ao executar o binário monk")
}

/// Confere o formato `L:C TIPO 'lexema'`.
fn formato_valido(linha: &str) -> bool {
    let Some((pos, resto)) = linha.split_once(' ') else {
        return false;
    };
    let Some((l, c)) = pos.split_once(':') else {
        return false;
    };
    let Some((tipo, lexema)) = resto.split_once(' ') else {
        return false;
    };
    l.parse::<usize>().is_ok()
        && c.parse::<usize>().is_ok()
        && !tipo.is_empty()
        && tipo.chars().all(|ch| ch.is_ascii_uppercase() || ch == '_')
        && lexema.len() >= 2
        && lexema.starts_with('\'')
        && lexema.ends_with('\'')
}

#[test]
fn tokens_fatorial() {
    let caminho = exemplo("fatorial.mk");
    let saida = monk(&[caminho.to_str().unwrap(), "--tokens"]);
    assert!(saida.status.success());

    let texto = String::from_utf8(saida.stdout).unwrap();
    let linhas: Vec<&str> = texto.lines().collect();

    assert_eq!(linhas[0], "2:1 VAR 'var'");
    assert_eq!(linhas[1], "2:5 ID 'n'");
    assert_eq!(*linhas.last().unwrap(), "9:1 EOF ''");
    for l in &linhas {
        assert!(formato_valido(l), "linha fora do formato: {l:?}");
    }
}

#[test]
fn sem_flag_ecoa() {
    let caminho = exemplo("fatorial.mk");
    let saida = monk(&[caminho.to_str().unwrap()]);
    assert!(saida.status.success());
    assert_eq!(
        String::from_utf8(saida.stdout).unwrap(),
        std::fs::read_to_string(&caminho).unwrap()
    );
}

#[test]
fn arquivo_inexistente() {
    let saida = monk(&["exemplos/nao_existe.mk"]);
    assert_eq!(saida.status.code(), Some(1));
    assert!(!saida.stderr.is_empty());
}

#[test]
fn ajuda_descreve_tokens() {
    let saida = monk(&["--help"]);
    assert!(saida.status.success());
    assert!(String::from_utf8(saida.stdout)
        .unwrap()
        .contains("--tokens"));
}
