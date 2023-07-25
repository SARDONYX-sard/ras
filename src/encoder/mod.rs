// mod addr;
// mod stack_op;
pub mod arch;

use self::arch::x86_64::{
    bin_const::OPERAND_SIZE_PREFIX16,
    instructions::{self, InstrKind},
    registers::{get_reg_info_by, get_xmm_by, DataSizeSuffix, Register},
    Expr,
};
use crate::error::{self, bail, Result};
use crate::lexer::{Location, Token, TokenKind};
use std::collections::HashMap;

macro_rules! bail {

    ($loc:expr, $($tt:tt)*) => {{
        let err = $crate::error::format_err!($($tt)*)
            .with_location($loc);
        return Err(err);
    }};
}

/// Instruction information
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Instr {
    pub(crate) kind: InstrKind,
    pub(crate) code: Vec<u8>,
    pub(crate) symbol_name: String,
    pub(crate) flags: String,
    pub(crate) addr: usize,
    pub(crate) binding: u8,
    /// STV_DEFAULT, STV_INTERNAL, STV_HIDDEN, STV_PROTECTED
    pub(crate) visibility: u8,
    pub(crate) symbol_type: u8,
    pub(crate) section: String,
    pub(crate) is_jmp_or_call: bool,
    pub(crate) loc: Location,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Rela {
    pub uses: String,
    pub instr: Instr,
    pub offset: usize,
    pub rtype: u64,
    pub adjust: i32,
    pub is_already_resolved: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct UserDefinedSection {
    pub code: Vec<u8>,
    pub addr: usize,
    pub flags: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Encoder {
    tokens: Vec<Token>,
    token_idx: usize,
    current_section_name: String,
    current_instr: Instr,
    /// All instructions, sections, symbols, directives
    instrs: Vec<Instr>,
    user_defined_symbols: HashMap<String, Instr>,
    user_defined_sections: HashMap<String, UserDefinedSection>,
}

impl Default for Encoder {
    fn default() -> Self {
        Self {
            tokens: Default::default(),
            token_idx: Default::default(),
            current_instr: Default::default(),
            current_section_name: ".text".to_owned(),
            instrs: Vec::with_capacity(1500000),
            user_defined_symbols: Default::default(),
            user_defined_sections: Default::default(),
        }
    }
}

/// Next token is matched by 1st arg?
///
/// # Parameters:
/// - `token_kind`: expected tokenKind
/// - `index`: current index for tokens. This index is incremented.
/// - `tokens`: list of tokens
fn expect(token_kind: TokenKind, index: &mut usize, tokens: &[Token]) -> Result<()> {
    let Token { kind, loc } = peek_next(index, tokens)?;
    match token_kind == *kind {
        true => Ok(()),
        false => bail!(*loc, "Unexpected token {kind:?}. expected {token_kind:?}",),
    }
}

fn peek_n(n: usize, tokens: &[Token]) -> Result<&Token> {
    match tokens.get(n) {
        Some(token) => Ok(token),
        None => bail!(
            tokens[n].loc,
            "The '{n}'th Token in the Token vector was not found."
        ),
    }
}

fn peek_next<'a>(index: &mut usize, tokens: &'a [Token]) -> Result<&'a Token> {
    *index += 1;
    peek_n(*index, tokens)
}

/// First, index+=1.
/// 2nd, Parse register from global data. return XMM or general register.
fn parse_register(index: &mut usize, tokens: &[Token]) -> Result<Expr> {
    let err_msg = "The next character after `%` must be register.";
    let current_loc = peek_n(*index, tokens)?.loc;

    *index += 1;
    let next_token = match peek_n(*index, tokens) {
        Ok(next) => next,
        Err(_) => bail!(current_loc, "{err_msg}"),
    };

    match &next_token.kind {
        TokenKind::Ident(reg_name) => Ok(match get_xmm_by(&reg_name.to_uppercase()) {
            Ok(xmm) => Expr::Xmm(xmm),
            Err(_err) => Expr::Register(get_reg_info_by(&reg_name.to_uppercase())?),
        }),
        _ => bail!(current_loc, "{err_msg}"),
    }
}

/// Parse Number | Identifier | Unary minus
fn parse_factor(index: &mut usize, tokens: &[Token]) -> Result<Expr> {
    let current_token = peek_n(*index, tokens)?;
    Ok(match &current_token.kind {
        TokenKind::Number(num) => Expr::Number(num.to_string()),
        TokenKind::Ident(ident) => Expr::Ident(ident.to_string()),
        TokenKind::Minus => {
            *index += 1;
            Expr::Neg(Box::new(parse_factor(index, tokens)?))
        }
        _ => bail!(
            current_token.loc,
            "Unexpected token kind: {:?}",
            current_token.kind
        ),
    })
}

/// Parse binary expression
fn parse_expr(index: &mut usize, tokens: &[Token]) -> Result<Expr> {
    let left_hs = Box::new(parse_factor(index, tokens)?);

    let current_token = peek_n(*index, tokens)?;
    Ok(match &current_token.kind {
        TokenKind::Div | TokenKind::Minus | TokenKind::Mul | TokenKind::Plus => {
            let op = current_token.kind.clone();
            *index += 1;
            let right_hs = Box::new(parse_expr(index, tokens)?);
            Expr::Binop {
                left_hs,
                right_hs,
                op,
            }
        }
        _ => bail!(
            current_token.loc,
            "Unexpected token kind: {:?}",
            current_token.kind
        ),
    })
}

/// Parse e.g. (movq `rsi, rdi` )
fn parse_two_operand(index: &mut usize, tokens: &[Token]) -> Result<(Expr, Expr)> {
    let src = parse_operand(index, tokens)?;
    expect(TokenKind::Comma, index, tokens)?;
    let dst = parse_operand(index, tokens)?;
    Ok((src, dst))
}

fn parse_indirect(index: &mut usize, tokens: &[Token]) -> Result<Expr> {
    let kind = &peek_n(*index, tokens)?.kind;

    // - indirect expression
    //   displacement(base + index, scale)
    // e.g.         8(rbx + rdi, 8)
    let expr = match *kind == TokenKind::LParen {
        // Starting with '(' means displacement is omitted.
        true => Expr::Number("0".to_owned()),
        false => parse_expr(index, tokens)?,
    };
    if *kind != TokenKind::LParen {
        return Ok(expr);
    };

    Ok(match peek_next(index, tokens)?.kind == TokenKind::Comma {
        true => {
            let indirect = Expr::Indirection {
                disp: Some(Box::new(expr)),
                base: Some(Box::new(parse_register(index, tokens)?)),
                index: Some(Box::new(parse_register(index, tokens)?)),
                scale: Some(Box::new(
                    match peek_next(index, tokens)?.kind == TokenKind::Comma {
                        true => parse_expr(index, tokens)?,
                        false => Expr::Number("1".to_owned()),
                    },
                )),
                has_base: false,
                has_index_scale: false,
            };
            expect(TokenKind::RParen, index, tokens)?;
            indirect
        }
        false => Expr::Indirection {
            disp: Some(Box::new(expr)),
            base: None,
            index: None,
            scale: None,
            has_base: false,
            has_index_scale: false,
        },
    })
}

fn parse_operand(index: &mut usize, tokens: &[Token]) -> Result<Expr> {
    let Token { kind, loc } = peek_n(*index, tokens)?;

    Ok(match &kind {
        // Dolor is immediate prefix. e.g. $1
        TokenKind::Dolor => {
            *index += 1;
            Expr::Immediate(Box::new(parse_expr(index, tokens)?))
        }
        TokenKind::Percent => parse_register(index, tokens)?,
        TokenKind::Mul => Expr::Star(Box::new(parse_register(index, tokens)?)),
        TokenKind::LParen => parse_indirect(index, tokens)?,
        _ => {
            bail!(*loc, "Unexpected token kind: {kind:?}")
        }
    })
}

fn eval_expr_get_symbol_64(expr: Expr, arr: &mut Vec<String>) -> Result<i64> {
    Ok(match expr {
        Expr::Number(string) => match string.parse::<i64>() {
            Ok(int) => int,
            Err(_) => error::bail!("Failed to parse number"),
        },
        Expr::Binop {
            left_hs,
            right_hs,
            op,
        } => match op {
            TokenKind::Plus => {
                eval_expr_get_symbol_64(*left_hs, arr)? + eval_expr_get_symbol_64(*right_hs, arr)?
            }
            TokenKind::Minus => {
                eval_expr_get_symbol_64(*left_hs, arr)? - eval_expr_get_symbol_64(*right_hs, arr)?
            }
            TokenKind::Mul => {
                eval_expr_get_symbol_64(*left_hs, arr)? * eval_expr_get_symbol_64(*right_hs, arr)?
            }
            TokenKind::Div => {
                eval_expr_get_symbol_64(*left_hs, arr)? / eval_expr_get_symbol_64(*right_hs, arr)?
            }
            _ => error::bail!("Unimplemented yet!"),
        },
        Expr::Ident(ident) => {
            arr.push(ident);
            0
        }
        Expr::Neg(num_stmt) => -eval_expr_get_symbol_64(*num_stmt, arr)?,
        Expr::Immediate(stmt) => eval_expr_get_symbol_64(*stmt, arr)?,
        _ => unimplemented!(),
    })
}

fn eval_expr(expr: Expr) -> Result<i32> {
    let mut arr = Vec::new();
    Ok(eval_expr_get_symbol_64(expr, &mut arr)? as i32)
}

/// The 4-bit regions are called REX.w, REX.r, REX.x, and REX.b, in order from bit 3 to 0.
///
/// |7|6|5|4|  3  |  2  |  1  |  0  |
/// |-|-|-|-|-----|-----|-----|-----|
/// |0|1|0|0|REX.w|REX.r|REX.x|REX.b|
/// - REX.w - 1 = operand size to 64 bits
/// - REX.r - functions as bit 4 in the reg field of ModR/M.
/// - REX.x - Functions as bit 4 in the index field of the SIB byte.
/// - REX.b - functions as bit 4 in the r/m field of ModR/M and the base field of SIB byte.
///
/// See:
/// - eng: https://wiki.osdev.org/X86-64_Instruction_Encoding#REX_prefix
/// - jp: https://www.wdic.org/w/SCI/REX%E3%83%97%E3%83%AA%E3%83%95%E3%82%A3%E3%83%83%E3%82%AF%E3%82%B9
fn rex(w: u8, r: u8, x: u8, b: u8) -> u8 {
    64 | (w << 3) | (r << 2) | (x << 1) | b
}

impl Encoder {
    fn add_prefix(
        &mut self,
        reg_r: Register,
        reg_i: Register,
        reg_b: Register,
        sizes: &[DataSizeSuffix],
    ) {
        let w = match sizes.contains(&DataSizeSuffix::Quad) {
            true => 1,
            false => 0,
        };
        // need rex prefix R8~ registers.
        let r = match reg_r.base_offset >= 8 {
            true => 1,
            false => 0,
        };
        let x = match reg_i.base_offset >= 8 {
            true => 1,
            false => 0,
        };

        let b = match reg_b.base_offset >= 8 {
            true => 1,
            false => 0,
        };

        if sizes.contains(&DataSizeSuffix::Word) {
            self.current_instr.code.push(OPERAND_SIZE_PREFIX16);
        };
        if sizes.contains(&DataSizeSuffix::Single) {
            self.current_instr.code.push(0xf3);
        };
        if sizes.contains(&DataSizeSuffix::Double) {
            self.current_instr.code.push(0xf2);
        }

        if w != 0 || r != 0 || b != 0 || x != 0 || reg_r.rex_required || reg_b.rex_required {
            self.current_instr.code.push(rex(w, r, x, b));
        }
    }
}

fn align_to(n: i32, align: i32) -> i32 {
    (n + align - 1) / align * align
}

fn compose_mod_rm(r#mod: u8, reg_op: u8, rm: u8) -> u8 {
    (r#mod << 6) + (reg_op << 3) + rm
}

pub(crate) fn parse(tokens: Vec<Token>) -> Result<()> {
    let mut index = 0;
    dbg!(index);
    while index <= tokens.len() {
        dbg!(parse_operand(&mut index, &tokens)?);
        index += 1;
    }

    Ok(())
}
