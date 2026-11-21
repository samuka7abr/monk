//! Árvore sintática abstrata (`Programa`, `Comando`, `Expr`, ...) e seu pretty-printer. Preenchido em M10 e M14.

use std::{fmt, mem};

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
///
/// Uma cadeia como `1 + 1 + ...` gera uma árvore tão funda quanto o número de operadores,
/// então `Clone`, `PartialEq` e o `Drop` de [`ExprKind`] são iterativos.
///
/// **Limite:** `Debug` continua derivado e recursivo. Com 8 MB de pilha, `{:?}` aborta o
/// processo a partir de cerca de 10 mil níveis, e `{:#?}` já leva mais de 20 s com mil
/// níveis. Não formate com `Debug` (nem use `assert_eq!`) a AST de um programa do usuário.
#[derive(Debug)]
pub struct Expr {
    pub kind: ExprKind,
    pub linha: usize,
    pub tipo: Option<Tipo>,
}

/// Formas de expressão. A precedência fica no formato da árvore, não em nós próprios,
/// e os parênteses não geram nó.
///
/// Como implementa `Drop`, seus campos não podem ser movidos por valor (E0509): use
/// `match &e.kind`. Mover o `ExprKind` inteiro para fora de um `Expr` continua permitido.
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

impl Drop for ExprKind {
    /// Desmonta a árvore com uma pilha explícita, sem recursão.
    fn drop(&mut self) {
        let mut pendentes = Vec::new();
        mover_filhos(self, &mut pendentes);
        while let Some(mut kind) = pendentes.pop() {
            // Sem filhos compostos, o `drop` de `kind` ao fim do laço não aprofunda.
            mover_filhos(&mut kind, &mut pendentes);
        }
    }
}

/// Move para `pendentes` os filhos compostos de `kind`, deixando folhas no lugar.
fn mover_filhos(kind: &mut ExprKind, pendentes: &mut Vec<ExprKind>) {
    let mut mover = |filho: &mut Expr| {
        if matches!(
            filho.kind,
            ExprKind::Unario { .. } | ExprKind::Binario { .. }
        ) {
            pendentes.push(mem::replace(&mut filho.kind, ExprKind::Num(0)));
        }
    };
    match kind {
        ExprKind::Num(_) | ExprKind::Bool(_) | ExprKind::Var(_) => {}
        ExprKind::Unario { expr, .. } => mover(expr),
        ExprKind::Binario { esq, dir, .. } => {
            mover(esq);
            mover(dir);
        }
    }
}

impl Clone for Expr {
    /// Copia a árvore de cima para baixo, com uma pilha explícita.
    fn clone(&self) -> Self {
        let mut raiz = copia_rasa(self);
        let mut pendentes: Vec<(&Expr, &mut Expr)> = vec![(self, &mut raiz)];
        while let Some((origem, destino)) = pendentes.pop() {
            match (&origem.kind, &mut destino.kind) {
                (ExprKind::Unario { expr: o, .. }, ExprKind::Unario { expr: d, .. }) => {
                    **d = copia_rasa(o);
                    pendentes.push((o, d));
                }
                (
                    ExprKind::Binario {
                        esq: oe, dir: od, ..
                    },
                    ExprKind::Binario {
                        esq: de, dir: dd, ..
                    },
                ) => {
                    **de = copia_rasa(oe);
                    **dd = copia_rasa(od);
                    pendentes.push((oe, de));
                    pendentes.push((od, dd));
                }
                _ => {}
            }
        }
        raiz
    }
}

/// Copia um nó; os filhos ficam como folhas provisórias, trocadas por `Expr::clone`.
fn copia_rasa(e: &Expr) -> Expr {
    let provisorio = || Box::new(Expr::num(0, 0));
    let kind = match &e.kind {
        ExprKind::Num(n) => ExprKind::Num(*n),
        ExprKind::Bool(b) => ExprKind::Bool(*b),
        ExprKind::Var(nome) => ExprKind::Var(nome.clone()),
        ExprKind::Unario { op, .. } => ExprKind::Unario {
            op: *op,
            expr: provisorio(),
        },
        ExprKind::Binario { op, .. } => ExprKind::Binario {
            op: *op,
            esq: provisorio(),
            dir: provisorio(),
        },
    };
    Expr {
        kind,
        linha: e.linha,
        tipo: e.tipo,
    }
}

impl PartialEq for Expr {
    /// Compara as duas árvores nó a nó, com uma pilha explícita.
    fn eq(&self, outro: &Self) -> bool {
        let mut pendentes = vec![(self, outro)];
        while let Some((a, b)) = pendentes.pop() {
            if a.linha != b.linha || a.tipo != b.tipo {
                return false;
            }
            match (&a.kind, &b.kind) {
                (ExprKind::Num(x), ExprKind::Num(y)) if x == y => {}
                (ExprKind::Bool(x), ExprKind::Bool(y)) if x == y => {}
                (ExprKind::Var(x), ExprKind::Var(y)) if x == y => {}
                (ExprKind::Unario { op: x, expr: ex }, ExprKind::Unario { op: y, expr: ey })
                    if x == y =>
                {
                    pendentes.push((ex, ey));
                }
                (
                    ExprKind::Binario {
                        op: x,
                        esq: ex,
                        dir: dx,
                    },
                    ExprKind::Binario {
                        op: y,
                        esq: ey,
                        dir: dy,
                    },
                ) if x == y => {
                    pendentes.push((ex, ey));
                    pendentes.push((dx, dy));
                }
                _ => return false,
            }
        }
        true
    }
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
