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
mod testes;
