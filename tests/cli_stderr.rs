//! Testes do CLI para falhas de escrita em stderr: nunca entrar em pânico.

use std::path::PathBuf;
use std::process::{Command, Stdio};

fn exemplo(nome: &str) -> String {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("exemplos")
        .join(nome)
        .to_str()
        .expect("caminho não é UTF-8")
        .to_owned()
}

/// Roda o `monk` por um shell que aplica os redirecionamentos e devolve o código.
#[cfg(target_os = "linux")]
fn codigo_com_redirecionamento(args: &[String], redirecionamento: &str) -> Option<i32> {
    Command::new("sh")
        .arg("-c")
        .arg(format!("exec \"$0\" \"$@\" {redirecionamento}"))
        .arg(env!("CARGO_BIN_EXE_monk"))
        .args(args)
        .stdout(Stdio::null())
        .status()
        .expect("falha ao executar o shell")
        .code()
}

/// Os três caminhos de erro: leitura, léxico e escrita em stdout.
#[cfg(target_os = "linux")]
fn caminhos_de_erro() -> [(Vec<String>, &'static str); 3] {
    [
        (vec![exemplo("nao_existe.mk")], ""),
        (vec![exemplo("erro_lexico.mk")], ""),
        (
            vec![exemplo("fatorial.mk"), "--tokens".to_owned()],
            ">/dev/full",
        ),
    ]
}

#[cfg(target_os = "linux")]
#[test]
fn stderr_em_dev_full_sai_com_1() {
    for (args, stdout) in caminhos_de_erro() {
        let codigo = codigo_com_redirecionamento(&args, &format!("{stdout} 2>/dev/full"));
        assert_eq!(codigo, Some(1), "args: {args:?}");
    }
}

#[cfg(target_os = "linux")]
#[test]
fn stderr_fechado_sai_com_1() {
    for (args, stdout) in caminhos_de_erro() {
        let codigo = codigo_com_redirecionamento(&args, &format!("{stdout} 2>&-"));
        assert_eq!(codigo, Some(1), "args: {args:?}");
    }
}

#[test]
fn stderr_com_pipe_fechado_sai_com_1() {
    for args in [exemplo("nao_existe.mk"), exemplo("erro_lexico.mk")] {
        let (leitor, escritor) = std::io::pipe().expect("falha ao criar o pipe");
        drop(leitor);
        let status = Command::new(env!("CARGO_BIN_EXE_monk"))
            .arg(&args)
            .stdout(Stdio::null())
            .stderr(escritor)
            .status()
            .expect("falha ao executar o binário monk");
        assert_eq!(status.code(), Some(1), "arquivo: {args}");
    }
}
