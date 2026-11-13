//! Analisador sintático descendente recursivo (`Parser`, `ErroSintatico`). Preenchido em M11 e M12.
//!
//! Cada não-terminal da gramática (`docs/gramatica.md`) vira um método. Operadores
//! associativos à esquerda usam laço em vez de recursão à esquerda.

use std::fmt;

use crate::ast::{Expr, OpBin, OpUn};
use crate::token::{TipoToken, Token};

/// Erro sintático já formatado como `{linha}:{coluna}: erro sintático: {msg}, encontrado '{lexema}'`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ErroSintatico {
    pub mensagem: String,
}

impl fmt::Display for ErroSintatico {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.mensagem)
    }
}

impl std::error::Error for ErroSintatico {}

/// Parser sobre a lista de tokens produzida pelo lexer.
pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    /// Cria o parser. Garante que a lista termine em `Eof`, mesmo se vier vazia.
    pub fn new(mut tokens: Vec<Token>) -> Self {
        if tokens.last().map(|t| t.tipo) != Some(TipoToken::Eof) {
            let (linha, coluna) = tokens
                .last()
                .map(|t| (t.linha, t.coluna + t.lexema.chars().count()))
                .unwrap_or((1, 1));
            tokens.push(Token::new(TipoToken::Eof, "", linha, coluna));
        }
        Parser { tokens, pos: 0 }
    }

    // ---------------------------------------------------------------- utilitários

    /// Token atual, sem consumir.
    fn atual(&self) -> &Token {
        &self.tokens[self.pos]
    }

    /// O token atual é de algum dos tipos dados?
    fn verificar(&self, tipos: &[TipoToken]) -> bool {
        tipos.contains(&self.atual().tipo)
    }

    /// Consome e devolve o token atual. Nunca avança além do `Eof`.
    fn consumir(&mut self) -> Token {
        let token = self.tokens[self.pos].clone();
        if token.tipo != TipoToken::Eof {
            self.pos += 1;
        }
        token
    }

    /// Consome um token do tipo esperado ou devolve erro com `msg`.
    fn esperar(&mut self, tipo: TipoToken, msg: &str) -> Result<Token, ErroSintatico> {
        if self.verificar(&[tipo]) {
            Ok(self.consumir())
        } else {
            Err(self.erro(msg))
        }
    }

    /// Erro sintático na posição do token atual.
    fn erro(&self, msg: &str) -> ErroSintatico {
        let t = self.atual();
        let lexema = if t.lexema.is_empty() {
            "EOF"
        } else {
            t.lexema.as_str()
        };
        ErroSintatico {
            mensagem: format!(
                "{}:{}: erro sintático: {}, encontrado '{}'",
                t.linha, t.coluna, msg, lexema
            ),
        }
    }

    // ---------------------------------------------------------------- expressões

    /// Ponto de entrada temporário para testar expressões isoladamente (M11).
    pub fn expressao(&mut self) -> Result<Expr, ErroSintatico> {
        self.expr()
    }

    /// `expr = soma [ op_comp soma ]` — comparação não associativa (`if`, não laço).
    fn expr(&mut self) -> Result<Expr, ErroSintatico> {
        let esq = self.soma()?;
        if let Some(op) = op_comparacao(self.atual().tipo) {
            let linha = self.consumir().linha;
            let dir = self.soma()?;
            return Ok(Expr::binario(op, esq, dir, linha));
        }
        Ok(esq)
    }

    /// `soma = termo { ( "+" | "-" ) termo }`
    fn soma(&mut self) -> Result<Expr, ErroSintatico> {
        let mut esq = self.termo()?;
        while self.verificar(&[TipoToken::Mais, TipoToken::Menos]) {
            let token = self.consumir();
            let op = if token.tipo == TipoToken::Mais {
                OpBin::Mais
            } else {
                OpBin::Menos
            };
            let dir = self.termo()?;
            esq = Expr::binario(op, esq, dir, token.linha);
        }
        Ok(esq)
    }

    /// `termo = unario { ( "*" | "/" ) unario }`
    fn termo(&mut self) -> Result<Expr, ErroSintatico> {
        let mut esq = self.unario()?;
        while self.verificar(&[TipoToken::Vezes, TipoToken::Div]) {
            let token = self.consumir();
            let op = if token.tipo == TipoToken::Vezes {
                OpBin::Vezes
            } else {
                OpBin::Div
            };
            let dir = self.unario()?;
            esq = Expr::binario(op, esq, dir, token.linha);
        }
        Ok(esq)
    }

    /// `unario = "-" unario | fator`
    fn unario(&mut self) -> Result<Expr, ErroSintatico> {
        if self.verificar(&[TipoToken::Menos]) {
            let linha = self.consumir().linha;
            let expr = self.unario()?;
            return Ok(Expr::unario(OpUn::Neg, expr, linha));
        }
        self.fator()
    }

    /// `fator = NUM | "true" | "false" | ID | "(" expr ")"`
    fn fator(&mut self) -> Result<Expr, ErroSintatico> {
        let linha = self.atual().linha;
        match self.atual().tipo {
            TipoToken::Num => {
                let valor = self
                    .atual()
                    .lexema
                    .parse::<i64>()
                    .map_err(|_| self.erro("literal inteiro fora do intervalo"))?;
                self.consumir();
                Ok(Expr::num(valor, linha))
            }
            TipoToken::True => {
                self.consumir();
                Ok(Expr::bool(true, linha))
            }
            TipoToken::False => {
                self.consumir();
                Ok(Expr::bool(false, linha))
            }
            TipoToken::Id => {
                let token = self.consumir();
                Ok(Expr::var(token.lexema, linha))
            }
            TipoToken::AbrePar => {
                self.consumir();
                let expr = self.expr()?;
                self.esperar(TipoToken::FechaPar, "esperado ')'")?;
                Ok(expr)
            }
            _ => Err(self.erro("esperado expressão")),
        }
    }
}

/// Operador de comparação correspondente ao token, se houver.
fn op_comparacao(tipo: TipoToken) -> Option<OpBin> {
    match tipo {
        TipoToken::Igual => Some(OpBin::Igual),
        TipoToken::Diferente => Some(OpBin::Diferente),
        TipoToken::Menor => Some(OpBin::Menor),
        TipoToken::Maior => Some(OpBin::Maior),
        TipoToken::MenorIgual => Some(OpBin::MenorIgual),
        TipoToken::MaiorIgual => Some(OpBin::MaiorIgual),
        _ => None,
    }
}

#[cfg(test)]
mod testes;
