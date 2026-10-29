# monk <img src="https://raw.githubusercontent.com/TheZoq2/ferris/master/rustacean-flat-happy.svg" height="40"/>

Compilador da linguagem MiniLang escrito em Rust, desenvolvido para a disciplina
Compiladores 2026.2. O compilador é construído em fases: léxico, sintático,
semântico, geração de código de três endereços (TAC), máquina virtual e otimizador.

Programas MiniLang usam a extensão `.mk`.

## Pré-requisitos

Rust stable, instalado via [rustup](https://rustup.rs), e um compilador C (o Rust
usa como linker). O alvo `bootstrap` instala o que faltar:

```bash
make bootstrap
```

`make` sozinho lista todos os atalhos.

## Compilar

```bash
make build          # ou: cargo build
```

## Executar

```bash
make run                                   # ecoa exemplos/fatorial.mk
make tokens ARQ=exemplos/escopos.mk        # lista os tokens
make run ARQ=exemplos/if_else.mk ARGS=--tokens
```

Sem o `make`: `cargo run -- exemplos/fatorial.mk --tokens`.

Por enquanto o CLI ecoa o arquivo ou, com `--tokens`, imprime um token por linha
(`linha:coluna TIPO 'lexema'`). As demais fases entram nas próximas issues.

## Testar

```bash
make test     # cargo test
make lint     # cargo fmt --check e cargo clippy --all-targets -- -D warnings
make check    # lint + test, o mesmo que o CI roda
```

Os testes unitários de cada módulo ficam em `src/<modulo>/testes.rs`; os de
integração, em `tests/`.
