# Especificação da MiniLang

> Este documento está congelado ao fim da issue M02. Qualquer mudança exige
> atualizar o lexer, a gramática (`docs/gramatica.md`) e os testes, e avisar a equipe.

MiniLang é uma linguagem imperativa didática, com sintaxe próxima de C, Go e Rust.
Programas usam a extensão `.mini`.

## 1. Estrutura do programa

Um programa é uma sequência de comandos, possivelmente vazia. Não há funções:
a execução começa no primeiro comando e termina depois do último.

## 2. Léxico

### 2.1 Fonte e espaços

O código-fonte é texto UTF-8. Espaço, `\t`, `\r` e `\n` separam tokens e são
ignorados.

### 2.2 Comentários

`//` inicia um comentário que vai até o fim da linha. O comentário pode conter
qualquer caractere. Não existe comentário de bloco.

### 2.3 Identificadores

```
[A-Za-z_][A-Za-z0-9_]*
```

Identificadores diferenciam maiúsculas de minúsculas (`x` e `X` são nomes distintos).

### 2.4 Palavras reservadas

`var` `int` `bool` `true` `false` `if` `else` `while` `print`

Uma palavra reservada nunca é identificador.

### 2.5 Literais inteiros

```
[0-9]+
```

Somente decimal e sem sinal: o `-` é o operador unário (seção 4). O valor deve
caber em `i64`, ou seja, estar em `0..=9223372036854775807`. O lexer não valida o
valor; o parser sim, e um valor maior produz o erro sintático
`literal inteiro fora do intervalo`. Consequência: o menor `i64`
(`-9223372036854775808`) não é escrevível como literal.

Os literais booleanos são as palavras reservadas `true` e `false`.

### 2.6 Símbolos

```
+  -  *  /  =  ==  !=  <  >  <=  >=  (  )  {  }  :  ;
```

O casamento é guloso (máxima munição): `<=` é um único token, nunca `<` seguido de
`=`; `==` nunca é `=` `=`. O mesmo vale para `>=` e `!=`.

### 2.7 Tabela de tokens

| Token | Lexema |
|---|---|
| `NUM` | literal inteiro |
| `ID` | identificador |
| `VAR` | `var` |
| `INT` | `int` |
| `BOOL` | `bool` |
| `TRUE` | `true` |
| `FALSE` | `false` |
| `IF` | `if` |
| `ELSE` | `else` |
| `WHILE` | `while` |
| `PRINT` | `print` |
| `MAIS` | `+` |
| `MENOS` | `-` |
| `VEZES` | `*` |
| `DIV` | `/` |
| `ATRIB` | `=` |
| `IGUAL` | `==` |
| `DIFERENTE` | `!=` |
| `MENOR` | `<` |
| `MAIOR` | `>` |
| `MENOR_IGUAL` | `<=` |
| `MAIOR_IGUAL` | `>=` |
| `ABRE_PAR` | `(` |
| `FECHA_PAR` | `)` |
| `ABRE_CHAVE` | `{` |
| `FECHA_CHAVE` | `}` |
| `DOIS_PONTOS` | `:` |
| `PONTO_VIRG` | `;` |
| `EOF` | fim da entrada (lexema vazio) |

### 2.8 Caracteres inválidos e posições

Qualquer caractere fora de comentário que não inicie um token da seção 2.7 é erro
léxico. Isso inclui um `!` isolado, que só é válido como parte de `!=`.

Linha e coluna começam em 1 e contam caracteres, não bytes. O `\t` conta como uma
coluna.

## 3. Tipos

| Tipo | Valores |
|---|---|
| `int` | inteiro de 64 bits com sinal, em complemento de dois |
| `bool` | `true` e `false` |

- O overflow de `int` dá a volta (wrapping) e nunca é erro.
- Não há conversão implícita entre `int` e `bool`, em nenhum contexto.

## 4. Expressões

### 4.1 Precedência e associatividade

Do nível mais baixo ao mais alto:

| Nível | Operadores | Associatividade |
|---|---|---|
| 1 (mais baixo) | `==` `!=` `<` `>` `<=` `>=` | não associativa |
| 2 | `+` `-` (binários) | esquerda |
| 3 | `*` `/` | esquerda |
| 4 (mais alto) | `-` (unário) | prefixo; pode empilhar |

- Parênteses agrupam e sobrepõem a precedência.
- Comparação não associativa significa que `a < b < c` e `a == b == c` são erros
  sintáticos. Para encadear, use parênteses: `(a < b) == (c < d)`.
- O `-` unário pode ser empilhado: `--x` e `- -x` valem `x`.
- `a - b - c` é `(a - b) - c`; `a / b * c` é `(a / b) * c`.

### 4.2 Tipos dos operadores

| Operador | Operandos | Resultado |
|---|---|---|
| `+` `-` `*` `/` | `int`, `int` | `int` |
| `<` `>` `<=` `>=` | `int`, `int` | `bool` |
| `==` `!=` | `T`, `T`, com `T` igual a `int` ou `bool` | `bool` |
| `-` (unário) | `int` | `int` |

Em `==` e `!=` os dois lados têm de ter o mesmo tipo: `1 == true` é erro. Os demais
casos não listados na tabela são erros semânticos.

### 4.3 Ordem de avaliação

Os operandos são avaliados da esquerda para a direita.

### 4.4 Divisão

`/` é divisão inteira que trunca em direção a zero: `-7 / 2 == -3` e
`7 / -2 == -3`. Divisão por zero é erro de execução, não de compilação, mesmo
quando o divisor é uma constante (`1 / 0` compila e falha ao executar).

## 5. Comandos

| Comando | Forma |
|---|---|
| Declaração | `var nome: tipo = expr;` |
| Atribuição | `nome = expr;` |
| Impressão | `print(expr);` |
| Condicional | `if (cond) { ... }`, `if (cond) { ... } else { ... }` ou `if (cond) { ... } else if (cond) { ... } else { ... }` |
| Repetição | `while (cond) { ... }` |
| Bloco | `{ ... }` |

- **Declaração.** Sempre inicializada; `tipo` é `int` ou `bool` e a expressão deve
  ter esse tipo.
- **Atribuição.** A variável já deve estar declarada e a expressão deve ter o tipo
  da variável.
- **`print`.** Imprime o valor da expressão seguido de quebra de linha. Um `bool`
  é impresso como `true` ou `false`.
- **`if` e `while`.** Parênteses em volta da condição e chaves em volta dos blocos
  são obrigatórios. A condição deve ser `bool`. O `else` é seguido de um bloco ou
  de outro `if`, o que permite encadear `else if` sem limite. O `else if` é
  açúcar sintático: `if (a) { A } else if (b) { B } else { C }` equivale a
  `if (a) { A } else { if (b) { B } else { C } }`. O parser representa o `if`
  interno como o único comando de um bloco `else`, então o ramo `else if` também
  abre um escopo.
- **Bloco solto.** `{ ... }` é um comando e abre um escopo novo.

## 6. Escopos

- Existe um escopo global (nível 0). Cada `{ }`, seja de `if`, `else`, `while` ou
  bloco solto, abre um escopo novo.
- Uma variável é visível do fim da sua declaração até o fim do bloco que a
  contém. Usá-la antes de declarar é erro.
- Redeclarar um nome no mesmo escopo é erro.
- Declarar o mesmo nome em um escopo interno é permitido (sombreamento): a nova
  variável esconde a externa até o fim do bloco interno, e a externa volta a
  valer depois dele.
- Na declaração `var x: T = e;`, o `x` novo não é visível dentro de `e`. A
  expressão `e` enxerga um `x` externo, se existir:

```
var x: int = 1;
{
    var x: int = x + 1;
    print(x);              // 2
}
```

## 7. Semântica de execução

- A execução percorre os comandos em ordem a partir do primeiro.
- `while` reavalia a condição antes de cada iteração.
- Aritmética `int` em 64 bits com wrapping; `/` conforme a seção 4.4.
- A saída de `print` vai para a saída padrão, um valor por linha.

## 8. Erros

As fases rodam em ordem: léxica, sintática, semântica, execução. Uma fase só roda
se a anterior não teve erro.

| Fase | Comportamento | Formato |
|---|---|---|
| Léxico | reporta todos os erros | `{linha}:{coluna}: erro léxico: caractere inválido '{c}'` |
| Sintático | para no primeiro erro | `{linha}:{coluna}: erro sintático: {msg}, encontrado '{lexema}'` |
| Semântico | reporta todos os erros | `linha {linha}: erro semântico: {msg}` |
| Execução | para no primeiro erro | `linha {linha}: erro de execução: divisão por zero` |

Os erros vão para stderr e o processo sai com código 1.

Nos erros sintáticos, a posição é a do token encontrado e o lexema vazio (fim da
entrada) aparece como `EOF`. As mensagens seguem o estilo `esperado '<símbolo>'`:

- `esperado ';'`
- `esperado ')'`
- `esperado '{'`
- `esperado expressão`
- `literal inteiro fora do intervalo`

Mensagens semânticas padronizadas:

| Situação | Mensagem |
|---|---|
| uso de nome não declarado | `'{x}' não foi declarada` |
| redeclaração no mesmo escopo | `'{x}' já declarada neste escopo` |
| atribuição ou declaração com tipo errado | `'{x}' é {tipo}, mas recebe {tipo}` |
| condição que não é `bool` | `condição deve ser bool, encontrado {tipo}` |
| operador binário com tipos inválidos | `operador '{op}' inválido para {tipo} e {tipo}` |
| `-` unário sobre não `int` | `operador '-' inválido para {tipo}` |

## 9. Exemplo

```
// Fatorial de 5: imprime 120
var n: int = 5;
var f: int = 1;
while (n > 0) {
    f = f * n;
    n = n - 1;
}
print(f);
```

Mais exemplos em `exemplos/`.

## 10. Extensões opcionais (fora do escopo obrigatório)

Nenhum item desta seção faz parte da linguagem congelada. Só podem ser
implementados depois das fases obrigatórias, com atualização de lexer, gramática
e testes.

- Operadores lógicos `&&`, `||` e `!`
- Laço `for`
- Operador `%` (resto)
- Comentário de bloco `/* ... */`
- Strings no `print`
