//! monk: compilador da linguagem MiniLang (léxico, sintático, semântico, TAC, VM e otimizador).

pub mod ast;
pub mod ir;
pub mod lexer;
pub mod otimizador;
pub mod parser;
pub mod semantica;
pub mod simbolos;
pub mod token;
pub mod vm;
