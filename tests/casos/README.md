# Banco de casos

Programas `.mk` que os testes de integração rodam. A gramática que eles seguem está
em `docs/gramatica.md`.

| Pasta | Conteúdo | Teste |
|---|---|---|
| `validos/` | programas sintaticamente válidos: o parser tem de aceitar | `tests/casos.rs` (M13) |
| `invalidos/` | programas com **um único** erro sintático | `tests/casos.rs` (M13) |
| `semanticos/` | programas com erros semânticos (mesma convenção dos inválidos) | `tests/semanticos.rs` (M19) |

## Convenção dos inválidos

- A 1ª linha é um comentário com um trecho da mensagem esperada:

  ```
  // ERRO: esperado ';'
  print(x)
  ```

- O teste confere se o texto depois de `// ERRO: ` aparece na mensagem do
  `ErroSintatico`. Por isso essas mensagens são contrato do parser (M11, M12).
- O arquivo tem de ser **lexicamente** válido: o erro é sintático, não léxico.
- Cada arquivo testa um único erro.

| Arquivo | Onde a gramática falha |
|---|---|
| `falta_dois_pontos.mk` | `decl`: depois de `"var" ID` vem `":"`, encontrado `int` |
| `expressao_esperada.mk` | `soma`: depois de `"+"` vem um `termo`, mas `;` ∉ FIRST(expr) |
| `if_sem_parenteses.mk` | `if`: depois de `"if"` vem `"("`, encontrado `x` |
| `falta_ponto_virgula.mk` | `print`: depois de `")"` vem `";"`, encontrado EOF |
| `falta_fecha_chave.mk` | `bloco`: `{ comando }` para em EOF, que não inicia comando, e falta `"}"` |
| `comando_inesperado.mk` | `NUM` ∉ FIRST(comando) |
| `falta_fecha_parenteses.mk` | `fator`: `"(" expr` exige `")"`, encontrado `;` |
| `identificador_esperado.mk` | `decl`: depois de `"var"` vem `ID`, encontrado `NUM` (`1x` é `NUM ID`) |
| `comparacao_encadeada.mk` | `expr` termina depois de `1 < 2` (comparação não associativa); `decl` espera `";"`, encontrado `<` |

Dois casos fixam como o parser decide o fim de uma lista de comandos:

- Em `programa`, o laço tenta um comando para todo token que não é EOF. Por isso
  `1 = x;` dá `comando inesperado`.
- Em `bloco`, o laço para em `}` **ou** em EOF. Por isso o caso do bloco não fechado
  dá `esperado '}'` e não `comando inesperado`.

## Convenção dos válidos

Os válidos também são semanticamente corretos, para poderem ser reaproveitados
nas fases seguintes (semântica, TAC, VM).
