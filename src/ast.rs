//! Árvore sintática abstrata (`Programa`, `Comando`, `Expr`, ...) e seu pretty-printer. Preenchido em M10 e M14.

use std::fmt;

/// Tipo de uma variável ou expressão (seção 3 da especificação).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tipo {
    Int,
    Bool,
}

impl fmt::Display for Tipo {
    /// Imprime o nome do tipo como no código-fonte (`int` ou `bool`).
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let nome: &'static str = match self {
            Tipo::Int => "int",
            Tipo::Bool => "bool",
        };
        f.pad(nome)
    }
}

/// Operador binário: aritmético ou de comparação.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpBin {
    Mais,
    Menos,
    Vezes,
    Div,
    Igual,
    Diferente,
    Menor,
    Maior,
    MenorIgual,
    MaiorIgual,
}

impl OpBin {
    /// Símbolo do operador no código-fonte (`"+"`, `"<="`, ...).
    pub fn simbolo(&self) -> &'static str {
        match self {
            OpBin::Mais => "+",
            OpBin::Menos => "-",
            OpBin::Vezes => "*",
            OpBin::Div => "/",
            OpBin::Igual => "==",
            OpBin::Diferente => "!=",
            OpBin::Menor => "<",
            OpBin::Maior => ">",
            OpBin::MenorIgual => "<=",
            OpBin::MaiorIgual => ">=",
        }
    }
}

/// Operador unário: só o menos prefixo.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpUn {
    Neg,
}

/// Raiz da AST: a sequência de comandos do arquivo.
#[derive(Debug, Clone, PartialEq)]
pub struct Programa {
    pub comandos: Vec<Comando>,
}

/// Bloco `{ ... }`; `linha` é a do `{`. Cada bloco abre um escopo.
#[derive(Debug, Clone, PartialEq)]
pub struct Bloco {
    pub comandos: Vec<Comando>,
    pub linha: usize,
}

/// Comando com a linha em que começa.
#[derive(Debug, Clone, PartialEq)]
pub struct Comando {
    pub kind: ComandoKind,
    pub linha: usize,
}

/// Formas de comando, uma por alternativa de `comando` na gramática.
///
/// `else if` não tem variante própria: o `if` encadeado vira o único comando do
/// bloco `senao`.
#[derive(Debug, Clone, PartialEq)]
pub enum ComandoKind {
    Decl {
        nome: String,
        tipo: Tipo,
        expr: Expr,
    },
    Atrib {
        nome: String,
        expr: Expr,
    },
    Print {
        expr: Expr,
    },
    If {
        cond: Expr,
        entao: Bloco,
        senao: Option<Bloco>,
    },
    While {
        cond: Expr,
        corpo: Bloco,
    },
    Bloco(Bloco),
}

/// Expressão com a linha e o tipo, que nasce `None` e é preenchido pela semântica (M18).
#[derive(Debug, Clone, PartialEq)]
pub struct Expr {
    pub kind: ExprKind,
    pub linha: usize,
    pub tipo: Option<Tipo>,
}

/// Formas de expressão. A precedência fica no formato da árvore, não em nós próprios,
/// e os parênteses não geram nó.
#[derive(Debug, Clone, PartialEq)]
pub enum ExprKind {
    Num(i64),
    Bool(bool),
    Var(String),
    Unario {
        op: OpUn,
        expr: Box<Expr>,
    },
    Binario {
        op: OpBin,
        esq: Box<Expr>,
        dir: Box<Expr>,
    },
}

impl Expr {
    /// Cria uma expressão ainda sem tipo.
    pub fn new(kind: ExprKind, linha: usize) -> Self {
        Expr {
            kind,
            linha,
            tipo: None,
        }
    }

    /// Literal inteiro.
    pub fn num(valor: i64, linha: usize) -> Self {
        Expr::new(ExprKind::Num(valor), linha)
    }

    /// Literal booleano.
    pub fn bool(valor: bool, linha: usize) -> Self {
        Expr::new(ExprKind::Bool(valor), linha)
    }

    /// Uso de variável.
    pub fn var(nome: impl Into<String>, linha: usize) -> Self {
        Expr::new(ExprKind::Var(nome.into()), linha)
    }

    /// Operação unária `op expr`.
    pub fn unario(op: OpUn, expr: Expr, linha: usize) -> Self {
        Expr::new(
            ExprKind::Unario {
                op,
                expr: Box::new(expr),
            },
            linha,
        )
    }

    /// Operação binária `esq op dir`.
    pub fn binario(op: OpBin, esq: Expr, dir: Expr, linha: usize) -> Self {
        Expr::new(
            ExprKind::Binario {
                op,
                esq: Box::new(esq),
                dir: Box::new(dir),
            },
            linha,
        )
    }
}

impl Comando {
    /// Cria um comando a partir de suas partes.
    pub fn new(kind: ComandoKind, linha: usize) -> Self {
        Comando { kind, linha }
    }
}
