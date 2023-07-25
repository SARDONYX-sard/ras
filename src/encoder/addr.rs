use byteorder::{LittleEndian, WriteBytesExt};

use crate::elf::{
    R_X86_64_PC32, SHF_ALLOC, SHF_EXECINSTR, SHF_WRITE, STB_GLOBAL, STB_LOCAL, STV_HIDDEN,
    STV_INTERNAL, STV_PROTECTED,
};
use crate::encoder::{Encoder, Instr, InstrKind, UserDefinedSection};
use crate::globals::{RELA_TEXT_USERS, USER_DEFINED_SECTIONS, USER_DEFINED_SYMBOLS};

fn section_flags(flags: &str) -> u64 {
    let mut val = 0;
    for c in flags.chars() {
        match c {
            'a' => val |= SHF_ALLOC,
            'x' => val |= SHF_EXECINSTR,
            'w' => val |= SHF_WRITE,
            _ => panic!("unknown attribute '{}'", c),
        }
    }
    val
}

fn change_symbol_binding(instr: &Instr, binding: u8) {
    let mut user_symbols = USER_DEFINED_SYMBOLS.lock().unwrap();
    let cache_instr = user_symbols.get_mut(instr.symbol_name).unwrap_or_else(|| {
        panic!("{} undefined symbol '{}'", instr.loc, instr.symbol_name);
    });

    if binding == STB_GLOBAL && cache_instr.kind == InstrKind::Section {
        panic!("{} sections cannot be global", instr.loc);
    }

    cache_instr.binding = binding;
}

fn change_symbol_visibility(instr: &Instr, visibility: u8) {
    let mut bindings = USER_DEFINED_SYMBOLS.lock().unwrap();
    let s = bindings.get_mut(instr.symbol_name).unwrap_or_else(|| {
        panic!("{} undefined symbol '{}'", instr.loc, instr.symbol_name);
    });

    s.visibility = visibility;
}

fn fix_same_section_relocations() {
    for mut rela in RELA_TEXT_USERS.lock().unwrap().iter_mut() {
        if let Some(symbol) = USER_DEFINED_SYMBOLS.lock().unwrap().get(rela.uses) {
            if symbol.section != rela.instr.section {
                continue;
            }
            if symbol.binding == STB_GLOBAL {
                continue;
            }

            if !rela.instr.is_jmp_or_call && rela.rtype != R_X86_64_PC32 {
                continue;
            }

            let num =
                ((symbol.addr - rela.instr.addr) - rela.instr.code.len()) + rela.adjust as usize;

            let mut hex = vec![0u8; 4];
            let mut binding = USER_DEFINED_SECTIONS.lock().unwrap();
            let user = binding.get_mut(rela.instr.section).unwrap();
            hex.write_u32::<LittleEndian>(num as u32).unwrap();
            user.code[rela.instr.addr + rela.offset] = hex[0];
            user.code[rela.instr.addr + rela.offset + 1] = hex[1];
            user.code[rela.instr.addr + rela.offset + 2] = hex[2];
            user.code[rela.instr.addr + rela.offset + 3] = hex[3];

            rela.is_already_resolved = true;
        }
    }
}

impl Encoder<'_> {
    pub fn assign_addresses(&mut self) {
        for (name, mut instrs) in self.instrs.clone() {
            if !USER_DEFINED_SECTIONS.lock().unwrap().contains_key(name) {
                USER_DEFINED_SECTIONS
                    .lock()
                    .unwrap()
                    .insert(name.to_owned(), UserDefinedSection::default());
            }
            let mut bindings = USER_DEFINED_SECTIONS.lock().unwrap();
            let mut section = bindings.get_mut(name).unwrap();

            for mut i in instrs.iter_mut() {
                match i.kind {
                    InstrKind::Section => section.flags = section_flags(i.flags),
                    InstrKind::Global => change_symbol_binding(i, STB_GLOBAL),
                    InstrKind::Local => change_symbol_binding(i, STB_LOCAL),
                    InstrKind::Hidden => change_symbol_visibility(i, STV_HIDDEN),
                    InstrKind::Internal => change_symbol_visibility(i, STV_INTERNAL),
                    InstrKind::Protected => change_symbol_visibility(i, STV_PROTECTED),
                    _ => {}
                }

                i.addr = section.addr;
                section.addr += i.code.len();
                section.code.extend_from_slice(&i.code);
            }
        }

        fix_same_section_relocations();
    }
}
