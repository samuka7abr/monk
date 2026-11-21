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

#[test]
fn erro_um_caractere() {
    let mut lexer = Lexer::new("x @ y");
    let tipos_lidos: Vec<TipoToken> = lexer.tokens().into_iter().map(|t| t.tipo).collect();
    assert_eq!(
        tipos_lidos,
        vec![TipoToken::Id, TipoToken::Id, TipoToken::Eof]
    );
    assert_eq!(lexer.erros(), ["1:3: erro léxico: caractere inválido '@'"]);
}

#[test]
fn erros_em_linhas_diferentes() {
    let mut lexer = Lexer::new("a @ b\n\n  c $ d");
    let toks = lexer.tokens();
    assert_eq!(toks.len(), 5);
    assert_eq!(
        lexer.erros(),
        [
            "1:3: erro léxico: caractere inválido '@'",
            "3:5: erro léxico: caractere inválido '$'",
        ]
    );
}

#[test]
fn erro_no_fim_do_arquivo() {
    let mut lexer = Lexer::new("x #");
    let toks = lexer.tokens();
    let eof = toks.last().expect("sempre há Eof");
    assert_eq!(eof.tipo, TipoToken::Eof);
    assert_eq!((eof.linha, eof.coluna), (1, 4));
    assert_eq!(lexer.erros(), ["1:3: erro léxico: caractere inválido '#'"]);
}

#[test]
fn erros_consecutivos() {
    let mut lexer = Lexer::new("x@#y");
    let tipos_lidos: Vec<TipoToken> = lexer.tokens().into_iter().map(|t| t.tipo).collect();
    assert_eq!(
        tipos_lidos,
        vec![TipoToken::Id, TipoToken::Id, TipoToken::Eof]
    );
    assert_eq!(
        lexer.erros(),
        [
            "1:2: erro léxico: caractere inválido '@'",
            "1:3: erro léxico: caractere inválido '#'",
        ]
    );
}

/// Token observado nos testes: `(tipo, lexema, linha, coluna)`.
type TokenVisto = (TipoToken, String, usize, usize);

/// Tokens e erros léxicos da entrada.
fn lexar_detalhado(fonte: &str) -> (Vec<TokenVisto>, Vec<String>) {
    let mut lexer = Lexer::new(fonte);
    let toks = lexer
        .tokens()
        .into_iter()
        .map(|t| (t.tipo, t.lexema, t.linha, t.coluna))
        .collect();
    (toks, lexer.erros().to_vec())
}

fn tok(tipo: TipoToken, lexema: &str, linha: usize, coluna: usize) -> TokenVisto {
    (tipo, lexema.to_string(), linha, coluna)
}

#[test]
fn retorno_de_carro_e_espaco() {
    use TipoToken::*;
    let (toks, erros) = lexar_detalhado("var\r\nx\r\n");
    assert_eq!(
        toks,
        vec![
            tok(Var, "var", 1, 1),
            tok(Id, "x", 2, 1),
            tok(Eof, "", 3, 1)
        ]
    );
    assert!(erros.is_empty(), "{erros:?}");

    // `\r` sozinho separa tokens, conta uma coluna e não inicia nova linha.
    let (toks, erros) = lexar_detalhado("a\rb");
    assert_eq!(
        toks,
        vec![tok(Id, "a", 1, 1), tok(Id, "b", 1, 3), tok(Eof, "", 1, 4)]
    );
    assert!(erros.is_empty(), "{erros:?}");
}

#[test]
fn exclamacao_isolada_e_erro() {
    use TipoToken::*;
    let (toks, erros) = lexar_detalhado("a ! b");
    assert_eq!(
        toks,
        vec![tok(Id, "a", 1, 1), tok(Id, "b", 1, 5), tok(Eof, "", 1, 6)]
    );
    assert_eq!(erros, ["1:3: erro léxico: caractere inválido '!'"]);

    // `! =` separados não formam `!=`.
    let (toks, erros) = lexar_detalhado("! =");
    assert_eq!(toks, vec![tok(Atrib, "=", 1, 3), tok(Eof, "", 1, 4)]);
    assert_eq!(erros, ["1:1: erro léxico: caractere inválido '!'"]);

    // `!` no fim da entrada.
    let (toks, erros) = lexar_detalhado("!");
    assert_eq!(toks, vec![tok(Eof, "", 1, 2)]);
    assert_eq!(erros, ["1:1: erro léxico: caractere inválido '!'"]);

    // Controle: `!=` colado continua um único token.
    let (toks, erros) = lexar_detalhado("a!=b");
    assert_eq!(
        toks,
        vec![
            tok(Id, "a", 1, 1),
            tok(Diferente, "!=", 1, 2),
            tok(Id, "b", 1, 4),
            tok(Eof, "", 1, 5),
        ]
    );
    assert!(erros.is_empty(), "{erros:?}");
}

#[test]
fn letra_unicode_nao_inicia_identificador() {
    use TipoToken::*;
    let (toks, erros) = lexar_detalhado("çx");
    assert_eq!(toks, vec![tok(Id, "x", 1, 2), tok(Eof, "", 1, 3)]);
    assert_eq!(erros, ["1:1: erro léxico: caractere inválido 'ç'"]);
}

#[test]
fn letra_unicode_nao_continua_identificador() {
    use TipoToken::*;
    let (toks, erros) = lexar_detalhado("aç1");
    assert_eq!(
        toks,
        vec![tok(Id, "a", 1, 1), tok(Num, "1", 1, 3), tok(Eof, "", 1, 4)]
    );
    assert_eq!(erros, ["1:2: erro léxico: caractere inválido 'ç'"]);
}

#[test]
fn digito_unicode_nao_continua_identificador() {
    use TipoToken::*;
    // U+0663 é o dígito três arábico-índico.
    let (toks, erros) = lexar_detalhado("a\u{0663}b");
    assert_eq!(
        toks,
        vec![tok(Id, "a", 1, 1), tok(Id, "b", 1, 3), tok(Eof, "", 1, 4)]
    );
    assert_eq!(erros, ["1:2: erro léxico: caractere inválido '\u{0663}'"]);
}

#[test]
fn digito_unicode_nao_inicia_numero() {
    use TipoToken::*;
    let (toks, erros) = lexar_detalhado("\u{0663}7");
    assert_eq!(toks, vec![tok(Num, "7", 1, 2), tok(Eof, "", 1, 3)]);
    assert_eq!(erros, ["1:1: erro léxico: caractere inválido '\u{0663}'"]);
}

#[test]
fn digito_unicode_nao_continua_numero() {
    use TipoToken::*;
    let (toks, erros) = lexar_detalhado("1\u{0663}2");
    assert_eq!(
        toks,
        vec![tok(Num, "1", 1, 1), tok(Num, "2", 1, 3), tok(Eof, "", 1, 4)]
    );
    assert_eq!(erros, ["1:2: erro léxico: caractere inválido '\u{0663}'"]);
}
