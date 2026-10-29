# monk

Compilador da linguagem MiniLang escrito em Rust, desenvolvido para a disciplina
Compiladores 2026.2. O compilador é construído em fases: léxico, sintático,
semântico, geração de código de três endereços (TAC), máquina virtual e otimizador.

Programas MiniLang usam a extensão `.mini`.

## Pré-requisitos

Rust stable, instalado via [rustup](https://rustup.rs).

## Compilar

```bash
cargo build
```

## Executar

```bash
cargo run -- exemplos/fatorial.mini
```

Por enquanto o CLI apenas lê o arquivo e ecoa o seu conteúdo. As fases do
compilador serão adicionadas pelas próximas issues.

## Testar

```bash
cargo test
cargo fmt --check
cargo clippy --all-targets -- -D warnings
```
