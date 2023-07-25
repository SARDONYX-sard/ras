use crate::error::{bail, Result};
use seq_macro::seq;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct Register {
    pub(crate) lit: &'static str,
    pub(crate) size: DataSizeSuffix,
    pub(crate) base_offset: u8,
    /// Need rex prefix?
    pub(crate) rex_required: bool,
}

#[derive(Clone, Debug, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum DataSizeSuffix {
    Byte,
    Word,
    Long,
    Quad,
    #[default]
    Unknown,
}

macro_rules! register_tuple {
    ($lit:expr, $base_offset:expr, $size:expr) => {
        register_tuple!($lit, $base_offset, $size, false)
    };
    ($lit:expr, $base_offset:expr, $size:expr, $rex_required:expr) => {
        (
            $lit,
            Register {
                lit: $lit,
                base_offset: $base_offset,
                size: $size,
                rex_required: $rex_required,
            },
        )
    };
}

// I used seq! macro at first but stopped. The reasons are as follows.
// - Registers 1~7 eventually require pattern matching which is as inefficient as handwriting.
// - It is not worth the cost of poor readability.
#[rustfmt::skip]
const GENERAL_REGISTERS: [(&str, Register); 72] = [
    // 64bit
    register_tuple!( "RAX",  0, DataSizeSuffix::Quad),
    register_tuple!( "RCX",  1, DataSizeSuffix::Quad),
    register_tuple!( "RDX",  2, DataSizeSuffix::Quad),
    register_tuple!( "RBX",  3, DataSizeSuffix::Quad),
    register_tuple!( "RSP",  4, DataSizeSuffix::Quad),
    register_tuple!( "RBP",  5, DataSizeSuffix::Quad),
    register_tuple!( "RSI",  6, DataSizeSuffix::Quad),
    register_tuple!( "RDI",  7, DataSizeSuffix::Quad),
    register_tuple!(  "R8",  8, DataSizeSuffix::Quad),
    register_tuple!(  "R9",  9, DataSizeSuffix::Quad),
    register_tuple!( "R10", 10, DataSizeSuffix::Quad),
    register_tuple!( "R11", 11, DataSizeSuffix::Quad),
    register_tuple!( "R12", 12, DataSizeSuffix::Quad),
    register_tuple!( "R13", 13, DataSizeSuffix::Quad),
    register_tuple!( "R14", 14, DataSizeSuffix::Quad),
    register_tuple!( "R15", 15, DataSizeSuffix::Quad),
    // 32bit
    register_tuple!( "EAX",  0, DataSizeSuffix::Long),
    register_tuple!( "ECX",  1, DataSizeSuffix::Long),
    register_tuple!( "EDX",  2, DataSizeSuffix::Long),
    register_tuple!( "EBX",  3, DataSizeSuffix::Long),
    register_tuple!( "ESP",  4, DataSizeSuffix::Long),
    register_tuple!( "EBP",  5, DataSizeSuffix::Long),
    register_tuple!( "ESI",  6, DataSizeSuffix::Long),
    register_tuple!( "EDI",  7, DataSizeSuffix::Long),
    register_tuple!( "R8D",  8, DataSizeSuffix::Long),
    register_tuple!( "R9D",  9, DataSizeSuffix::Long),
    register_tuple!("R10D", 10, DataSizeSuffix::Long),
    register_tuple!("R11D", 11, DataSizeSuffix::Long),
    register_tuple!("R12D", 12, DataSizeSuffix::Long),
    register_tuple!("R13D", 13, DataSizeSuffix::Long),
    register_tuple!("R14D", 14, DataSizeSuffix::Long),
    register_tuple!("R15D", 15, DataSizeSuffix::Long),
    // 16bit
    register_tuple!(  "AX",  0, DataSizeSuffix::Word),
    register_tuple!(  "CX",  1, DataSizeSuffix::Word),
    register_tuple!(  "DX",  2, DataSizeSuffix::Word),
    register_tuple!(  "BX",  3, DataSizeSuffix::Word),
    register_tuple!(  "SP",  4, DataSizeSuffix::Word),
    register_tuple!(  "BP",  5, DataSizeSuffix::Word),
    register_tuple!(  "SI",  6, DataSizeSuffix::Word),
    register_tuple!(  "DI",  7, DataSizeSuffix::Word),
    register_tuple!( "R8W",  8, DataSizeSuffix::Word),
    register_tuple!( "R9W",  9, DataSizeSuffix::Word),
    register_tuple!("R10W", 10, DataSizeSuffix::Word),
    register_tuple!("R11W", 11, DataSizeSuffix::Word),
    register_tuple!("R12W", 12, DataSizeSuffix::Word),
    register_tuple!("R13W", 13, DataSizeSuffix::Word),
    register_tuple!("R14W", 14, DataSizeSuffix::Word),
    register_tuple!("R15W", 15, DataSizeSuffix::Word),
    // 8bit
    register_tuple!(  "AL",  0, DataSizeSuffix::Byte),
    register_tuple!(  "CL",  1, DataSizeSuffix::Byte),
    register_tuple!(  "DL",  2, DataSizeSuffix::Byte),
    register_tuple!(  "BL",  3, DataSizeSuffix::Byte),
    register_tuple!(  "AH",  4, DataSizeSuffix::Byte),
    register_tuple!(  "BP",  5, DataSizeSuffix::Byte),
    register_tuple!(  "CH",  5, DataSizeSuffix::Byte),
    register_tuple!(  "DH",  6, DataSizeSuffix::Byte),
    register_tuple!(  "BH",  7, DataSizeSuffix::Byte),
    register_tuple!( "SPL",  4, DataSizeSuffix::Byte, true),
    register_tuple!( "BPL",  5, DataSizeSuffix::Byte, true),
    register_tuple!( "SIL",  6, DataSizeSuffix::Byte, true),
    register_tuple!( "DIL",  7, DataSizeSuffix::Byte, true),
    register_tuple!( "R8B",  8, DataSizeSuffix::Byte),
    register_tuple!( "R9B",  9, DataSizeSuffix::Byte),
    register_tuple!("R10B", 10, DataSizeSuffix::Byte),
    register_tuple!("R11B", 11, DataSizeSuffix::Byte),
    register_tuple!("R12B", 12, DataSizeSuffix::Byte),
    register_tuple!("R13B", 13, DataSizeSuffix::Byte),
    register_tuple!("R14B", 14, DataSizeSuffix::Byte),
    register_tuple!("R15B", 15, DataSizeSuffix::Byte),
    // instruction pointers(counter)
    register_tuple!( "RIP",  0, DataSizeSuffix::Quad),
    register_tuple!( "EIP",  0, DataSizeSuffix::Long),
    register_tuple!(  "IP",  0, DataSizeSuffix::Word),
];

macro_rules! xmm_entry {
    ($index:expr) => {
        (
            concat!("XMM", stringify!($index)),
            Register {
                lit: concat!("XMM", stringify!($index)),
                base_offset: $index,
                size: DataSizeSuffix::Unknown,
                rex_required: false,
            },
        )
    };
}

seq!(N in 0..16 {
const XMM_REGISTERS: [(&str, Register); 16] = [
    #(xmm_entry!(N),)*
];
});

/// Get(Copy) general register from GENERAL global const by register name.
pub(crate) fn get_reg_info_by(reg_name: &str) -> Result<Register> {
    let e = GENERAL_REGISTERS
        .into_iter()
        .find(|(reg, _)| reg == &reg_name);
    match e {
        Some(v) => Ok(v.1),
        None => bail!("No such general purpose register could be found."),
    }
}

/// Get(Copy) XMM register from XMM global const by register name.
pub(crate) fn get_xmm_by(reg_name: &str) -> Result<Register> {
    let e = XMM_REGISTERS.into_iter().find(|(reg, _)| reg == &reg_name);
    match e {
        Some(v) => Ok(v.1),
        None => bail!("Not such XMM register could be found."),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn show_registers() {
        dbg!(GENERAL_REGISTERS);
        dbg!(XMM_REGISTERS);
    }

    #[test]
    fn should_get_registers() {
        assert_eq!(
            Ok(Register {
                lit: "R12",
                size: DataSizeSuffix::Quad,
                base_offset: 12,
                rex_required: false
            }),
            get_reg_info_by("R12")
        );

        assert_eq!(
            Ok(Register {
                lit: "XMM11",
                size: DataSizeSuffix::Unknown,
                base_offset: 11,
                rex_required: false
            }),
            get_xmm_by("XMM11")
        );
    }
}
