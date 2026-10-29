//! Analisador léxico (`Lexer`) e tratamento de erros léxicos. Preenchido em M04 e M06.

use crate::token::{palavra_reservada, TipoToken, Token};

/// Analisador léxico escrito à mão, caractere a caractere.
pub struct Lexer {
    fonte: Vec<char>,
    pos: usize,
    linha: usize,
    coluna: usize,
    erros: Vec<String>,
}

impl Lexer {
    /// Cria um lexer sobre o texto-fonte; linha e coluna começam em 1.
    pub fn new(fonte: &str) -> Self {
        Lexer {
            fonte: fonte.chars().collect(),
            pos: 0,
            linha: 1,
            coluna: 1,
            erros: Vec::new(),
        }
    }

    /// Consome a entrada e devolve os tokens, sempre terminando em `Eof`.
    pub fn tokens(&mut self) -> Vec<Token> {
        let mut saida = Vec::new();
        loop {
            self.pular_espacos_e_comentarios();
            let Some(c) = self.atual() else { break };
            let (linha, coluna) = (self.linha, self.coluna);

            if c.is_ascii_digit() {
                let lexema = self.ler_enquanto(|c| c.is_ascii_digit());
                saida.push(Token::new(TipoToken::Num, lexema, linha, coluna));
            } else if c.is_ascii_alphabetic() || c == '_' {
                let lexema = self.ler_enquanto(|c| c.is_ascii_alphanumeric() || c == '_');
                let tipo = palavra_reservada(&lexema).unwrap_or(TipoToken::Id);
                saida.push(Token::new(tipo, lexema, linha, coluna));
            } else if let Some((tipo, lexema)) = self.ler_operador(c) {
                saida.push(Token::new(tipo, lexema, linha, coluna));
            } else {
                // M06: registrar erro léxico aqui.
                self.avancar();
            }
        }
        saida.push(Token::new(TipoToken::Eof, "", self.linha, self.coluna));
        saida
    }

    /// Erros léxicos encontrados (sempre vazio até a M06).
    pub fn erros(&self) -> &[String] {
        &self.erros
    }

    fn atual(&self) -> Option<char> {
        self.fonte.get(self.pos).copied()
    }

    fn proximo(&self) -> Option<char> {
        self.fonte.get(self.pos + 1).copied()
    }

    fn avancar(&mut self) {
        if let Some(c) = self.atual() {
            self.pos += 1;
            if c == '\n' {
                self.linha += 1;
                self.coluna = 1;
            } else {
                self.coluna += 1;
            }
        }
    }

    fn pular_espacos_e_comentarios(&mut self) {
        while let Some(c) = self.atual() {
            if matches!(c, ' ' | '\t' | '\r' | '\n') {
                self.avancar();
            } else if c == '/' && self.proximo() == Some('/') {
                while self.atual().is_some_and(|c| c != '\n') {
                    self.avancar();
                }
            } else {
                break;
            }
        }
    }

    fn ler_enquanto(&mut self, pred: impl Fn(char) -> bool) -> String {
        let mut lexema = String::new();
        while let Some(c) = self.atual() {
            if !pred(c) {
                break;
            }
            lexema.push(c);
            self.avancar();
        }
        lexema
    }

    /// Reconhece um operador ou delimitador começando em `c` (2 caracteres antes de 1).
    fn ler_operador(&mut self, c: char) -> Option<(TipoToken, String)> {
        let dois = match (c, self.proximo()) {
            ('=', Some('=')) => Some(TipoToken::Igual),
            ('!', Some('=')) => Some(TipoToken::Diferente),
            ('<', Some('=')) => Some(TipoToken::MenorIgual),
            ('>', Some('=')) => Some(TipoToken::MaiorIgual),
            _ => None,
        };
        if let Some(tipo) = dois {
            self.avancar();
            self.avancar();
            return Some((tipo, format!("{c}=")));
        }
        let tipo = match c {
            '+' => TipoToken::Mais,
            '-' => TipoToken::Menos,
            '*' => TipoToken::Vezes,
            '/' => TipoToken::Div,
            '=' => TipoToken::Atrib,
            '<' => TipoToken::Menor,
            '>' => TipoToken::Maior,
            '(' => TipoToken::AbrePar,
            ')' => TipoToken::FechaPar,
            '{' => TipoToken::AbreChave,
            '}' => TipoToken::FechaChave,
            ':' => TipoToken::DoisPontos,
            ';' => TipoToken::PontoVirg,
            _ => return None,
        };
        self.avancar();
        Some((tipo, c.to_string()))
    }
}

#[cfg(test)]
mod testes {
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
        let toks = lexar(include_str!("../exemplos/fatorial.mk"));
        assert_eq!(toks.len(), 40);
        assert_eq!(toks.last().map(|t| t.tipo), Some(TipoToken::Eof));
        assert!(toks.iter().all(|t| t.linha >= 1 && t.coluna >= 1));
        assert_eq!(toks[0].tipo, TipoToken::Var);
        assert_eq!((toks[0].linha, toks[0].coluna), (2, 1));
    }
}
