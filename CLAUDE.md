# CLAUDE.md — monk (compilador MiniLang em Rust)

Projeto da disciplina **Compiladores 2026.2**: um compilador da linguagem **MiniLang**
construído em fases (léxico → sintático → semântico → TAC → VM → otimizador).
O enunciado original usa Python; **este repositório implementa tudo em Rust**.
Programas MiniLang usam a extensão `.mini`.

Equipe: `samuka7abr`, `andreiabtiveron`, `ArturMSS`.
O backlog está nas issues do GitHub. Cada issue tem um ID estável no título
(`[M01]` … `[M26]`) — este arquivo sempre se refere às issues por esse ID.

---

## 1. Regra de ouro: verifique as dependências antes de começar

Toda issue é escrita **assumindo que as issues das quais ela depende já estão prontas**.
Antes de escrever qualquer código para uma issue `MXX`:

1. Leia a issue (`gh issue list --search "[MXX]"` → `gh issue view <n>`) e a seção
   **"Depende de"**.
2. Para **cada** dependência, rode a verificação dela na tabela da seção 6.
   Rode também `cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test`.
3. Se alguma verificação falhar:
   - **pare** e informe ao usuário qual dependência não está pronta e o que falhou
     (comando + saída relevante);
   - **não** implemente o trabalho de outra issue por conta própria. Se o conserto for
     trivial (ex.: um `cargo fmt`), proponha e só faça com o aval do usuário.
4. Só então implemente a issue, respeitando os **contratos** da seção 4. Se precisar
   mudar um contrato, avise o usuário: outras issues dependem dele.
5. Ao terminar, rode a verificação da **própria** issue (seção 6) e todos os testes.
   Testes de fases anteriores precisam continuar passando (regressão).

O estado do projeto é decidido pelos comandos da seção 6, **não** por o issue estar
fechado ou aberto no GitHub.

---

## 2. Comandos

```bash
cargo build
cargo test                       # todos os testes
cargo test --lib lexer           # só os testes unitários do módulo lexer
cargo test --test casos          # um arquivo de tests/ específico
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo run -q -- exemplos/fatorial.mini            # pipeline completo (a partir de M23)
cargo run -q -- exemplos/fatorial.mini --tokens   # M05
cargo run -q -- exemplos/fatorial.mini --ast      # M14
cargo run -q -- exemplos/fatorial.mini --tac      # M21
cargo run -q -- exemplos/fatorial.mini --stats    # M23
cargo run -q -- exemplos/fatorial.mini -O         # M24
```

O CI (`.github/workflows/ci.yml`, criado em M01) roda fmt, clippy e testes em todo PR.

---

## 3. Estrutura do repositório

```
monk/
├── Cargo.toml              # crate "monk": lib + bin; dependência: clap (derive)
├── src/
│   ├── main.rs             # CLI (M01, M05, M06, M14, M17, M19, M21, M23, M24)
│   ├── lib.rs              # declara os módulos públicos
│   ├── token.rs            # M03
│   ├── lexer.rs            # M04, M06
│   ├── ast.rs              # M10, M14 (pretty-printer)
│   ├── parser.rs           # M11, M12
│   ├── simbolos.rs         # M16
│   ├── semantica.rs        # M17, M18
│   ├── ir.rs               # M20
│   ├── vm.rs               # M22
│   └── otimizador.rs       # M24, M25
├── tests/
│   ├── comum/mod.rs        # helpers compartilhados pelos testes de integração
│   ├── casos.rs            # M13 — banco de casos sintáticos
│   ├── semanticos.rs       # M19
│   ├── tac.rs              # M21
│   ├── ponta_a_ponta.rs    # M23
│   ├── otimizacao.rs       # M26
│   └── casos/
│       ├── validos/        # M09
│       ├── invalidos/      # M09 — 1ª linha: "// ERRO: <mensagem esperada>"
│       └── semanticos/     # M19 — 1ª linha: "// ERRO: <mensagem esperada>"
├── exemplos/               # programas .mini; *.out = saída esperada (M23)
└── docs/
    ├── especificacao.md    # M02
    ├── gramatica.md        # M02 (rascunho), M08 (final)
    ├── analise-lexer.md    # M07
    └── otimizacao.md       # M26
```

Testes unitários ficam no próprio módulo (`#[cfg(test)] mod testes`). Testes de
integração ficam em `tests/`.

---

## 4. Contratos entre fases (não mude sem avisar)

Nomes de tipos e funções seguem o português do enunciado. Variantes de enum em CamelCase.

### token.rs (M03)
```rust
pub enum TipoToken { Num, Id, Var, Int, Bool, True, False, If, Else, While, Print,
    Mais, Menos, Vezes, Div, Atrib, Igual, Diferente, Menor, Maior, MenorIgual, MaiorIgual,
    AbrePar, FechaPar, AbreChave, FechaChave, DoisPontos, PontoVirg, Eof }
// derive(Debug, Clone, Copy, PartialEq, Eq, Hash); Display imprime em SCREAMING_SNAKE:
// TipoToken::DoisPontos -> "DOIS_PONTOS", TipoToken::Eof -> "EOF"
pub struct Token { pub tipo: TipoToken, pub lexema: String, pub linha: usize, pub coluna: usize }
pub fn palavra_reservada(lexema: &str) -> Option<TipoToken>;
```

### lexer.rs (M04, M06)
```rust
impl Lexer {
    pub fn new(fonte: &str) -> Self;
    pub fn tokens(&mut self) -> Vec<Token>;   // sempre termina com um Token Eof
    pub fn erros(&self) -> &[String];         // vazio se não houve erro léxico
}
```
Linha e coluna começam em 1 e contam **caracteres** (não bytes); `\t` conta 1 coluna.
O token `Eof` tem lexema `""`.

### ast.rs (M10)
```rust
pub enum Tipo { Int, Bool }                       // Display: "int" / "bool"
pub enum OpBin { Mais, Menos, Vezes, Div, Igual, Diferente, Menor, Maior, MenorIgual, MaiorIgual }
// OpBin::simbolo(&self) -> &'static str  ("+", "-", "*", "/", "==", "!=", "<", ">", "<=", ">=")
pub enum OpUn { Neg }

pub struct Programa { pub comandos: Vec<Comando> }
pub struct Bloco    { pub comandos: Vec<Comando>, pub linha: usize }
pub struct Comando  { pub kind: ComandoKind, pub linha: usize }
pub enum ComandoKind {
    Decl  { nome: String, tipo: Tipo, expr: Expr },
    Atrib { nome: String, expr: Expr },
    Print { expr: Expr },
    If    { cond: Expr, entao: Bloco, senao: Option<Bloco> },
    While { cond: Expr, corpo: Bloco },
    Bloco(Bloco),
}
pub struct Expr { pub kind: ExprKind, pub linha: usize, pub tipo: Option<Tipo> }
pub enum ExprKind {
    Num(i64), Bool(bool), Var(String),
    Unario  { op: OpUn,  expr: Box<Expr> },
    Binario { op: OpBin, esq: Box<Expr>, dir: Box<Expr> },
}
```
`Expr.tipo` nasce `None` no parser e é preenchido pela análise semântica (M18).
Não existe trait Visitor: cada fase percorre a AST com `match` em métodos
`visitar_comando`, `visitar_expr`, `visitar_bloco` — é o equivalente em Rust do
despacho `visitar_<Nó>` do enunciado.

### parser.rs (M11, M12)
```rust
pub struct ErroSintatico { pub mensagem: String }   // impl Display + std::error::Error
impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self;
    pub fn programa(&mut self) -> Result<Programa, ErroSintatico>;
}
```
O parser para no primeiro erro sintático.

### simbolos.rs (M16)
```rust
pub enum Categoria { Variavel }
pub struct Simbolo { pub nome: String, pub tipo: Tipo, pub categoria: Categoria, pub linha: usize, pub nivel: usize }
impl TabelaSimbolos {
    pub fn new() -> Self;                 // já com o escopo global (nível 0)
    pub fn entrar_escopo(&mut self);
    pub fn sair_escopo(&mut self);
    pub fn declarar(&mut self, nome: &str, tipo: Tipo, linha: usize) -> bool; // false = redeclaração
    pub fn buscar(&self, nome: &str) -> Option<&Simbolo>;
}
```

### semantica.rs (M17, M18)
```rust
impl AnalisadorSemantico {
    pub fn new() -> Self;
    pub fn analisar(&mut self, prog: &mut Programa) -> Vec<String>;  // vazio = programa válido
}
```

### ir.rs (M20)
```rust
pub enum Valor { Int(i64), Bool(bool) }            // Display: "5", "true", "false"
pub enum Operando { Const(Valor), Nome(String) }   // Display: o valor ou o nome
pub enum InstrKind {
    Copia   { dest: String, a: Operando },                       // "  x = a"
    Bin     { op: OpBin, dest: String, a: Operando, b: Operando }, // "  t1 = a + b"
    Neg     { dest: String, a: Operando },                       // "  t1 = -a"
    Goto    { rotulo: String },                                  // "  goto L1"
    IfFalse { cond: Operando, rotulo: String },                  // "  ifFalse t1 goto L2"
    Label   { rotulo: String },                                  // "L1:"
    Print   { a: Operando },                                     // "  print a"
}
pub struct Instr { pub kind: InstrKind, pub linha: usize }       // impl Display (formatos acima)
impl GeradorTAC { pub fn new() -> Self; pub fn gerar(self, prog: &Programa) -> Vec<Instr>; }
```
Instruções são indentadas com **2 espaços**; rótulos não têm indentação.
Temporários `t1, t2, …` e rótulos `L1, L2, …` são numerados globalmente e nunca se repetem.

### vm.rs (M22)
```rust
pub struct ErroExecucao { pub mensagem: String }
impl VM {
    pub fn new(codigo: Vec<Instr>) -> Self;
    pub fn executar(&mut self, saida: &mut dyn FnMut(&Valor)) -> Result<(), ErroExecucao>;
    pub fn executadas(&self) -> usize;   // instruções executadas, rótulos não contam
}
```
Aritmética de `int` é `i64` com operações `wrapping_*` (sem pânico por overflow).
`/` trunca em direção a zero (`-7 / 2 == -3`), que é o comportamento nativo do Rust.

### otimizador.rs (M24, M25)
```rust
pub type Passe = fn(Vec<Instr>) -> (Vec<Instr>, bool);   // (novo código, mudou?)
pub fn otimizar(codigo: Vec<Instr>) -> Vec<Instr>;       // aplica PASSES até o ponto fixo
```
Nunca dobre uma divisão por constante zero: o erro tem de acontecer em execução.

### CLI (src/main.rs, clap derive)
`monk <ARQUIVO> [--tokens] [--ast] [--tac] [--stats] [-O]`.
Erros de qualquer fase vão para **stderr**, todos de uma vez (exceto o sintático, que é um só),
e o processo sai com código **1**. Uma fase só roda se a anterior não teve erro.

---

## 5. Mensagens de erro (formato exato — os testes comparam texto)

| Fase | Formato |
|---|---|
| Léxico | `{linha}:{coluna}: erro léxico: caractere inválido '{c}'` |
| Sintático | `{linha}:{coluna}: erro sintático: {msg}, encontrado '{lexema}'` (lexema vazio → `EOF`) |
| Semântico | `linha {linha}: erro semântico: {msg}` |
| Execução | `linha {linha}: erro de execução: divisão por zero` |

Mensagens semânticas padronizadas:
`'{x}' não foi declarada` · `'{x}' já declarada neste escopo` ·
`'{x}' é {tipo}, mas recebe {tipo}` · `condição deve ser bool, encontrado {tipo}` ·
`operador '{op}' inválido para {tipo} e {tipo}` · `operador '-' inválido para {tipo}`.

---

## 6. Verificação de cada issue (use para checar dependências)

Cada linha diz como saber se a issue está **pronta no código**. Rode na raiz do repo.

| ID | Pronta quando… |
|---|---|
| M01 | `cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test` passa; existem `src/{lib,main,token,lexer,ast,parser,simbolos,semantica,ir,vm,otimizador}.rs` e `.github/workflows/ci.yml`; `cargo run -q -- README.md` ecoa o arquivo |
| M02 | `docs/especificacao.md` e `docs/gramatica.md` existem e não estão vazios; `ls exemplos/*.mini` lista ≥ 5 arquivos (incluindo `fatorial.mini`) |
| M03 | `cargo test --lib token` passa; `src/token.rs` define `TipoToken`, `Token` e `palavra_reservada` como na seção 4 |
| M04 | `cargo test --lib lexer` passa com ≥ 8 testes (`cargo test --lib lexer -- --list 2>/dev/null \| grep -c ': test'`) |
| M05 | `cargo run -q -- exemplos/fatorial.mini --tokens` imprime linhas no formato `L:C TIPO 'lexema'`, terminando em `EOF` |
| M06 | `cargo run -q -- exemplos/erro_lexico.mini` imprime erros léxicos em stderr e sai com código 1; `git tag -l v0.1-lexer` não é vazio |
| M07 | `docs/analise-lexer.md` responde às 6 perguntas da ficha |
| M08 | `docs/gramatica.md` contém a gramática EBNF final, a tabela FIRST de `comando` e `fator` e uma derivação mais à esquerda |
| M09 | ≥ 8 arquivos em `tests/casos/validos/` e ≥ 8 em `tests/casos/invalidos/`, todo inválido com 1ª linha `// ERRO: ...` |
| M10 | `cargo test --lib ast` passa; `src/ast.rs` segue o contrato da seção 4 |
| M11 | `cargo test --lib parser` passa, incluindo os testes `precedencia` e `associatividade_esquerda` |
| M12 | `cargo test --lib parser` passa, incluindo `falta_ponto_virgula`; `Parser::programa` aceita todos os comandos |
| M13 | `cargo test --test casos` passa |
| M14 | `cargo run -q -- exemplos/fatorial.mini --ast` imprime a AST indentada começando por `Programa` |
| M15 | `git tag -l v0.2-checkpoint` não é vazio; README tem seções de instalar, executar e testar; existem `exemplos/erro_lexico.mini` e `exemplos/erro_sintatico.mini` |
| M16 | `cargo test --lib simbolos` passa |
| M17 | `cargo test --lib semantica` passa, incluindo `fora_do_escopo` e `redeclaracao`; o CLI roda o analisador após o parser |
| M18 | `cargo test --lib semantica` passa, incluindo os testes `tipos_*` e `sem_cascata` |
| M19 | `cargo test --test semanticos` passa; `cargo run -q -- exemplos/semantico.mini` sai com código 1; `git tag -l v0.3-semantica` não é vazio |
| M20 | `cargo test --lib ir` passa, incluindo `expressao`, `while_rotulos`, `if_sem_else` |
| M21 | `cargo test --test tac` passa; `cargo run -q -- exemplos/fatorial.mini --tac` imprime o TAC |
| M22 | `cargo test --lib vm` passa, incluindo `fatorial` e `divisao_por_zero` |
| M23 | `cargo test --test ponta_a_ponta` passa; `cargo run -q -- exemplos/fatorial.mini` imprime `120`; `git tag -l v0.4-vm` não é vazio |
| M24 | `cargo test --lib otimizador` passa, incluindo `dobramento`, `propagacao` e `ponto_fixo`; `-O` existe no CLI |
| M25 | `cargo test --lib otimizador` passa, incluindo `salto_constante`, `codigo_morto` e `limpeza_rotulos` |
| M26 | `cargo test --test otimizacao` passa; `docs/otimizacao.md` tem a tabela comparativa; `git tag -l v0.5-otimizacao` não é vazio |

Os nomes de teste citados acima fazem parte do contrato: quem implementa a issue cria
testes com esses nomes (ex.: `fn precedencia()` dentro de `mod testes` de `parser.rs`).

---

## 7. Fluxo de trabalho

- Uma branch por issue: `mXX-descricao-curta` (ex.: `m04-lexer`). PR para `main` com
  `Closes #<número>` e o ID no título. Outro membro revisa.
- Commits pequenos, mensagens em português no imperativo.
- Tags (`v0.1-lexer` … `v0.5-otimizacao`) são criadas em `main`, depois do merge,
  pela pessoa da issue que fecha a fase.
- Não deixe `todo!()`, `unimplemented!()` ou `unwrap()` em caminho de erro do usuário:
  erros do programa MiniLang viram mensagens, nunca pânico.
- A especificação da linguagem (`docs/especificacao.md`) está congelada depois de M02.
  Mudanças exigem atualizar lexer, gramática e testes — avise a equipe.
