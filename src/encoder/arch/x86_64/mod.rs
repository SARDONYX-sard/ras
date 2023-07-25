pub mod bin_const;
pub mod instructions;
pub mod registers;

use self::registers::Register;
use crate::lexer::TokenKind;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Expr {
    Ident(String),
    Number(String),
    /// unary minus
    Neg(Box<Expr>),
    Binop {
        left_hs: Box<Expr>,
        right_hs: Box<Expr>,
        op: TokenKind,
    },
    /// e.g. movq $1, rax
    Immediate(Box<Expr>),
    /// displacement(base + index << scale)
    Indirection {
        disp: Option<Box<Expr>>,
        /// Expected Register
        base: Option<Box<Expr>>,
        /// Expected Register
        index: Option<Box<Expr>>,
        /// Expected
        scale: Option<Box<Expr>>,
        has_base: bool,
        has_index_scale: bool,
    },
    /// General purpose registers
    Register(Register),
    /// Single instruction, multiple data registers(https://en.wikipedia.org/wiki/Single_instruction,_multiple_data)
    Xmm(Register),
    /// Expected Register
    Star(Box<Expr>),
}
