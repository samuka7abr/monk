use super::*;

fn lexar(fonte: &str) -> Vec<Token> {
    Lexer::new(fonte).tokens()
}

fn tipos(fonte: &str) -> Vec<TipoToken> {
    lexar(fonte).into_iter().map(|t| t.tipo).collect()
}

fn lexemas(fonte: &str) -> Vec<String> {
    lexar(fonte).into_iter().map(|t| t.lexema).collect()
}

#[test]
fn declaracao_completa() {
    use TipoToken::*;
    assert_eq!(
        tipos("var x: int = 10;"),
        vec![Var, Id, DoisPontos, Int, Atrib, Num, PontoVirg, Eof]
    );
    assert_eq!(
        lexemas("var x: int = 10;"),
        vec!["var", "x", ":", "int", "=", "10", ";", ""]
    );
}

#[test]
fn reservada_x_identificador() {
    use TipoToken::*;
    assert_eq!(
        tipos("while whilex _x x1 If"),
        vec![While, Id, Id, Id, Id, Eof]
    );
}

#[test]
fn operadores_dois_caracteres() {
    use TipoToken::*;
    assert_eq!(
        tipos("== != <= >= = < > "),
        vec![Igual, Diferente, MenorIgual, MaiorIgual, Atrib, Menor, Maior, Eof]
    );
    assert_eq!(tipos("a<=b"), vec![Id, MenorIgual, Id, Eof]);
    assert_eq!(tipos("x==y"), vec![Id, Igual, Id, Eof]);
    assert_eq!(lexemas("a<=b"), vec!["a", "<=", "b", ""]);
}

#[test]
fn operadores_um_caractere() {
    use TipoToken::*;
    assert_eq!(tipos("+ - * /"), vec![Mais, Menos, Vezes, Div, Eof]);
}

#[test]
fn delimitadores() {
    use TipoToken::*;
    assert_eq!(
        tipos("( ) { } : ;"),
        vec![AbrePar, FechaPar, AbreChave, FechaChave, DoisPontos, PontoVirg, Eof]
    );
}

#[test]
fn posicao() {
    let toks = lexar("var\n  x");
    assert_eq!((toks[0].linha, toks[0].coluna), (1, 1));
    assert_eq!((toks[1].linha, toks[1].coluna), (2, 3));
    assert_eq!((toks[2].linha, toks[2].coluna), (2, 4));

    let toks = lexar("\tx\ty");
    assert_eq!((toks[0].linha, toks[0].coluna), (1, 2));
    assert_eq!((toks[1].linha, toks[1].coluna), (1, 4));
}

#[test]
fn comentario() {
    let toks = lexar("// tudo isto some @#!\nprint");
    assert_eq!(
        toks.iter().map(|t| t.tipo).collect::<Vec<_>>(),
        vec![TipoToken::Print, TipoToken::Eof]
    );
    assert_eq!((toks[0].linha, toks[0].coluna), (2, 1));

    assert_eq!(
        tipos("x // fim sem quebra"),
        vec![TipoToken::Id, TipoToken::Eof]
    );
    assert_eq!(
        tipos("a / b // c"),
        vec![TipoToken::Id, TipoToken::Div, TipoToken::Id, TipoToken::Eof]
    );
}

#[test]
fn numero_varios_digitos() {
    assert_eq!(lexemas("12345")[0], "12345");
    assert_eq!(lexemas("007")[0], "007");
    let enorme = "1234567890123456789012345";
    let toks = lexar(enorme);
    assert_eq!(toks[0].tipo, TipoToken::Num);
    assert_eq!(toks[0].lexema, enorme);
}

#[test]
fn entrada_vazia() {
    let toks = lexar("");
    assert_eq!(toks.len(), 1);
    assert_eq!(toks[0].tipo, TipoToken::Eof);
    assert_eq!((toks[0].linha, toks[0].coluna), (1, 1));

    let toks = lexar("   \n ");
    assert_eq!(toks.len(), 1);
    assert_eq!(toks[0].tipo, TipoToken::Eof);
    assert_eq!((toks[0].linha, toks[0].coluna), (2, 2));
}

#[test]
fn eof_lexema_vazio() {
    let toks = lexar("x");
    let eof = toks.last().expect("sempre há Eof");
    assert_eq!(eof.tipo, TipoToken::Eof);
    assert_eq!(eof.lexema, "");
    assert_eq!((eof.linha, eof.coluna), (1, 2));
}

#[test]
fn erros_vazio() {
    let mut lexer = Lexer::new("var x: int = 1;");
    lexer.tokens();
    assert!(lexer.erros().is_empty());
}

#[test]
fn utf8_coluna_em_caracteres() {
    let toks = lexar("// ção\nvar ç x");
    let x = &toks[1];
    assert_eq!(x.tipo, TipoToken::Id);
    assert_eq!(x.lexema, "x");
    assert_eq!((x.linha, x.coluna), (2, 7));
}

#[test]
fn segunda_chamada_nao_entra_em_panico() {
    let mut lexer = Lexer::new("x");
    assert_eq!(lexer.tokens().len(), 2);
    assert_eq!(lexer.tokens().len(), 1);
}

#[test]
fn fatorial() {
    let toks = lexar(include_str!("../../exemplos/fatorial.mk"));
    assert_eq!(toks.len(), 40);
    assert_eq!(toks.last().map(|t| t.tipo), Some(TipoToken::Eof));
    assert!(toks.iter().all(|t| t.linha >= 1 && t.coluna >= 1));
    assert_eq!(toks[0].tipo, TipoToken::Var);
    assert_eq!((toks[0].linha, toks[0].coluna), (2, 1));
}
