// use std::process::exit;

// use crate::{elf::R_X86_64_32S};
// use crate::globals::RELA_TEXT_USERS;
// use crate::encoder::{Rela, SLASH_6,Instr, InstrKind, Encoder};

// impl Encoder<'_> {
//     fn pop(&mut self) {
//         let mut instr = Instr {
//             kind: InstrKind::Pop,
//             section: self.current_section,
//             pos: self.tok.pos,
//             ..Default::default()
//         };
//         self.instrs
//             .get_mut(&self.current_section)
//             .unwrap()
//             .push(instr.clone());

//         let source = self.parse_operand();

//         if let Operand::Register(reg) = source {
//             reg.check_regi_size(Suffix::Quad);
//             instr.add_rex_prefix("", "", reg.lit, &[]);
//             instr.code.push(0x58 + reg.regi_bits());
//             return;
//         }

//         if let Operand::Indirection(ind) = source {
//             instr.add_segment_override_prefix(&ind);
//             instr.add_rex_prefix("", ind.index.lit, ind.base.lit, &[]);
//             instr.code.(0x8f); // op_code
//             instr.add_modrm_sib_disp(&ind, SLASH_0);
//             return;
//         }

//         eprintln!("invalid operand for instruction. {}", source.pos);
//         exit(1);
//     }

//     fn push(&mut self) {
//         let mut instr = Instr {
//             kind: InstrKind::Push,
//             section: self.current_section,
//             pos: self.tok.pos,
//             ..Default::default()
//         };
//         self.instrs
//             .get_mut(&self.current_section)
//             .unwrap()
//             .push(instr.clone());

//         let source = self.parse_operand();

//         if let Operand::Register(reg) = source {
//             reg.check_regi_size(Suffix::Quad);
//             if r8_r15.contains(&source.lit) {
//                 instr.code.push(rex(0, 0, 0, 1));
//             }
//             source.check_regi_size(Suffix::Quad);
//             instr.code.push(0x50 + source.regi_bits());
//             return;
//         }

//         if let Operand::Indirection(ind) = source {
//             instr.add_segment_override_prefix(&ind);
//             instr.add_rex_prefix("", ind.index.lit, ind.base.lit, &[]);
//             instr.code.push(0xff); // op_code
//             instr.add_modrm_sib_disp(&ind, SLASH_6);
//             return;
//         }

//         if let Operand::Immediate(imm) = source {
//             let mut used_symbols = Vec::<String>::new();
//             let imm_val = eval_expr_get_symbol(&imm, &mut used_symbols);
//             if used_symbols.len() >= 2 {
//                 eprintln!("invalid immediate operand. {}", imm.pos);
//                 exit(1);
//             }
//             let imm_need_rela = used_symbols.len() == 1;
//             if imm_need_rela {
//                 instr.code.push(0x68);
//                 instr.code.push(0);
//                 instr.code.push(0);
//                 instr.code.push(0);
//                 instr.code.push(0);
//                 RELA_TEXT_USERS.lock().unwrap().push(Rela {
//                     uses: used_symbols[0].as_str(),
//                     instr,
//                     offset: 0x1,
//                     rtype: R_X86_64_32S,
//                     adjust: imm_val,
//                     ..Default::default()
//                 });
//             } else   {
//               match imm_val{
//                 i8::MIN..=i8::MAX => {
//                     instr.code = vec![0x6a, imm_val as u8];
//                 },
//                 i32::MIN..=i32::MAX =>{
//                     let hex = u32::to_le_bytes(imm_val as u32);
//                     instr.code = vec![0x68, hex[0], hex[1], hex[2], hex[3]];
//                 }
//                 _=>{}

//               };
//             };
//             return;
//         }

//         eprintln!("invalid operand for instruction. {}", source.pos);
//         exit(1);
//     }
// }
