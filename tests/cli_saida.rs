//! Testes do CLI para falhas de escrita em stdout.

use std::fs::{self, File};
use std::io;
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};

const SEM_NEWLINE: &str = "var x: int = 1;";
const COM_NEWLINE: &str = "var x: int = 1;\n";

/// Cria um arquivo-fonte em diretório temporário do Cargo.
fn fonte(nome: &str, conteudo: &str) -> PathBuf {
    let caminho = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(nome);
    fs::write(&caminho, conteudo).expect("falha ao criar o arquivo-fonte");
    caminho
}

fn monk_com_stdout(args: &[&str], stdout: impl Into<Stdio>) -> Output {
    Command::new(env!("CARGO_BIN_EXE_monk"))
        .args(args)
        .stdout(stdout)
        .output()
        .expect("falha ao executar o binário monk")
}

/// Stdout cuja outra ponta já foi fechada: a escrita falha com `BrokenPipe`.
fn pipe_fechado() -> io::PipeWriter {
    let (leitor, escritor) = io::pipe().expect("falha ao criar o pipe");
    drop(leitor);
    escritor
}

#[cfg(target_os = "linux")]
fn dev_full() -> File {
    File::options()
        .write(true)
        .open("/dev/full")
        .expect("falha ao abrir /dev/full")
}

#[cfg(target_os = "linux")]
fn assert_falha_de_escrita(saida: &Output) {
    assert_eq!(saida.status.code(), Some(1));
    let erros = String::from_utf8_lossy(&saida.stderr);
    assert!(
        erros.starts_with("erro: falha ao escrever a saída:"),
        "stderr inesperado: {erros:?}"
    );
}

#[cfg(target_os = "linux")]
#[test]
fn eco_sem_newline_em_dev_full_sai_com_1() {
    let arq = fonte("saida_full_sem_nl.mk", SEM_NEWLINE);
    let saida = monk_com_stdout(&[arq.to_str().unwrap()], dev_full());
    assert_falha_de_escrita(&saida);
}

#[cfg(target_os = "linux")]
#[test]
fn eco_com_newline_em_dev_full_sai_com_1() {
    let arq = fonte("saida_full_com_nl.mk", COM_NEWLINE);
    let saida = monk_com_stdout(&[arq.to_str().unwrap()], dev_full());
    assert_falha_de_escrita(&saida);
}

#[cfg(target_os = "linux")]
#[test]
fn tokens_em_dev_full_sai_com_1() {
    let arq = fonte("saida_full_tokens.mk", SEM_NEWLINE);
    let saida = monk_com_stdout(&[arq.to_str().unwrap(), "--tokens"], dev_full());
    assert_falha_de_escrita(&saida);
}

#[test]
fn broken_pipe_nao_e_erro() {
    let sem_nl = fonte("saida_pipe_sem_nl.mk", SEM_NEWLINE);
    let com_nl = fonte("saida_pipe_com_nl.mk", COM_NEWLINE);
    for args in [
        vec![sem_nl.to_str().unwrap()],
        vec![com_nl.to_str().unwrap()],
        vec![sem_nl.to_str().unwrap(), "--tokens"],
    ] {
        let saida = monk_com_stdout(&args, pipe_fechado());
        assert_eq!(saida.status.code(), Some(0), "args: {args:?}");
        assert!(saida.stderr.is_empty(), "args: {args:?}");
    }
}

#[test]
fn eco_sem_newline_preserva_o_conteudo() {
    let arq = fonte("saida_ok_sem_nl.mk", SEM_NEWLINE);
    let saida = monk_com_stdout(&[arq.to_str().unwrap()], Stdio::piped());
    assert!(saida.status.success());
    assert_eq!(saida.stdout, SEM_NEWLINE.as_bytes());
}
