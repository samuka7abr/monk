# Análise do scanner de um projeto didático: jlox

Ficha da M07. Comparamos o scanner do **jlox**, o interpretador Java do livro
*Crafting Interpreters* (Robert Nystrom), com o nosso `src/lexer.rs` (M04, M06).

| | |
|---|---|
| Projeto | [munificent/craftinginterpreters](https://github.com/munificent/craftinginterpreters) |
| Arquivo lido | [`java/com/craftinginterpreters/lox/Scanner.java`][scanner] (241 linhas) |
| Apoio | [`Lox.java`][lox] (relato de erros) e [`Token.java`][token] (posição) |
| Commit fixado | `4a840f70f69c6ddd17cfef4f6964f8e1bcd8c3d4` (os links de linha apontam para ele) |
| Nosso código | `src/lexer.rs`, `src/token.rs` e `src/main.rs` no commit `d11fdc7` da `main` |

O jlox lê a fonte como `String` e anda por índices (`start` e `current`). O nosso
lexer converte a fonte em `Vec<char>` e anda por `pos`, mantendo linha e coluna.

## 1. Onde está o laço principal do scanner?

Em [`scanTokens()`][s-scantokens] (linhas 47–56). O laço repete
`start = current; scanToken();` até `isAtEnd()` e, no fim, acrescenta o token
`EOF`:

```java
while (!isAtEnd()) {
  // We are at the beginning of the next lexeme.
  start = current;
  scanToken();
}
tokens.add(new Token(EOF, "", null, line));
```

Cada chamada de [`scanToken()`][s-scantoken] (linhas 59–132) consome **um
lexema**: lê um caractere com `advance()` e decide num `switch`. Espaços,
quebras de linha e comentários também são casos desse `switch`, só que não geram
token.

**No nosso lexer:** o laço é o `loop` de [`Lexer::tokens`][m-tokens] (linhas
27–52). A diferença é que espaços e comentários são consumidos **antes** de cada
token, em `pular_espacos_e_comentarios`, e a decisão é uma cadeia `if/else`
(dígito, letra, operador, erro) em vez de um `switch` por caractere. Os dois
terminam igual: um token `Eof` com lexema vazio.

## 2. Como diferencia palavra reservada de identificador?

Por uma tabela consultada **depois** de ler o identificador inteiro:

- o mapa estático [`keywords`][s-keywords] (linhas 13–33) é preenchido uma vez,
  na inicialização da classe, com as 16 reservadas do Lox;
- [`identifier()`][s-identifier] (linhas 135–147) consome letras, dígitos e `_`
  enquanto houver (máxima munição), pega o texto com
  `source.substring(start, current)` e faz `keywords.get(text)`. Se o resultado
  for `null`, o token é `IDENTIFIER`.

Como a busca só acontece depois do lexema completo, `whilex` é identificador e
não `while` seguido de `x`.

**No nosso lexer:** é a mesma estratégia. A [linha 39][m-reservada] lê o
identificador inteiro e chama [`palavra_reservada`][m-palavra] (`token.rs`,
linhas 99–112), com `unwrap_or(TipoToken::Id)`. A diferença é que a nossa tabela
é um `match` sobre `&str`, resolvido pelo compilador, sem `HashMap` montado em
tempo de execução. O teste `reservada_x_identificador` cobre o caso `whilex`.

## 3. Como reconhece operadores de 2 caracteres?

Com [`match(expected)`][s-match] (linhas 186–192), um avanço condicional: se o
próximo caractere for `expected`, consome e devolve `true`; senão, não consome
nada. No [`switch`][s-twochar] (linhas 73–84), o primeiro caractere já foi
consumido e o `match` decide o resto:

```java
case '<':
  addToken(match('=') ? LESS_EQUAL : LESS);
  break;
```

O mesmo truque separa `/` de comentário `//` ([linhas 87–94][s-slash]).

**No nosso lexer:** [`ler_operador`][m-operador] (linhas 106–137) faz o
contrário: primeiro **olha** o par `(c, proximo())` sem consumir nada e só depois
avança 1 ou 2 posições. O efeito é o mesmo (casamento guloso, `<=` antes de `<`).
Para a MiniLang essa ordem é mais natural, porque `!` sozinho não é token (seção
2.8 da especificação). No jlox existe `BANG`, então consumir o `!` antes de
decidir nunca dá errado. No nosso caso, se `!` não for seguido de `=`,
`ler_operador` devolve `None` sem ter consumido nada, e quem chama registra o
erro léxico.

## 4. Ao achar um caractere inválido: para ou continua?

**Continua.** O caso [`default`][s-default] (linhas 114–129) chama
`Lox.error(line, "Unexpected character.")` ([linha 126][s-erro]). O caractere já
foi consumido pelo `advance()` do início de `scanToken()` ([linha 60][s-advance]),
então o laço segue no próximo. Em [`Lox.java`][l-report] (linhas 109–118),
`error` só imprime `[line N] Error: ...` em stderr e liga a flag global
`hadError`. Assim o scanner reporta **todos** os caracteres inválidos numa
passada. É um modo pânico mínimo: descarta um caractere e recomeça.

Quem decide parar é [`Lox.run`][l-run] (linhas 64–106). Um detalhe: o **parser
roda mesmo com erro léxico** (linhas 75–80), e o `hadError` só é conferido
depois dele, na [linha 84][l-hadError]. O parser recebe uma lista de tokens com
um "buraco" onde estava o caractere inválido, e pode reportar erros sintáticos
derivados do erro léxico. O processo termina com código 65.

**Comparação com o nosso lexer (M06, #6):**

| | jlox | monk |
|---|---|---|
| Ao achar caractere inválido | reporta e continua | reporta e continua ([linhas 43–47][m-erro]: `erros.push(...)` e `avancar()`) |
| Recuperação | descarta 1 caractere | descarta 1 caractere (mesmo modo pânico) |
| Onde ficam os erros | impressos na hora; flag global `hadError` | acumulados em `Lexer::erros()`, sem estado global |
| A fase seguinte roda? | **sim**: o parser roda e só depois se confere `hadError` | **não**: [`main.rs`][m-main] (linhas 45–52) imprime todos os erros e sai com código 1 antes do parser |
| Formato | `[line 3] Error: Unexpected character.` | `3:16: erro léxico: caractere inválido '@'` |

A recuperação é a mesma. A diferença está no que vem depois: a nossa regra "uma
fase só roda se a anterior não teve erro" (CLAUDE.md, seção 4) evita erros
sintáticos em cascata causados por um caractere inválido. Além disso, devolver os
erros num `Vec` em vez de imprimir deixa o lexer testável sem capturar stderr.

## 5. Que posição o token guarda (linha? coluna?)

**Só a linha.** [`Token`][t-token] (linhas 4–20 de `Token.java`) tem `type`,
`lexeme`, `literal` (o valor já convertido, como o `double` de um número) e
`line`. O scanner mantém um contador [`line`][s-line] (linha 40) que só aumenta
no `'\n'` ([linhas 104–106][s-newline]) e dentro de strings
([linha 168][s-string]). Não há coluna. Como `addToken` usa o valor de `line` no
**fim** do lexema, uma string de várias linhas fica com a linha em que termina.

**No nosso lexer:** guardamos **linha e coluna**, contadas em caracteres, com
`\t` valendo 1 ([`Token`][m-token], `token.rs` linhas 79–84). As duas são
capturadas no **início** do token ([linha 32][m-posicao]) e atualizadas em
[`avancar`][m-avancar] (linhas 67–77). A coluna é necessária porque o nosso
formato de erro é `{linha}:{coluna}`. Também não guardamos `literal`: o token só
tem o lexema, e quem converte `NUM` para `i64` (e valida o intervalo da seção
2.5) é o parser.

## 6. Uma ideia que vamos copiar, e por quê

**O lexema como fatia da fonte, entre `start` e `current`.** No jlox, toda
função de reconhecimento só **avança**; o texto do token é recortado num único
lugar, [`addToken`][s-addtoken] (linhas 236–239):
`String text = source.substring(start, current);`. Por construção, o lexema é
exatamente o trecho consumido da fonte.

No nosso lexer, cada caminho monta o lexema por conta própria:

- `ler_enquanto` empurra caractere por caractere numa `String`;
- `ler_operador` reconstrói o texto com `format!("{c}=")` e `c.to_string()`.

O `format!("{c}=")` só funciona porque, hoje, todo operador de 2 caracteres
termina em `=`. Com as extensões da seção 10 da especificação (`&&`, `||`), ele
geraria o lexema errado sem nenhum erro de compilação. Copiar a ideia do jlox
elimina essa classe de bug e simplifica as funções auxiliares.

Como ficaria no `lexer.rs`. Isto é só um esboço: o código não foi alterado nesta
issue.

```rust
loop {
    self.pular_espacos_e_comentarios();
    let Some(c) = self.atual() else { break };
    let (inicio, linha, coluna) = (self.pos, self.linha, self.coluna); // `start` do jlox

    let tipo = if c.is_ascii_digit() {
        self.pular_enquanto(|c| c.is_ascii_digit()); // só avança, não monta String
        TipoToken::Num
    } else if c.is_ascii_alphabetic() || c == '_' {
        self.pular_enquanto(|c| c.is_ascii_alphanumeric() || c == '_');
        TipoToken::Id
    } else if let Some(tipo) = self.ler_operador(c) { // devolve só o tipo
        tipo
    } else {
        // erro léxico, exatamente como hoje
        continue;
    };

    // Único lugar que recorta o lexema, como o `addToken` do jlox.
    let lexema: String = self.fonte[inicio..self.pos].iter().collect();
    let tipo = if tipo == TipoToken::Id {
        palavra_reservada(&lexema).unwrap_or(TipoToken::Id)
    } else {
        tipo
    };
    saida.push(Token::new(tipo, lexema, linha, coluna));
}
```

Os testes atuais de `src/lexer/testes.rs` (lexemas de `a<=b`, `x==y`, reservadas)
cobririam essa mudança sem alteração, porque a interface pública não muda.

**Ideia considerada e não escolhida: o `match(expected)`.** É elegante no jlox,
mas pressupõe consumir o primeiro caractere antes de decidir. Na MiniLang o `!`
sozinho é erro, então seria preciso tratar o `!` como caso especial antes de
consumir. A nossa leitura antecipada com `proximo()` já resolve isso sem exceção.

[scanner]: https://github.com/munificent/craftinginterpreters/blob/4a840f70f69c6ddd17cfef4f6964f8e1bcd8c3d4/java/com/craftinginterpreters/lox/Scanner.java
[lox]: https://github.com/munificent/craftinginterpreters/blob/4a840f70f69c6ddd17cfef4f6964f8e1bcd8c3d4/java/com/craftinginterpreters/lox/Lox.java
[token]: https://github.com/munificent/craftinginterpreters/blob/4a840f70f69c6ddd17cfef4f6964f8e1bcd8c3d4/java/com/craftinginterpreters/lox/Token.java
[s-keywords]: https://github.com/munificent/craftinginterpreters/blob/4a840f70f69c6ddd17cfef4f6964f8e1bcd8c3d4/java/com/craftinginterpreters/lox/Scanner.java#L13-L33
[s-line]: https://github.com/munificent/craftinginterpreters/blob/4a840f70f69c6ddd17cfef4f6964f8e1bcd8c3d4/java/com/craftinginterpreters/lox/Scanner.java#L40
[s-scantokens]: https://github.com/munificent/craftinginterpreters/blob/4a840f70f69c6ddd17cfef4f6964f8e1bcd8c3d4/java/com/craftinginterpreters/lox/Scanner.java#L47-L56
[s-scantoken]: https://github.com/munificent/craftinginterpreters/blob/4a840f70f69c6ddd17cfef4f6964f8e1bcd8c3d4/java/com/craftinginterpreters/lox/Scanner.java#L59-L132
[s-advance]: https://github.com/munificent/craftinginterpreters/blob/4a840f70f69c6ddd17cfef4f6964f8e1bcd8c3d4/java/com/craftinginterpreters/lox/Scanner.java#L60
[s-twochar]: https://github.com/munificent/craftinginterpreters/blob/4a840f70f69c6ddd17cfef4f6964f8e1bcd8c3d4/java/com/craftinginterpreters/lox/Scanner.java#L73-L84
[s-slash]: https://github.com/munificent/craftinginterpreters/blob/4a840f70f69c6ddd17cfef4f6964f8e1bcd8c3d4/java/com/craftinginterpreters/lox/Scanner.java#L87-L94
[s-newline]: https://github.com/munificent/craftinginterpreters/blob/4a840f70f69c6ddd17cfef4f6964f8e1bcd8c3d4/java/com/craftinginterpreters/lox/Scanner.java#L104-L106
[s-default]: https://github.com/munificent/craftinginterpreters/blob/4a840f70f69c6ddd17cfef4f6964f8e1bcd8c3d4/java/com/craftinginterpreters/lox/Scanner.java#L114-L129
[s-erro]: https://github.com/munificent/craftinginterpreters/blob/4a840f70f69c6ddd17cfef4f6964f8e1bcd8c3d4/java/com/craftinginterpreters/lox/Scanner.java#L126
[s-identifier]: https://github.com/munificent/craftinginterpreters/blob/4a840f70f69c6ddd17cfef4f6964f8e1bcd8c3d4/java/com/craftinginterpreters/lox/Scanner.java#L135-L147
[s-string]: https://github.com/munificent/craftinginterpreters/blob/4a840f70f69c6ddd17cfef4f6964f8e1bcd8c3d4/java/com/craftinginterpreters/lox/Scanner.java#L168
[s-match]: https://github.com/munificent/craftinginterpreters/blob/4a840f70f69c6ddd17cfef4f6964f8e1bcd8c3d4/java/com/craftinginterpreters/lox/Scanner.java#L186-L192
[s-addtoken]: https://github.com/munificent/craftinginterpreters/blob/4a840f70f69c6ddd17cfef4f6964f8e1bcd8c3d4/java/com/craftinginterpreters/lox/Scanner.java#L236-L239
[l-run]: https://github.com/munificent/craftinginterpreters/blob/4a840f70f69c6ddd17cfef4f6964f8e1bcd8c3d4/java/com/craftinginterpreters/lox/Lox.java#L64-L106
[l-hadError]: https://github.com/munificent/craftinginterpreters/blob/4a840f70f69c6ddd17cfef4f6964f8e1bcd8c3d4/java/com/craftinginterpreters/lox/Lox.java#L84
[l-report]: https://github.com/munificent/craftinginterpreters/blob/4a840f70f69c6ddd17cfef4f6964f8e1bcd8c3d4/java/com/craftinginterpreters/lox/Lox.java#L109-L118
[t-token]: https://github.com/munificent/craftinginterpreters/blob/4a840f70f69c6ddd17cfef4f6964f8e1bcd8c3d4/java/com/craftinginterpreters/lox/Token.java#L4-L20
[m-tokens]: https://github.com/samuka7abr/monk/blob/d11fdc7cdd9f344161a6acb953d43ff4a12080fe/src/lexer.rs#L27-L52
[m-posicao]: https://github.com/samuka7abr/monk/blob/d11fdc7cdd9f344161a6acb953d43ff4a12080fe/src/lexer.rs#L32
[m-reservada]: https://github.com/samuka7abr/monk/blob/d11fdc7cdd9f344161a6acb953d43ff4a12080fe/src/lexer.rs#L39
[m-erro]: https://github.com/samuka7abr/monk/blob/d11fdc7cdd9f344161a6acb953d43ff4a12080fe/src/lexer.rs#L43-L47
[m-avancar]: https://github.com/samuka7abr/monk/blob/d11fdc7cdd9f344161a6acb953d43ff4a12080fe/src/lexer.rs#L67-L77
[m-operador]: https://github.com/samuka7abr/monk/blob/d11fdc7cdd9f344161a6acb953d43ff4a12080fe/src/lexer.rs#L106-L137
[m-palavra]: https://github.com/samuka7abr/monk/blob/d11fdc7cdd9f344161a6acb953d43ff4a12080fe/src/token.rs#L99-L112
[m-token]: https://github.com/samuka7abr/monk/blob/d11fdc7cdd9f344161a6acb953d43ff4a12080fe/src/token.rs#L79-L84
[m-main]: https://github.com/samuka7abr/monk/blob/d11fdc7cdd9f344161a6acb953d43ff4a12080fe/src/main.rs#L45-L52
