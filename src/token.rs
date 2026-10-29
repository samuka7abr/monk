//! Tipos de token da MiniLang (`TipoToken`, `Token`, `palavra_reservada`).

use std::fmt;

/// Categoria léxica de um token, conforme a tabela 2.7 da especificação.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TipoToken {
    Num,
    Id,
    Var,
    Int,
    Bool,
    True,
    False,
    If,
    Else,
    While,
    Print,
    Mais,
    Menos,
    Vezes,
    Div,
    Atrib,
    Igual,
    Diferente,
    Menor,
    Maior,
    MenorIgual,
    MaiorIgual,
    AbrePar,
    FechaPar,
    AbreChave,
    FechaChave,
    DoisPontos,
    PontoVirg,
    Eof,
}

impl fmt::Display for TipoToken {
    /// Imprime o nome em SCREAMING_SNAKE (formato usado por `--tokens`).
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let nome: &'static str = match self {
            TipoToken::Num => "NUM",
            TipoToken::Id => "ID",
            TipoToken::Var => "VAR",
            TipoToken::Int => "INT",
            TipoToken::Bool => "BOOL",
            TipoToken::True => "TRUE",
            TipoToken::False => "FALSE",
            TipoToken::If => "IF",
            TipoToken::Else => "ELSE",
            TipoToken::While => "WHILE",
            TipoToken::Print => "PRINT",
            TipoToken::Mais => "MAIS",
            TipoToken::Menos => "MENOS",
            TipoToken::Vezes => "VEZES",
            TipoToken::Div => "DIV",
            TipoToken::Atrib => "ATRIB",
            TipoToken::Igual => "IGUAL",
            TipoToken::Diferente => "DIFERENTE",
            TipoToken::Menor => "MENOR",
            TipoToken::Maior => "MAIOR",
            TipoToken::MenorIgual => "MENOR_IGUAL",
            TipoToken::MaiorIgual => "MAIOR_IGUAL",
            TipoToken::AbrePar => "ABRE_PAR",
            TipoToken::FechaPar => "FECHA_PAR",
            TipoToken::AbreChave => "ABRE_CHAVE",
            TipoToken::FechaChave => "FECHA_CHAVE",
            TipoToken::DoisPontos => "DOIS_PONTOS",
            TipoToken::PontoVirg => "PONTO_VIRG",
            TipoToken::Eof => "EOF",
        };
        f.pad(nome)
    }
}

/// Token com o lexema original e a posição (linha e coluna a partir de 1).
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub tipo: TipoToken,
    pub lexema: String,
    pub linha: usize,
    pub coluna: usize,
}

impl Token {
    /// Cria um token a partir de suas partes.
    pub fn new(tipo: TipoToken, lexema: impl Into<String>, linha: usize, coluna: usize) -> Self {
        Token {
            tipo,
            lexema: lexema.into(),
            linha,
            coluna,
        }
    }
}

/// Retorna o tipo da palavra reservada, ou `None` se o lexema for um identificador.
pub fn palavra_reservada(lexema: &str) -> Option<TipoToken> {
    match lexema {
        "var" => Some(TipoToken::Var),
        "int" => Some(TipoToken::Int),
        "bool" => Some(TipoToken::Bool),
        "true" => Some(TipoToken::True),
        "false" => Some(TipoToken::False),
        "if" => Some(TipoToken::If),
        "else" => Some(TipoToken::Else),
        "while" => Some(TipoToken::While),
        "print" => Some(TipoToken::Print),
        _ => None,
    }
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn reservada_while() {
        assert_eq!(palavra_reservada("while"), Some(TipoToken::While));
    }

    #[test]
    fn prefixo_nao_e_reservada() {
        assert_eq!(palavra_reservada("whilex"), None);
        assert_eq!(palavra_reservada("wh"), None);
        assert_eq!(palavra_reservada(""), None);
    }

    #[test]
    fn todas_as_reservadas() {
        let esperado = [
            ("var", TipoToken::Var),
            ("int", TipoToken::Int),
            ("bool", TipoToken::Bool),
            ("true", TipoToken::True),
            ("false", TipoToken::False),
            ("if", TipoToken::If),
            ("else", TipoToken::Else),
            ("while", TipoToken::While),
            ("print", TipoToken::Print),
        ];
        for (lexema, tipo) in esperado {
            assert_eq!(palavra_reservada(lexema), Some(tipo), "lexema {lexema}");
        }
    }

    #[test]
    fn reservadas_diferenciam_maiusculas() {
        assert_eq!(palavra_reservada("While"), None);
        assert_eq!(palavra_reservada("TRUE"), None);
        assert_eq!(palavra_reservada("Print"), None);
    }

    #[test]
    fn display_de_todos_os_tipos() {
        let tabela = [
            (TipoToken::Num, "NUM"),
            (TipoToken::Id, "ID"),
            (TipoToken::Var, "VAR"),
            (TipoToken::Int, "INT"),
            (TipoToken::Bool, "BOOL"),
            (TipoToken::True, "TRUE"),
            (TipoToken::False, "FALSE"),
            (TipoToken::If, "IF"),
            (TipoToken::Else, "ELSE"),
            (TipoToken::While, "WHILE"),
            (TipoToken::Print, "PRINT"),
            (TipoToken::Mais, "MAIS"),
            (TipoToken::Menos, "MENOS"),
            (TipoToken::Vezes, "VEZES"),
            (TipoToken::Div, "DIV"),
            (TipoToken::Atrib, "ATRIB"),
            (TipoToken::Igual, "IGUAL"),
            (TipoToken::Diferente, "DIFERENTE"),
            (TipoToken::Menor, "MENOR"),
            (TipoToken::Maior, "MAIOR"),
            (TipoToken::MenorIgual, "MENOR_IGUAL"),
            (TipoToken::MaiorIgual, "MAIOR_IGUAL"),
            (TipoToken::AbrePar, "ABRE_PAR"),
            (TipoToken::FechaPar, "FECHA_PAR"),
            (TipoToken::AbreChave, "ABRE_CHAVE"),
            (TipoToken::FechaChave, "FECHA_CHAVE"),
            (TipoToken::DoisPontos, "DOIS_PONTOS"),
            (TipoToken::PontoVirg, "PONTO_VIRG"),
            (TipoToken::Eof, "EOF"),
        ];
        assert_eq!(tabela.len(), 29);
        for (tipo, texto) in tabela {
            assert_eq!(tipo.to_string(), texto);
        }
    }

    #[test]
    fn display_explicito() {
        assert_eq!(TipoToken::DoisPontos.to_string(), "DOIS_PONTOS");
        assert_eq!(TipoToken::MenorIgual.to_string(), "MENOR_IGUAL");
        assert_eq!(TipoToken::Eof.to_string(), "EOF");
    }

    #[test]
    fn display_respeita_largura() {
        assert_eq!(format!("{:<6}|", TipoToken::Id), "ID    |");
    }

    #[test]
    fn construtor_de_token() {
        let t = Token::new(TipoToken::Id, "x", 2, 5);
        assert_eq!(t.tipo, TipoToken::Id);
        assert_eq!(t.lexema, "x");
        assert_eq!((t.linha, t.coluna), (2, 5));
    }
}
