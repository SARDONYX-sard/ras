mod addr;
mod stack_op;

use std::{collections::HashMap, process::exit};

use crate::{
    elf::STT_SECTION,
    lexer::Lexer,
    token::{Position, Token, TokenKind, TokenKindError},
};

#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub enum InstrKind {
    #[default]
    None,
    Section,
    Global,
    Local,
    Hidden,
    Internal,
    Protected,
    String,
    Byte,
    Word,
    Long,
    Quad,
    Add,
    Sub,
    InstrOr,
    Adc,
    Sbb,
    Xor,
    And,
    Imul,
    Idiv,
    Div,
    Neg,
    Mul,
    Lea,
    Mov,
    Movabsq,
    Rep,
    Test,
    Movzx,
    Movsx,
    Not,
    Cqto,
    Cltq,
    Cltd,
    Cwtl,
    Cmp,
    Shl,
    Shr,
    Sar,
    Sal,
    Pop,
    Push,
    Call,
    Seto,
    Setno,
    Setb,
    Setnb,
    Setae,
    Setbe,
    Seta,
    Setpo,
    Setl,
    Setg,
    Setle,
    Setge,
    Sete,
    Setne,
    Jmp,
    Jne,
    Je,
    Jl,
    Jg,
    Jle,
    Jge,
    Jbe,
    Jnb,
    Jnbe,
    Jp,
    Ja,
    Js,
    Jb,
    Jns,
    Ret,
    Syscall,
    Nop,
    Hlt,
    Leave,
    Cmovs,
    Cmovns,
    Cmovge,
    Cvttss2sil,
    Cvtsi2ssq,
    Cvtsi2sdq,
    Cvtsd2ss,
    Cvtss2sd,
    Movss,
    Movsd,
    Movd,
    Ucomiss,
    Ucomisd,
    Comisd,
    Comiss,
    Subss,
    Subsd,
    Addss,
    Addsd,
    Mulss,
    Mulsd,
    Divss,
    Divsd,
    Movaps,
    Movups,
    Xorpd,
    Xorps,
    Pxor,
    Label,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Encoder<'a> {
    /// current token
    tok: Token<'a>,
    lexer: Lexer<'a>,
    current_section: &'a str,
    /// map with section name as keys and instruction list as value
    instrs: HashMap<&'a str, Vec<Instr<'a>>>,
}

/// Instruction information
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Instr<'a> {
    pub kind: InstrKind,
    pub code: Vec<u8>,
    pub symbol_name: &'a str,
    pub flags: &'a str,
    pub addr: usize,
    pub binding: u8,
    /// STV_DEFAULT, STV_INTERNAL, STV_HIDDEN, STV_PROTECTED
    pub visibility: u8,
    pub symbol_type: u8,
    pub section: &'a str,
    pub is_jmp_or_call: bool,
    pub pos: Position<'a>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Rela<'a> {
    pub uses: &'a str,
    pub instr: Instr<'a>,
    pub offset: usize,
    pub rtype: u64,
    pub adjust: i32,
    pub is_already_resolved: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Expr<'a> {
    Ident(Ident<'a>),
    Immediate(Immediate<'a>),
    Register(Register<'a>),
    Indirection(Indirection<'a>),
    Number(Number<'a>),
    Binop(Binop<'a>),
    Neg(Neg<'a>),
    Xmm(Xmm<'a>),
    Star(Star<'a>),
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Number<'a> {
    lit: &'a str,
    pos: Position<'a>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Star<'a> {
    regi: Register<'a>,
    pos: Position<'a>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Binop<'a> {
    left_hs: Box<Expr<'a>>,
    right_hs: Box<Expr<'a>>,
    op: TokenKind,
    pos: Position<'a>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Neg<'a> {
    expr: Box<Expr<'a>>,
    pos: Position<'a>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Register<'a> {
    lit: String,
    size: DataSize,
    pos: Position<'a>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Xmm<'a> {
    lit: &'a str,
    pos: Position<'a>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Immediate<'a> {
    expr: Box<Expr<'a>>,
    pos: Position<'a>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Indirection<'a> {
    disp: Box<Expr<'a>>,
    base: Register<'a>,
    index: Register<'a>,
    scale: Box<Expr<'a>>,
    pos: Position<'a>,
    has_base: bool,
    has_index_scale: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Ident<'a> {
    lit: &'a str,
    pos: Position<'a>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct UserDefinedSection {
    pub code: Vec<u8>,
    pub addr: usize,
    pub flags: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum DataSize {
    SuffixByte,
    SuffixWord,
    SuffixLong,
    SuffixQuad,
    SuffixSingle,
    SuffixDouble,
}

const MOD_INDIRECTION_WITH_NO_DISP: u8 = 0;
const MOD_INDIRECTION_WITH_DISP8: u8 = 1;
const MOD_INDIRECTION_WITH_DISP32: u8 = 2;
const MOD_REGI: u8 = 3;
const REX_W: u8 = 0x48;
const OPERAND_SIZE_PREFIX16: u8 = 0x66;
const SLASH_0: usize = 0;
const SLASH_1: usize = 1;
const SLASH_2: usize = 2;
const SLASH_3: usize = 3;
const SLASH_4: usize = 4;
const SLASH_5: usize = 5;
const SLASH_6: usize = 6;
const SLASH_7: usize = 7;

#[rustfmt::skip]
const R8_R15: [&'static str; 32] = [
    "R8",  "R8D",  "R8W",  "R8B",
    "R9",  "R9D",  "R9W",  "R9B",
    "R10", "R10D", "R10W", "R10B",
    "R11", "R11D", "R11W", "R11B",
    "R12", "R12D", "R12W", "R12B",
    "R13", "R13D", "R13W", "R13B",
    "R14", "R14D", "R14W", "R14B",
    "R15", "R15D", "R15W", "R15B",
];

#[rustfmt::skip]
const XMM8_XMM15: [&'static str; 8] = [
    "XMM8",  "XMM9",  "XMM10", "XMM11",
    "XMM12", "XMM13", "XMM14", "XMM15",
];

fn reg_size(name: &str) -> DataSize {
    match name {
        "RAX" | "RCX" | "RDX" | "RBX" | "RSP" | "RBP" | "RSI" | "RDI" | "RIP" | "R8'" | "R9'"
        | "R10" | "R11" | "R12" | "R13" | "R14" | "R15" => DataSize::SuffixQuad,
        "EAX" | "ECX" | "EDX" | "EBX" | "ESP" | "EBP" | "ESI" | "EDI" | "EIP" | "R8D" | "R9D"
        | "R10D" | "R11D" | "R12D" | "R13D" | "R14D" | "R15D" => DataSize::SuffixLong,
        "AX" | "CX" | "DX" | "BX" | "SP" | "BP" | "SI" | "DI" | "IP" | "R8W" | "R9W" | "R10W"
        | "R11W" | "R12W" | "R13W" | "R14W" | "R15W" => DataSize::SuffixWord,
        "AL" | "CL" | "DL" | "BL" | "AH" | "CH" | "DH" | "BH" | "SIL" | "DIL" | "SPL" | "BPL"
        | "R8B" | "R9B" | "R10B" | "R11B" | "R12B" | "R13B" | "R14B" | "R15B" => {
            DataSize::SuffixByte
        }
        _ => panic!("Invalid register name {}", name),
    }
}

impl<'a> TryFrom<&'a str> for DataSize {
    type Error = DataSizeError<'a>;

    /// Predict DataSize from x86_64 register name.
    /// # Must
    /// &str to upper case
    fn try_from(name: &'a str) -> Result<Self, Self::Error> {
        Ok(match name {
            "RAX" | "RCX" | "RDX" | "RBX" | "RSP" | "RBP" | "RSI" | "RDI" | "RIP" | "R8'"
            | "R9'" | "R10" | "R11" | "R12" | "R13" | "R14" | "R15" => DataSize::SuffixQuad,

            "EAX" | "ECX" | "EDX" | "EBX" | "ESP" | "EBP" | "ESI" | "EDI" | "EIP" | "R8D"
            | "R9D" | "R10D" | "R11D" | "R12D" | "R13D" | "R14D" | "R15D" => DataSize::SuffixLong,

            "AX" | "CX" | "DX" | "BX" | "SP" | "BP" | "SI" | "DI" | "IP" | "R8W" | "R9W"
            | "R10W" | "R11W" | "R12W" | "R13W" | "R14W" | "R15W" => DataSize::SuffixWord,

            "AL" | "CL" | "DL" | "BL" | "AH" | "CH" | "DH" | "BH" | "SIL" | "DIL" | "SPL"
            | "BPL" | "R8B" | "R9B" | "R10B" | "R11B" | "R12B" | "R13B" | "R14B" | "R15B" => {
                DataSize::SuffixByte
            }
            _ => return Err(DataSizeError::UnknownRegister(name)),
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, thiserror::Error)]
enum DataSizeError<'a> {
    #[error("Invalid register name {0}")]
    UnknownRegister(&'a str),
}

impl<'a> Encoder<'a> {
    pub fn new(l: &'a mut Lexer<'a>, file_name: &'a str) -> Encoder<'a> {
        let lexer = l.clone();
        let tok = l.lex().unwrap();

        let default_text_section = Instr {
            kind: InstrKind::Section,
            pos: tok.pos.clone(),
            section: ".text",
            symbol_type: STT_SECTION,
            flags: "ax",
            ..Default::default()
        };
        let mut hashmap = HashMap::new();
        hashmap.insert(".text", vec![default_text_section]);

        Self {
            tok,
            lexer,
            current_section: ".text",
            instrs: hashmap,
        }
    }

    /// call `lexer.lex()` & get next token into Self.
    fn next(&'a mut self) {
        self.tok = self.lexer.lex().unwrap();
    }

    fn expect(&self, exp: TokenKind) {
        if self.tok.kind != exp {
            eprintln!("{}", TokenKindError::UnexpectedKind(self.tok.kind.clone()));
            exit(1);
        }
    }

    // fn parse_register(&mut self) -> Register {
    //     self.expect(TokenKind::Percent);
    //     let pos = self.tok.pos.clone();
    //     let reg_name = self.tok.lit.to_uppercase();
    //     let size: DataSize = reg_name
    //         .as_str()
    //         .try_into()
    //         .unwrap_or_else(|err| panic!("{}", err));

    //     self.next();
    //     Register {
    //         lit: reg_name,
    //         size,
    //         pos,
    //     }
    // }
}
