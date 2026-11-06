//! Testes de integração da AST: montam árvores à mão pela API pública de `monk::ast`.

use monk::ast::*;

/// AST de `x = 1 + 2 * 3;`, montada à mão como o parser deve montá-la.
fn atribuicao_exemplo() -> Comando {
    let produto = Expr::binario(OpBin::Vezes, Expr::num(2, 1), Expr::num(3, 1), 1);
    let soma = Expr::binario(OpBin::Mais, Expr::num(1, 1), produto, 1);
    Comando::new(
        ComandoKind::Atrib {
            nome: "x".into(),
            expr: soma,
        },
        1,
    )
}

#[test]
fn atribuicao_com_precedencia() {
    let cmd = atribuicao_exemplo();
    assert_eq!(cmd.linha, 1);

    let ComandoKind::Atrib { nome, expr } = &cmd.kind else {
        panic!("esperava Atrib, veio {:?}", cmd.kind);
    };
    assert_eq!(nome, "x");

    // A raiz é o `+`; o `*` fica mais fundo, como operando direito.
    let ExprKind::Binario { op, esq, dir } = &expr.kind else {
        panic!("esperava Binario, veio {:?}", expr.kind);
    };
    assert_eq!(*op, OpBin::Mais);
    assert_eq!(esq.kind, ExprKind::Num(1));

    let ExprKind::Binario { op, esq, dir } = &dir.kind else {
        panic!("esperava Binario, veio {:?}", dir.kind);
    };
    assert_eq!(*op, OpBin::Vezes);
    assert_eq!(esq.kind, ExprKind::Num(2));
    assert_eq!(dir.kind, ExprKind::Num(3));
}

#[test]
fn construtores_equivalem_a_literais() {
    let esperado = Expr {
        kind: ExprKind::Binario {
            op: OpBin::Mais,
            esq: Box::new(Expr {
                kind: ExprKind::Num(1),
                linha: 1,
                tipo: None,
            }),
            dir: Box::new(Expr {
                kind: ExprKind::Binario {
                    op: OpBin::Vezes,
                    esq: Box::new(Expr {
                        kind: ExprKind::Num(2),
                        linha: 1,
                        tipo: None,
                    }),
                    dir: Box::new(Expr {
                        kind: ExprKind::Num(3),
                        linha: 1,
                        tipo: None,
                    }),
                },
                linha: 1,
                tipo: None,
            }),
        },
        linha: 1,
        tipo: None,
    };
    let ComandoKind::Atrib { expr, .. } = atribuicao_exemplo().kind else {
        unreachable!()
    };
    assert_eq!(expr, esperado);
}

#[test]
fn expressoes_nascem_sem_tipo() {
    for e in [
        Expr::num(7, 2),
        Expr::bool(true, 2),
        Expr::var("y", 2),
        Expr::unario(OpUn::Neg, Expr::num(1, 2), 2),
        Expr::binario(OpBin::Menor, Expr::var("a", 2), Expr::num(0, 2), 2),
    ] {
        assert_eq!(e.tipo, None);
        assert_eq!(e.linha, 2);
    }
}

#[test]
fn unario_empilhado() {
    // --x é Neg(Neg(x))
    let e = Expr::unario(OpUn::Neg, Expr::unario(OpUn::Neg, Expr::var("x", 1), 1), 1);
    let ExprKind::Unario { op, expr } = &e.kind else {
        panic!("esperava Unario, veio {:?}", e.kind);
    };
    assert_eq!(*op, OpUn::Neg);
    assert!(matches!(expr.kind, ExprKind::Unario { op: OpUn::Neg, .. }));
}

#[test]
fn else_if_vira_bloco_com_um_if() {
    // if (a) { } else if (b) { }
    let interno = Comando::new(
        ComandoKind::If {
            cond: Expr::var("b", 1),
            entao: Bloco {
                comandos: vec![],
                linha: 1,
            },
            senao: None,
        },
        1,
    );
    let externo = Comando::new(
        ComandoKind::If {
            cond: Expr::var("a", 1),
            entao: Bloco {
                comandos: vec![],
                linha: 1,
            },
            senao: Some(Bloco {
                comandos: vec![interno.clone()],
                linha: 1,
            }),
        },
        1,
    );
    let ComandoKind::If {
        senao: Some(bloco), ..
    } = &externo.kind
    else {
        panic!("esperava If com else");
    };
    assert_eq!(bloco.comandos, vec![interno]);
}

#[test]
fn simbolos_dos_operadores() {
    use OpBin::*;
    let pares = [
        (Mais, "+"),
        (Menos, "-"),
        (Vezes, "*"),
        (Div, "/"),
        (Igual, "=="),
        (Diferente, "!="),
        (Menor, "<"),
        (Maior, ">"),
        (MenorIgual, "<="),
        (MaiorIgual, ">="),
    ];
    for (op, simbolo) in pares {
        assert_eq!(op.simbolo(), simbolo);
    }
}

#[test]
fn tipo_display() {
    assert_eq!(Tipo::Int.to_string(), "int");
    assert_eq!(Tipo::Bool.to_string(), "bool");
    assert_eq!(format!("{:>5}", Tipo::Int), "  int");
}

/// Linhas 2–12 de `exemplos/if_else.mk`: o fatorial seguido de um `if/else`.
const FATORIAL_IF_ELSE: &str = "\
var n: int = 5;
var f: int = 1;
while (n > 0) {
    f = f * n;
    n = n - 1;
}
if (f > 100) {
    print(f);
} else {
    print(0);
}
";

/// AST de `FATORIAL_IF_ELSE`, com as linhas do arquivo de exemplo (a 1ª linha é
/// o comentário). A linha de um binário é a do operador.
fn fatorial_if_else() -> Programa {
    let decl = |nome: &str, valor, linha| {
        Comando::new(
            ComandoKind::Decl {
                nome: nome.into(),
                tipo: Tipo::Int,
                expr: Expr::num(valor, linha),
            },
            linha,
        )
    };
    let atrib_bin = |nome: &str, op, dir, linha| {
        Comando::new(
            ComandoKind::Atrib {
                nome: nome.into(),
                expr: Expr::binario(op, Expr::var(nome, linha), dir, linha),
            },
            linha,
        )
    };
    let print = |expr: Expr| {
        let linha = expr.linha;
        Comando::new(ComandoKind::Print { expr }, linha)
    };

    Programa {
        comandos: vec![
            decl("n", 5, 2),
            decl("f", 1, 3),
            Comando::new(
                ComandoKind::While {
                    cond: Expr::binario(OpBin::Maior, Expr::var("n", 4), Expr::num(0, 4), 4),
                    corpo: Bloco {
                        comandos: vec![
                            atrib_bin("f", OpBin::Vezes, Expr::var("n", 5), 5),
                            atrib_bin("n", OpBin::Menos, Expr::num(1, 6), 6),
                        ],
                        linha: 4,
                    },
                },
                4,
            ),
            Comando::new(
                ComandoKind::If {
                    cond: Expr::binario(OpBin::Maior, Expr::var("f", 8), Expr::num(100, 8), 8),
                    entao: Bloco {
                        comandos: vec![print(Expr::var("f", 9))],
                        linha: 8,
                    },
                    senao: Some(Bloco {
                        comandos: vec![print(Expr::num(0, 11))],
                        linha: 10,
                    }),
                },
                8,
            ),
        ],
    }
}

#[test]
fn programa_completo() {
    // O trecho montado à mão continua igual ao do exemplo.
    let exemplo = include_str!("../exemplos/if_else.mk");
    assert!(
        exemplo.contains(FATORIAL_IF_ELSE),
        "exemplos/if_else.mk mudou"
    );

    let prog = fatorial_if_else();
    assert_eq!(prog.comandos.len(), 4);

    // var n: int = 5;  var f: int = 1;
    for (cmd, nome_esperado, valor, linha) in [
        (&prog.comandos[0], "n", 5, 2),
        (&prog.comandos[1], "f", 1, 3),
    ] {
        assert_eq!(cmd.linha, linha);
        let ComandoKind::Decl { nome, tipo, expr } = &cmd.kind else {
            panic!("esperava Decl, veio {:?}", cmd.kind);
        };
        assert_eq!(nome, nome_esperado);
        assert_eq!(*tipo, Tipo::Int);
        assert_eq!(expr.kind, ExprKind::Num(valor));
    }

    // while (n > 0) { f = f * n; n = n - 1; }
    let ComandoKind::While { cond, corpo } = &prog.comandos[2].kind else {
        panic!("esperava While, veio {:?}", prog.comandos[2].kind);
    };
    assert_eq!(prog.comandos[2].linha, 4);
    assert_eq!(
        *cond,
        Expr::binario(OpBin::Maior, Expr::var("n", 4), Expr::num(0, 4), 4)
    );
    assert_eq!(corpo.linha, 4);
    assert_eq!(corpo.comandos.len(), 2);
    let linhas: Vec<usize> = corpo.comandos.iter().map(|c| c.linha).collect();
    assert_eq!(linhas, [5, 6]);
    let ComandoKind::Atrib { nome, expr } = &corpo.comandos[0].kind else {
        panic!("esperava Atrib, veio {:?}", corpo.comandos[0].kind);
    };
    assert_eq!(nome, "f");
    let ExprKind::Binario { op, esq, dir } = &expr.kind else {
        panic!("esperava Binario, veio {:?}", expr.kind);
    };
    assert_eq!(*op, OpBin::Vezes);
    assert_eq!(esq.kind, ExprKind::Var("f".into()));
    assert_eq!(dir.kind, ExprKind::Var("n".into()));

    // if (f > 100) { print(f); } else { print(0); }
    let ComandoKind::If { cond, entao, senao } = &prog.comandos[3].kind else {
        panic!("esperava If, veio {:?}", prog.comandos[3].kind);
    };
    assert_eq!(prog.comandos[3].linha, 8);
    assert!(matches!(
        cond.kind,
        ExprKind::Binario {
            op: OpBin::Maior,
            ..
        }
    ));
    assert_eq!(
        entao.comandos,
        [Comando::new(
            ComandoKind::Print {
                expr: Expr::var("f", 9)
            },
            9
        )]
    );
    let senao = senao.as_ref().expect("esperava else");
    assert_eq!(senao.linha, 10);
    assert_eq!(
        senao.comandos,
        [Comando::new(
            ComandoKind::Print {
                expr: Expr::num(0, 11)
            },
            11
        )]
    );

    // Nada foi anotado ainda: a semântica (M18) é quem preenche `tipo`.
    fn sem_tipo(e: &Expr) -> bool {
        e.tipo.is_none()
            && match &e.kind {
                ExprKind::Unario { expr, .. } => sem_tipo(expr),
                ExprKind::Binario { esq, dir, .. } => sem_tipo(esq) && sem_tipo(dir),
                _ => true,
            }
    }
    assert!(sem_tipo(cond));
}
