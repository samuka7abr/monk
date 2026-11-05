# Gramática da MiniLang

Gramática final da MiniLang, pronta para parsing descendente recursivo (LL(1)).
Começou como rascunho na M02 e foi fechada na M08. A linguagem é descrita em
`docs/especificacao.md`; em caso de dúvida, a especificação manda.

## 1. Notação

A gramática está em EBNF:

| Símbolo | Significado |
|---|---|
| `a = ... ;` | define o não-terminal `a` |
| `{ x }` | zero ou mais repetições de `x` |
| `[ x ]` | `x` opcional (zero ou uma vez) |
| `a \| b` | alternativa: `a` ou `b` |
| `( ... )` | agrupamento |
| `"texto"` | terminal (token de lexema fixo) |
| `NUM`, `ID`, `EOF` | tokens definidos pelo léxico |
| `ε` | cadeia vazia (usado só nas explicações) |
| `⇒` | um passo de derivação (usado só nas explicações) |

Os terminais são **tokens**, não caracteres: `"<="` é o token `MENOR_IGUAL`, que o
lexer já entrega pronto (seção 2.6 da especificação).

## 2. Gramática sintática

```
programa  = { comando } EOF ;
comando   = decl | atrib | print | if | while | bloco ;
decl      = "var" ID ":" tipo "=" expr ";" ;
tipo      = "int" | "bool" ;
atrib     = ID "=" expr ";" ;
print     = "print" "(" expr ")" ";" ;
if        = "if" "(" expr ")" bloco [ "else" ( bloco | if ) ] ;
while     = "while" "(" expr ")" bloco ;
bloco     = "{" { comando } "}" ;
expr      = soma [ op_comp soma ] ;
op_comp   = "==" | "!=" | "<" | ">" | "<=" | ">=" ;
soma      = termo { ( "+" | "-" ) termo } ;
termo     = unario { ( "*" | "/" ) unario } ;
unario    = "-" unario | fator ;
fator     = NUM | "true" | "false" | ID | "(" expr ")" ;
```

Cada não-terminal vira uma função do parser (`programa()`, `comando()`, ...). Um
`{ }` vira um laço `while`, um `[ ]` vira um `if`, e uma `|` vira um `match` sobre
o token atual.

## 3. Gramática léxica

Em expressões regulares:

```
ID          = [A-Za-z_][A-Za-z0-9_]*
NUM         = [0-9]+
comentario  = "//" [^\n]*
espaco      = [ \t\r\n]+
```

`comentario` e `espaco` são descartados pelo lexer. As palavras reservadas
(`var int bool true false if else while print`) casam com a regra de `ID`, mas
produzem seus próprios tokens. Os símbolos casam de forma gulosa (`<=` antes de
`<`). Não existe token `--`: `--x` são dois `MENOS` seguidos de `ID`.

## 4. Precedência e associatividade

### 4.1 Um não-terminal por nível

A precedência da especificação (seção 4.1) está codificada na gramática, um
não-terminal por nível, do mais baixo ao mais alto:

| Nível | Não-terminal | Operadores | Associatividade | Construção na gramática |
|---|---|---|---|---|
| 1 | `expr` | `==` `!=` `<` `>` `<=` `>=` | não associativa | `[ ]`: no máximo um operador |
| 2 | `soma` | `+` `-` | esquerda | `{ }`: laço |
| 3 | `termo` | `*` `/` | esquerda | `{ }`: laço |
| 4 | `unario` | `-` (prefixo) | empilha (`--x`) | recursão à direita |
| — | `fator` | literais, `ID`, `( expr )` | — | folhas |

Cada nível só enxerga diretamente o nível de cima. Um operador de nível mais alto
fica mais fundo na árvore e por isso liga primeiro. Os parênteses voltam ao nível
1 (`"(" expr ")"`) e servem para sobrepor a precedência.

### 4.2 Precedência: `1 + 2 * 3`

Árvore de derivação (só a parte de `expr`):

```
expr
└── soma
    ├── termo
    │   └── unario ── fator ── NUM 1
    ├── "+"
    └── termo
        ├── unario ── fator ── NUM 2
        ├── "*"
        └── unario ── fator ── NUM 3
```

O resultado é `1 + (2 * 3)`. **Não existe outra árvore**: o `*` só aparece dentro
de um `termo`, e os operandos de `*` são `unario`, que não contém `+` a não ser
entre parênteses. Logo `1 + 2` nunca pode ser operando de `*`. Para obter
`(1 + 2) * 3`, os parênteses são obrigatórios.

### 4.3 Associatividade: `1 - 2 - 3`

A repetição `{ }` sozinha descreve uma lista plana (`termo - termo - termo`). Quem
dá a estrutura é a gramática recursiva à esquerda equivalente
`soma = soma "-" termo | termo` (seção 5), que admite **uma única árvore**:

```
soma
├── soma
│   ├── soma ── termo ── unario ── fator ── NUM 1
│   ├── "-"
│   └── termo ── unario ── fator ── NUM 2
├── "-"
└── termo ── unario ── fator ── NUM 3
```

Ou seja, `(1 - 2) - 3`. A árvore é única porque o operando direito de cada `-` é
um `termo`, e um `termo` não contém `-` binário fora de parênteses. Então o
último `-` é necessariamente a raiz, e o mesmo argumento se repete à esquerda.
`1 - (2 - 3)` só é escrevível com parênteses.

### 4.4 Unário

`unario = "-" unario | fator` é recursiva à direita, o que não atrapalha o
parsing descendente (o `-` é consumido antes da chamada recursiva) e permite
empilhar:

```
unario ⇒ "-" unario ⇒ "-" "-" unario ⇒ "-" "-" fator ⇒ "-" "-" ID      (--x)
```

Como `unario` fica abaixo de `termo`, `-x * y` é `(-x) * y`. Em `1 - -4`, o
primeiro `-` é binário (está no laço de `soma`, logo depois de um `termo`) e o
segundo é unário (começa um `termo` novo). As duas posições nunca se confundem,
como mostra a seção 7.

O texto da issue M08 punha o unário em
`fator = ... | "-" fator`. As duas formas geram a mesma linguagem; mantemos o
nível `unario` próprio porque ele corresponde ao nível 4 da tabela de precedência
da especificação.

## 5. Eliminação da recursão à esquerda

A forma natural de escrever um operador associativo à esquerda é com recursão à
esquerda:

```
soma  = soma "+" termo | soma "-" termo | termo ;
termo = termo "*" unario | termo "/" unario | unario ;
```

Essa forma não serve para parsing descendente:

- **recursão infinita**: `soma()` começaria chamando `soma()` sem consumir
  nenhum token;
- **não é LL(1)**: as duas alternativas começam com o FIRST de `termo`, então um
  token não basta para escolher entre elas.

A regra geral transforma `A = A α | β` em `A = β { α }`. As duas formas geram a
mesma linguagem: um `β` seguido de zero ou mais `α`. Aplicada aqui:

```
soma  = soma ( "+" | "-" ) termo | termo        ⇒   soma  = termo { ( "+" | "-" ) termo } ;
termo = termo ( "*" | "/" ) unario | unario     ⇒   termo = unario { ( "*" | "/" ) unario } ;
```

**O laço preserva a associatividade à esquerda.** O parser implementa o `{ }`
como um laço que acumula o resultado à esquerda:

```
fn soma():
    esq = termo()
    enquanto o token atual é "+" ou "-":
        op  = consome o operador
        dir = termo()
        esq = Binario(op, esq, dir)     // o que já foi lido vira o operando esquerdo
    devolve esq
```

Para `1 - 2 - 3`: `esq = 1`, depois `esq = (1 - 2)`, depois `esq = ((1 - 2) - 3)`,
exatamente a árvore da seção 4.3.

## 6. Conjuntos FIRST

`FIRST(α)` é o conjunto de tokens que podem começar uma cadeia derivada de `α`.

### 6.1 Alternativas de `comando`

| Alternativa | FIRST |
|---|---|
| `decl` | `"var"` |
| `atrib` | `ID` |
| `print` | `"print"` |
| `if` | `"if"` |
| `while` | `"while"` |
| `bloco` | `"{"` |

Os seis conjuntos são disjuntos e têm um único token cada, então o token atual
decide a alternativa. `FIRST(comando)` é a união deles:
`{ "var", ID, "print", "if", "while", "{" }`. Qualquer outro token no início de um
comando, como `NUM` em `1 = x;`, é erro sintático.

### 6.2 Alternativas de `fator` e `unario`

| Não-terminal | Alternativa | FIRST |
|---|---|---|
| `fator` | `NUM` | `NUM` |
| `fator` | `"true"` | `"true"` |
| `fator` | `"false"` | `"false"` |
| `fator` | `ID` | `ID` |
| `fator` | `"(" expr ")"` | `"("` |
| `unario` | `"-" unario` | `"-"` |
| `unario` | `fator` | `NUM`, `"true"`, `"false"`, `ID`, `"("` |

As alternativas de `fator` são disjuntas entre si, e as de `unario` também (`"-"`
não começa nenhum `fator`).

Como `expr`, `soma` e `termo` começam todos por `unario`:

```
FIRST(expr) = FIRST(soma) = FIRST(termo) = FIRST(unario)
            = { "-", NUM, "true", "false", ID, "(" }
```

Um token fora desse conjunto onde se espera uma expressão, como o `;` em
`x = 1 + ;`, é erro sintático.

## 7. Por que a gramática não é ambígua (LL(1))

Uma gramática é LL(1) quando, em todo ponto de escolha, o próximo token decide o
caminho. Toda gramática LL(1) é não ambígua: o parser nunca tem duas opções, e
por isso cada programa tem uma única árvore.

Os pontos de escolha são as alternativas `|` (seção 6) e as construções `{ }` e
`[ ]`. Nestas últimas, o parser decide entre entrar (o token está no FIRST do
corpo) ou sair (o token está no FOLLOW, o conjunto de tokens que podem vir
depois). A gramática é LL(1) se esses dois conjuntos forem disjuntos:

| Ponto de escolha | Entra se o token for | Sai se o token for (FOLLOW) |
|---|---|---|
| `{ comando }` em `programa` | `FIRST(comando)` | `EOF` |
| `{ comando }` em `bloco` | `FIRST(comando)` | `"}"` |
| `[ "else" ... ]` em `if` | `"else"` | `FIRST(comando)`, `"}"`, `EOF` |
| `( bloco \| if )` depois de `else` | `"{"` ou `"if"` | — (não é opcional) |
| `[ op_comp soma ]` em `expr` | `"=="` `"!="` `"<"` `">"` `"<="` `">="` | `")"`, `";"` |
| `{ ( "+" \| "-" ) termo }` em `soma` | `"+"`, `"-"` | `op_comp`, `")"`, `";"` |
| `{ ( "*" \| "/" ) unario }` em `termo` | `"*"`, `"/"` | `"+"`, `"-"`, `op_comp`, `")"`, `";"` |
| `tipo` | `"int"` ou `"bool"` | — (não é opcional) |

Em todas as linhas, as duas colunas são disjuntas. Em particular, o `"-"` está no
FOLLOW de `termo` (é o `-` binário de `soma`), mas não no FOLLOW de `soma`. Por
isso o laço de `soma` sabe que todo `-` encontrado depois de um `termo` é
binário.

**Conferência mecânica.** Os conjuntos acima foram calculados à mão e conferidos
com um script descartável. O script converte a EBNF para BNF (cada `{ X }` vira
`R = X R | ε`, cada `[ X ]` vira `O = X | ε`), calcula FIRST e FOLLOW por ponto
fixo e monta a tabela LL(1). O resultado foi **zero conflitos**.

## 8. Decisões de projeto

- **Chaves obrigatórias eliminam o *else pendente*.** Em C,
  `if (a) if (b) s1; else s2;` é ambíguo: o `else` pode ser do `if` de fora ou do
  de dentro. Na MiniLang todo ramo é um `bloco`, então o `if` de dentro termina
  num `}` antes do `else` aparecer. Em `if (a) { if (b) { } } else { }`, o `else`
  só pode ser do `if` de fora. Na tabela da seção 7, `"else"` não pertence ao
  FOLLOW de `if`, e é isso que torna a escolha determinística.
- **`else if` é açúcar sintático.** `"else" if` equivale a `"else" "{" if "}"`. O
  parser não cria nó novo: embrulha o `if` encadeado num `Bloco` de um comando só
  e o usa como `senao`, então a AST da M10 não muda. Cada `if` tem no máximo um
  `else`, então numa cadeia `if ... else if ... else ...` cada `else` pertence ao
  `if` imediatamente anterior.
- **Comparação não associativa.** `expr = soma [ op_comp soma ]` admite no máximo
  um operador de comparação, logo `a < b < c` não tem derivação. Depois de
  `1 < 2`, `expr` termina e o próximo token tem de estar no FOLLOW de `expr`
  (`")"` ou `";"`). Em `var b: bool = 1 < 2 < 3;`, o parser espera `;` e encontra
  `<`. A linguagem não tem como expressar o sentido matemático de `a < b < c`
  (`a < b` e `b < c`), porque não há operadores lógicos. Como `==` e `!=` aceitam
  dois `bool` (seção 4.2 da especificação), dá para comparar os resultados de
  comparações usando parênteses: `(a < b) == (c < d)`.
- **Parênteses e chaves obrigatórios em `if`/`while`.** Eles delimitam a condição
  e o corpo sem precisar de outra regra.

## 9. Derivação mais à esquerda

Programa (2 linhas):

```
var x: int = 1 + 2;
print(x);
```

Abreviações, para caber na linha:

| Abreviação | Significa | Expansões possíveis |
|---|---|---|
| `C*` | `{ comando }` | `⇒ comando C*` ou `⇒ ε` |
| `E'` | `[ op_comp soma ]` | `⇒ op_comp soma` ou `⇒ ε` |
| `S'` | `{ ( "+" \| "-" ) termo }` | `⇒ "+" termo S'`, `⇒ "-" termo S'` ou `⇒ ε` |
| `T'` | `{ ( "*" \| "/" ) unario }` | `⇒ "*" unario T'`, `⇒ "/" unario T'` ou `⇒ ε` |
| `α` | `"var" ID ":" "int" "=" NUM "+" NUM ";"` | (a 1ª linha já derivada) |

Em cada passo é expandido o não-terminal **mais à esquerda**:

```
 1  programa
 2  ⇒ C* EOF                                                    programa
 3  ⇒ comando C* EOF                                            C* ⇒ comando C*
 4  ⇒ decl C* EOF                                               comando ⇒ decl
 5  ⇒ "var" ID ":" tipo "=" expr ";" C* EOF                     decl
 6  ⇒ "var" ID ":" "int" "=" expr ";" C* EOF                    tipo ⇒ "int"
 7  ⇒ "var" ID ":" "int" "=" soma E' ";" C* EOF                 expr
 8  ⇒ "var" ID ":" "int" "=" termo S' E' ";" C* EOF             soma
 9  ⇒ "var" ID ":" "int" "=" unario T' S' E' ";" C* EOF         termo
10  ⇒ "var" ID ":" "int" "=" fator T' S' E' ";" C* EOF          unario ⇒ fator
11  ⇒ "var" ID ":" "int" "=" NUM T' S' E' ";" C* EOF            fator ⇒ NUM
12  ⇒ "var" ID ":" "int" "=" NUM S' E' ";" C* EOF               T' ⇒ ε
13  ⇒ "var" ID ":" "int" "=" NUM "+" termo S' E' ";" C* EOF     S' ⇒ "+" termo S'
14  ⇒ "var" ID ":" "int" "=" NUM "+" unario T' S' E' ";" C* EOF termo
15  ⇒ "var" ID ":" "int" "=" NUM "+" fator T' S' E' ";" C* EOF  unario ⇒ fator
16  ⇒ "var" ID ":" "int" "=" NUM "+" NUM T' S' E' ";" C* EOF    fator ⇒ NUM
17  ⇒ "var" ID ":" "int" "=" NUM "+" NUM S' E' ";" C* EOF       T' ⇒ ε
18  ⇒ "var" ID ":" "int" "=" NUM "+" NUM E' ";" C* EOF          S' ⇒ ε
19  ⇒ α C* EOF                                                  E' ⇒ ε
20  ⇒ α comando C* EOF                                          C* ⇒ comando C*
21  ⇒ α print C* EOF                                            comando ⇒ print
22  ⇒ α "print" "(" expr ")" ";" C* EOF                         print
23  ⇒ α "print" "(" soma E' ")" ";" C* EOF                      expr
24  ⇒ α "print" "(" termo S' E' ")" ";" C* EOF                  soma
25  ⇒ α "print" "(" unario T' S' E' ")" ";" C* EOF              termo
26  ⇒ α "print" "(" fator T' S' E' ")" ";" C* EOF               unario ⇒ fator
27  ⇒ α "print" "(" ID T' S' E' ")" ";" C* EOF                  fator ⇒ ID
28  ⇒ α "print" "(" ID S' E' ")" ";" C* EOF                     T' ⇒ ε
29  ⇒ α "print" "(" ID E' ")" ";" C* EOF                        S' ⇒ ε
30  ⇒ α "print" "(" ID ")" ";" C* EOF                           E' ⇒ ε
31  ⇒ α "print" "(" ID ")" ";" EOF                              C* ⇒ ε
```

A forma final é exatamente a sequência de tokens que o lexer produz para o
programa (`monk prog.mk --tokens`):

```
VAR ID(x) DOIS_PONTOS INT ATRIB NUM(1) MAIS NUM(2) PONTO_VIRG
PRINT ABRE_PAR ID(x) FECHA_PAR PONTO_VIRG EOF
```

Cada passo foi decidido olhando um único token. Por exemplo, no passo 12 o token
atual é `+`, que não está em `FIRST(T') = { "*", "/" }`, então `T' ⇒ ε`. No passo
13, o mesmo `+` está em `FIRST(S')`, então o laço de `soma` continua.
