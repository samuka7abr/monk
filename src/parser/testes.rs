use super::*;
use crate::ast::ExprKind;
use crate::lexer::Lexer;

fn parser(fonte: &str) -> Parser {
    Parser::new(Lexer::new(fonte).tokens())
}

fn expr(fonte: &str) -> Expr {
    parser(fonte)
        .expressao()
        .unwrap_or_else(|e| panic!("{fonte:?}: {e}"))
}

fn erro(fonte: &str) -> String {
    match parser(fonte).expressao() {
        Ok(e) => panic!("{fonte:?} deveria falhar, mas deu {e:?}"),
        Err(e) => e.to_string(),
    }
}

/// Forma parentetizada da árvore, para comparar a estrutura de forma legível.
fn forma(e: &Expr) -> String {
    match &e.kind {
        ExprKind::Num(n) => n.to_string(),
        ExprKind::Bool(b) => b.to_string(),
        ExprKind::Var(x) => x.clone(),
        ExprKind::Unario { expr, .. } => format!("(-{})", forma(expr)),
        ExprKind::Binario { op, esq, dir } => {
            format!("({} {} {})", forma(esq), op.simbolo(), forma(dir))
        }
    }
}

fn arvore(fonte: &str) -> String {
    forma(&expr(fonte))
}

// ------------------------------------------------------------------ utilitários

#[test]
fn consumir_nao_passa_do_eof() {
    let mut p = parser("x");
    assert_eq!(p.consumir().tipo, TipoToken::Id);
    assert_eq!(p.consumir().tipo, TipoToken::Eof);
    assert_eq!(p.consumir().tipo, TipoToken::Eof);
    assert_eq!(p.atual().tipo, TipoToken::Eof);
}

#[test]
fn verificar_nao_consome() {
    let p = parser("+");
    assert!(p.verificar(&[TipoToken::Menos, TipoToken::Mais]));
    assert!(!p.verificar(&[TipoToken::Vezes]));
    assert_eq!(p.atual().tipo, TipoToken::Mais);
}

#[test]
fn esperar_formata_erro() {
    let mut p = parser("  x");
    assert_eq!(
        p.esperar(TipoToken::PontoVirg, "esperado ';'")
            .unwrap_err()
            .to_string(),
        "1:3: erro sintático: esperado ';', encontrado 'x'"
    );
    assert!(p.esperar(TipoToken::Id, "esperado identificador").is_ok());
    assert_eq!(
        p.esperar(TipoToken::PontoVirg, "esperado ';'")
            .unwrap_err()
            .mensagem,
        "1:4: erro sintático: esperado ';', encontrado 'EOF'"
    );
}

#[test]
fn lista_vazia_ganha_eof() {
    let mut p = Parser::new(Vec::new());
    assert_eq!(p.atual().tipo, TipoToken::Eof);
    assert_eq!(
        p.expressao().unwrap_err().mensagem,
        "1:1: erro sintático: esperado expressão, encontrado 'EOF'"
    );
}

// ------------------------------------------------------------------ expressões

#[test]
fn fatores() {
    assert_eq!(expr("42"), Expr::num(42, 1));
    assert_eq!(expr("true"), Expr::bool(true, 1));
    assert_eq!(expr("false"), Expr::bool(false, 1));
    assert_eq!(expr("contador"), Expr::var("contador", 1));
    assert!(expr("x").tipo.is_none());
}

#[test]
fn precedencia() {
    let e = expr("1 + 2 * 3");
    let ExprKind::Binario { op, esq, dir } = &e.kind else {
        panic!("raiz deveria ser binária: {e:?}");
    };
    assert_eq!(*op, OpBin::Mais);
    assert_eq!(**esq, Expr::num(1, 1));
    assert!(matches!(
        dir.kind,
        ExprKind::Binario {
            op: OpBin::Vezes,
            ..
        }
    ));
    assert_eq!(arvore("1 + 2 * 3"), "(1 + (2 * 3))");
    assert_eq!(arvore("1 * 2 + 3"), "((1 * 2) + 3)");
    assert_eq!(arvore("1 + 2 < 3 * 4"), "((1 + 2) < (3 * 4))");
    assert_eq!(arvore("-x * y"), "((-x) * y)");
}

#[test]
fn associatividade_esquerda() {
    let e = expr("10 - 3 - 2");
    let ExprKind::Binario { op, esq, dir } = &e.kind else {
        panic!("raiz deveria ser binária: {e:?}");
    };
    assert_eq!(*op, OpBin::Menos);
    assert_eq!(**dir, Expr::num(2, 1));
    assert!(matches!(
        esq.kind,
        ExprKind::Binario {
            op: OpBin::Menos,
            ..
        }
    ));
    assert_eq!(arvore("10 - 3 - 2"), "((10 - 3) - 2)");
    assert_eq!(arvore("8 / 4 / 2"), "((8 / 4) / 2)");
    assert_eq!(arvore("1 + 2 - 3 + 4"), "(((1 + 2) - 3) + 4)");
}

#[test]
fn unario() {
    assert_eq!(
        expr("- -4"),
        Expr::unario(OpUn::Neg, Expr::unario(OpUn::Neg, Expr::num(4, 1), 1), 1)
    );
    assert_eq!(arvore("1 - -4"), "(1 - (-4))");
    assert_eq!(arvore("-(1 + 2)"), "(-(1 + 2))");
}

#[test]
fn parenteses() {
    assert_eq!(arvore("(1 + 2) * 3"), "((1 + 2) * 3)");
    assert_eq!(arvore("10 - (3 - 2)"), "(10 - (3 - 2))");
    assert_eq!(arvore("((x))"), "x");
    assert_eq!(arvore("(1 < 2) == true"), "((1 < 2) == true)");
}

#[test]
fn comparacoes() {
    for (fonte, op) in [
        ("a == b", OpBin::Igual),
        ("a != b", OpBin::Diferente),
        ("a < b", OpBin::Menor),
        ("a > b", OpBin::Maior),
        ("a <= b", OpBin::MenorIgual),
        ("a >= b", OpBin::MaiorIgual),
    ] {
        assert_eq!(
            expr(fonte),
            Expr::binario(op, Expr::var("a", 1), Expr::var("b", 1), 1),
            "{fonte}"
        );
    }
}

#[test]
fn comparacao_nao_associativa() {
    // `expressao` só lê uma comparação; o `<` que sobra fica para quem chamou.
    let mut p = parser("1 < 2 < 3");
    assert_eq!(forma(&p.expressao().unwrap()), "(1 < 2)");
    assert_eq!(p.atual().tipo, TipoToken::Menor);
}

#[test]
fn linha_do_operador() {
    let e = expr("a\n+\nb");
    assert_eq!(e.linha, 2);
    let e = expr("\n\n-\nx");
    assert_eq!(e.linha, 3);
    let ExprKind::Unario { expr: x, .. } = e.kind else {
        panic!("deveria ser unário");
    };
    assert_eq!(x.linha, 4);
}

#[test]
fn expressao_esperada() {
    assert_eq!(
        erro("1 + ;"),
        "1:5: erro sintático: esperado expressão, encontrado ';'"
    );
    assert_eq!(
        erro("2 *"),
        "1:4: erro sintático: esperado expressão, encontrado 'EOF'"
    );
    assert_eq!(
        erro(")"),
        "1:1: erro sintático: esperado expressão, encontrado ')'"
    );
}

#[test]
fn falta_fecha_parenteses() {
    assert_eq!(
        erro("(1 + 2;"),
        "1:7: erro sintático: esperado ')', encontrado ';'"
    );
}

#[test]
fn literal_fora_do_intervalo() {
    assert_eq!(expr("9223372036854775807"), Expr::num(i64::MAX, 1));
    assert_eq!(
        erro("1 + 9223372036854775808"),
        "1:5: erro sintático: literal inteiro fora do intervalo, encontrado '9223372036854775808'"
    );
}

#[test]
fn sempre_termina() {
    // Entradas malformadas não podem travar o parser: ou dão Ok ou dão Err.
    for fonte in [
        "", "-", "- - -", "((((", "(1", "1 +", "1 < ", "+ + +", "1 2 3",
    ] {
        let _ = parser(fonte).expressao();
    }
}

// ------------------------------------------------------------------ comandos e programa

fn programa(fonte: &str) -> Programa {
    parser(fonte)
        .programa()
        .unwrap_or_else(|e| panic!("{fonte:?}: {e}"))
}

fn erro_programa(fonte: &str) -> String {
    match parser(fonte).programa() {
        Ok(p) => panic!("{fonte:?} deveria falhar, mas deu {p:?}"),
        Err(e) => e.to_string(),
    }
}

fn bloco(comandos: Vec<Comando>, linha: usize) -> Bloco {
    Bloco { comandos, linha }
}

fn print_num(n: i64, linha: usize) -> Comando {
    Comando::new(
        ComandoKind::Print {
            expr: Expr::num(n, linha),
        },
        linha,
    )
}

#[test]
fn programa_vazio() {
    assert_eq!(programa("").comandos, vec![]);
    assert_eq!(programa("  // só comentário\n").comandos, vec![]);
}

#[test]
fn declaracao() {
    assert_eq!(
        programa("var x: int = 1 + 2;").comandos,
        vec![Comando::new(
            ComandoKind::Decl {
                nome: "x".into(),
                tipo: Tipo::Int,
                expr: Expr::binario(OpBin::Mais, Expr::num(1, 1), Expr::num(2, 1), 1),
            },
            1,
        )]
    );
    let ComandoKind::Decl { tipo, .. } = &programa("var b: bool = true;").comandos[0].kind else {
        panic!("deveria ser Decl");
    };
    assert_eq!(*tipo, Tipo::Bool);
}

#[test]
fn atribuicao_e_print() {
    assert_eq!(
        programa("x = 3;\nprint(x);").comandos,
        vec![
            Comando::new(
                ComandoKind::Atrib {
                    nome: "x".into(),
                    expr: Expr::num(3, 1),
                },
                1,
            ),
            Comando::new(
                ComandoKind::Print {
                    expr: Expr::var("x", 2),
                },
                2,
            ),
        ]
    );
}

#[test]
fn if_sem_else() {
    assert_eq!(
        programa("if (true) {\n  print(1);\n}").comandos,
        vec![Comando::new(
            ComandoKind::If {
                cond: Expr::bool(true, 1),
                entao: bloco(vec![print_num(1, 2)], 1),
                senao: None,
            },
            1,
        )]
    );
}

#[test]
fn if_com_else() {
    assert_eq!(
        programa("if (false) { print(1); }\nelse { print(2); }").comandos,
        vec![Comando::new(
            ComandoKind::If {
                cond: Expr::bool(false, 1),
                entao: bloco(vec![print_num(1, 1)], 1),
                senao: Some(bloco(vec![print_num(2, 2)], 2)),
            },
            1,
        )]
    );
}

#[test]
fn else_if_vira_bloco_com_um_if() {
    let fonte = "if (a) { }\nelse if (b) { }\nelse { print(3); }";
    let interno = Comando::new(
        ComandoKind::If {
            cond: Expr::var("b", 2),
            entao: bloco(vec![], 2),
            senao: Some(bloco(vec![print_num(3, 3)], 3)),
        },
        2,
    );
    assert_eq!(
        programa(fonte).comandos,
        vec![Comando::new(
            ComandoKind::If {
                cond: Expr::var("a", 1),
                entao: bloco(vec![], 1),
                senao: Some(bloco(vec![interno], 2)),
            },
            1,
        )]
    );
}

#[test]
fn while_() {
    assert_eq!(
        programa("while (i < 3) {\n  i = i + 1;\n}").comandos,
        vec![Comando::new(
            ComandoKind::While {
                cond: Expr::binario(OpBin::Menor, Expr::var("i", 1), Expr::num(3, 1), 1),
                corpo: bloco(
                    vec![Comando::new(
                        ComandoKind::Atrib {
                            nome: "i".into(),
                            expr: Expr::binario(OpBin::Mais, Expr::var("i", 2), Expr::num(1, 2), 2,),
                        },
                        2,
                    )],
                    1,
                ),
            },
            1,
        )]
    );
}

#[test]
fn bloco_aninhado() {
    assert_eq!(
        programa("{\n  {\n    print(1);\n  }\n  {}\n}").comandos,
        vec![Comando::new(
            ComandoKind::Bloco(bloco(
                vec![
                    Comando::new(ComandoKind::Bloco(bloco(vec![print_num(1, 3)], 2)), 2),
                    Comando::new(ComandoKind::Bloco(bloco(vec![], 5)), 5),
                ],
                1,
            )),
            1,
        )]
    );
}

#[test]
fn falta_ponto_virgula() {
    let msg = erro_programa("var x: int = 1");
    assert!(msg.contains("esperado ';'"), "{msg}");
    assert!(msg.contains("encontrado 'EOF'"), "{msg}");
    assert_eq!(msg, "1:15: erro sintático: esperado ';', encontrado 'EOF'");
}

#[test]
fn erros_de_comando() {
    for (fonte, esperado) in [
        (
            "1 = x;",
            "1:1: erro sintático: comando inesperado, encontrado '1'",
        ),
        (
            "x = 1; }",
            "1:8: erro sintático: comando inesperado, encontrado '}'",
        ),
        (
            "var x int = 1;",
            "1:7: erro sintático: esperado ':', encontrado 'int'",
        ),
        (
            "var 1x: int = 0;",
            "1:5: erro sintático: esperado identificador, encontrado '1'",
        ),
        (
            "var x: 1 = 0;",
            "1:8: erro sintático: esperado tipo, encontrado '1'",
        ),
        (
            "var x: int 0;",
            "1:12: erro sintático: esperado '=', encontrado '0'",
        ),
        ("x 1;", "1:3: erro sintático: esperado '=', encontrado '1'"),
        (
            "print x;",
            "1:7: erro sintático: esperado '(', encontrado 'x'",
        ),
        (
            "print(x;",
            "1:8: erro sintático: esperado ')', encontrado ';'",
        ),
        (
            "if x > 1 { }",
            "1:4: erro sintático: esperado '(', encontrado 'x'",
        ),
        (
            "if (x) print(x);",
            "1:8: erro sintático: esperado '{', encontrado 'print'",
        ),
        (
            "if (x) { } else print(x);",
            "1:17: erro sintático: esperado '{', encontrado 'print'",
        ),
        (
            "while (x) { x = x - 1;",
            "1:23: erro sintático: esperado '}', encontrado 'EOF'",
        ),
        (
            "var b: bool = 1 < 2 < 3;",
            "1:21: erro sintático: esperado ';', encontrado '<'",
        ),
    ] {
        assert_eq!(erro_programa(fonte), esperado, "{fonte}");
    }
}

#[test]
fn programa_sempre_termina() {
    for fonte in [
        "}",
        "{",
        "else",
        "if",
        "if (",
        "while (x) {",
        "var",
        ";;",
        "{ } }",
        "print",
    ] {
        let _ = parser(fonte).programa();
    }
}
