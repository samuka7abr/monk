//! Estresse da AST: árvores muito profundas não podem estourar a pilha.
//!
//! Cada caso roda numa thread própria de 2 MB. Assim o resultado não depende de
//! `RUST_MIN_STACK` nem de `--test-threads=1`, que executa os testes na thread principal.
//! As comparações usam `assert!(a == b)` porque `assert_eq!` formataria a árvore com
//! `Debug`, que continua recursivo (veja a documentação de `Expr`).

mod comum;

use std::thread;

use monk::ast::*;

const OPERADORES: usize = 1_000_000;
const PILHA_DE_TESTE: usize = 2 * 1024 * 1024;

/// Executa `caso` numa thread com exatamente 2 MB de pilha.
fn com_pilha_de_teste(caso: impl FnOnce() + Send + 'static) {
    thread::Builder::new()
        .stack_size(PILHA_DE_TESTE)
        .spawn(caso)
        .expect("criar a thread do teste")
        .join()
        .expect("o caso de estresse falhou");
}

/// `1 + 1 + ...`: associativa à esquerda, como o parser monta.
fn cadeia_esquerda(operadores: usize) -> Expr {
    let mut e = Expr::num(1, 1);
    for _ in 0..operadores {
        e = Expr::binario(OpBin::Mais, e, Expr::num(1, 1), 1);
    }
    e
}

/// `1 + (1 + (...))`: a profundidade fica no filho direito.
fn cadeia_direita(operadores: usize) -> Expr {
    let mut e = Expr::var("x", 1);
    for _ in 0..operadores {
        e = Expr::binario(OpBin::Mais, Expr::num(1, 1), e, 1);
    }
    e
}

/// `- - - ... x`.
fn cadeia_unaria(operadores: usize) -> Expr {
    let mut e = Expr::var("x", 1);
    for _ in 0..operadores {
        e = Expr::unario(OpUn::Neg, e, 1);
    }
    e
}

/// Fonte `print(1 + 1 + ... + 1);`, com `operadores` somas.
fn fonte_com_somas(operadores: usize) -> String {
    let mut fonte = String::with_capacity(2 * operadores + 16);
    fonte.push_str("print(1");
    for _ in 0..operadores {
        fonte.push_str("+1");
    }
    fonte.push_str(");");
    fonte
}

#[test]
fn drop_de_programa_real_com_um_milhao_de_operadores() {
    com_pilha_de_teste(|| {
        let programa = comum::parse(&fonte_com_somas(OPERADORES)).expect("programa válido");
        assert_eq!(programa.comandos.len(), 1);
    });
}

#[test]
fn drop_no_caminho_de_erro_sintatico() {
    com_pilha_de_teste(|| {
        // Falta o último operando: a árvore parcial é descartada dentro do parser.
        let mut fonte = fonte_com_somas(OPERADORES);
        fonte.truncate(fonte.len() - 3);
        fonte.push_str(");");
        let erro = comum::parse(&fonte).expect_err("falta um operando");
        assert!(erro.mensagem.contains("esperado expressão"), "{erro}");
    });
}

#[test]
fn drop_de_cadeias_a_direita_e_unaria() {
    com_pilha_de_teste(|| {
        drop(cadeia_direita(OPERADORES));
        drop(cadeia_unaria(OPERADORES));
    });
}

#[test]
fn clone_e_igualdade_de_cadeias_profundas() {
    com_pilha_de_teste(|| {
        for original in [
            cadeia_esquerda(OPERADORES),
            cadeia_direita(OPERADORES),
            cadeia_unaria(OPERADORES),
        ] {
            let copia = original.clone();
            assert!(copia == original);
        }
    });
}

#[test]
fn igualdade_detecta_diferenca_no_fundo_da_arvore() {
    com_pilha_de_teste(|| {
        let mut a = Expr::num(1, 1);
        let mut b = Expr::num(2, 1);
        for _ in 0..OPERADORES {
            a = Expr::binario(OpBin::Mais, a, Expr::num(1, 1), 1);
            b = Expr::binario(OpBin::Mais, b, Expr::num(1, 1), 1);
        }
        assert!(a != b);
    });
}

#[test]
fn clone_preserva_linha_e_tipo_de_cada_no() {
    let mut esq = Expr::var("x", 2);
    esq.tipo = Some(Tipo::Int);
    let mut negado = Expr::unario(OpUn::Neg, Expr::num(7, 3), 4);
    negado.tipo = Some(Tipo::Int);
    let mut raiz = Expr::binario(OpBin::Menor, esq, negado, 5);
    raiz.tipo = Some(Tipo::Bool);

    let copia = raiz.clone();
    assert_eq!(copia, raiz);
    assert_eq!(format!("{copia:?}"), format!("{raiz:?}"));
}

#[test]
fn igualdade_compara_linha_tipo_e_operador() {
    let base = Expr::binario(OpBin::Mais, Expr::num(1, 1), Expr::num(2, 1), 1);

    let outra_linha = Expr::binario(OpBin::Mais, Expr::num(1, 9), Expr::num(2, 1), 1);
    assert_ne!(base, outra_linha);

    let outro_op = Expr::binario(OpBin::Menos, Expr::num(1, 1), Expr::num(2, 1), 1);
    assert_ne!(base, outro_op);

    let mut outro_tipo = base.clone();
    outro_tipo.tipo = Some(Tipo::Int);
    assert_ne!(base, outro_tipo);

    assert_ne!(Expr::num(1, 1), Expr::bool(true, 1));
    assert_ne!(Expr::var("a", 1), Expr::var("b", 1));
    assert_ne!(
        Expr::unario(OpUn::Neg, Expr::num(1, 1), 1),
        Expr::unario(OpUn::Neg, Expr::num(2, 1), 1)
    );
}

#[test]
fn debug_de_arvore_normal_nao_muda() {
    let mut e = Expr::binario(
        OpBin::Mais,
        Expr::var("x", 1),
        Expr::unario(OpUn::Neg, Expr::num(2, 1), 1),
        1,
    );
    e.tipo = Some(Tipo::Int);
    assert_eq!(
        format!("{e:?}"),
        "Expr { kind: Binario { op: Mais, \
         esq: Expr { kind: Var(\"x\"), linha: 1, tipo: None }, \
         dir: Expr { kind: Unario { op: Neg, \
         expr: Expr { kind: Num(2), linha: 1, tipo: None } }, linha: 1, tipo: None } }, \
         linha: 1, tipo: Some(Int) }"
    );
}
