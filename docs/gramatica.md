# Gramática da MiniLang (rascunho)

Este é um rascunho em EBNF, escrito na M02 e refinado na M08 (gramática final,
tabelas FIRST e derivação). A linguagem é descrita em `docs/especificacao.md`.

## Notação

| Símbolo | Significado |
|---|---|
| `{ x }` | zero ou mais repetições de `x` |
| `[ x ]` | `x` opcional |
| `a \| b` | alternativa |
| `"texto"` | terminal (token de lexema fixo) |
| `NUM`, `ID`, `EOF` | tokens definidos pelo léxico |

## Gramática sintática

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

## Gramática léxica

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
`<`).

## Precedência e associatividade

A gramática codifica a precedência com um não-terminal por nível: `expr`
(comparação, o mais baixo), `soma`, `termo` e `unario` (o mais alto), com `fator`
nas folhas. Um operador de nível mais alto fica mais fundo na árvore e liga primeiro.

- **Associatividade à esquerda** de `+ - * /`: vem da repetição `{ }`, que o parser
  consome em laço, montando a árvore para a esquerda (`a - b - c` vira
  `(a - b) - c`).
- **Comparação não associativa**: vem do `[ ]` em `expr`, que admite no máximo um
  `op_comp`. Por isso `a < b < c` não tem derivação.
- **Unário prefixo**: `unario = "-" unario` é recursivo à direita, o que permite
  empilhar (`--x`).

## `else if`

`"else" if` é açúcar sintático para `"else" "{" if "}"`. O parser não cria um nó
novo: embrulha o `if` encadeado num `Bloco` de um comando só e o usa como
`senao`, então a AST de `M10` não muda. O `else` pendurado não é ambíguo, porque
todo ramo exige chaves: cada `else` pertence ao `if` cujo bloco acabou de fechar.

## Análise

A gramática é LL(1): cada escolha é decidida pelo próximo token. Em `comando`,
`var`, `ID`, `print`, `if`, `while` e `{` distinguem as alternativas; em `if`, o
`else` decide se o ramo opcional entra e, depois dele, `{` ou `if` escolhe entre
bloco e `else if`. Ela cabe num parser descendente recursivo,
com uma função por não-terminal. O detalhamento dos conjuntos FIRST fica para a M08.
