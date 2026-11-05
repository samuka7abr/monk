use super::*;

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
