use std::{collections::HashMap, fs, io::Write, mem};

use crate::{
    globals::{RELA_TEXT_USERS, USER_DEFINED_SECTIONS, USER_DEFINED_SYMBOLS},
    utils::any_as_u8_slice,
};

#[derive(Clone, Debug, Default)]
pub struct Elf<'a> {
    out_file: &'a str,
    /// flag to keep local labels. labels that start from `.L`
    keep_locals: bool,
    // Elf header
    ehdr: Elf64Ehdr,
    /// symtab symbol index
    symtab_symbol_indexes: HashMap<String, usize>,
    /// used in .symtab section header
    local_symbols_count: usize,
    /// symbols that are not defined
    rela_symbols: Vec<String>,
    /// list of user-defined section names
    user_defined_section_names: Vec<String>,
    user_defined_section_idx: HashMap<String, usize>,
    section_name_offs: HashMap<String, usize>,
    strtab: Vec<u8>,
    symtab: Vec<Elf64Sym>,
    rela_section_names: Vec<String>,
    rela: HashMap<String, Vec<Elf64Rela>>,
    shstrtab: Vec<u8>,
    section_headers: Vec<Elf64Shdr>,
}

/// [File header](https://en.wikipedia.org/wiki/Executable_and_Linkable_Format#:~:text=header%5B4%5D-,File%20header,-edit)
#[repr(C)] // To prevent auto organize fields.
#[derive(Clone, Debug, Default)]
struct Elf64Ehdr {
    e_ident: [u8; 16],
    e_type: u16,
    e_machine: u16,
    e_version: u32,
    e_entry: usize,
    e_phoff: usize,
    e_shoff: usize,
    e_flags: u32,
    e_ehsize: u16,
    e_phentsize: u16,
    e_phnum: u16,
    e_shentsize: u16,
    e_shnum: u16,
    e_shstrndx: u16,
}

#[repr(C)] // To prevent auto organize fields.
#[derive(Clone, Debug, Default)]
pub struct Elf64Sym {
    st_name: u32,
    st_info: u8,
    st_other: u8,
    st_shndx: u16,
    st_value: usize,
    st_size: u64,
}

/// [Section header](https://en.wikipedia.org/wiki/Executable_and_Linkable_Format#:~:text=Program%20Header%20(size).-,Section%20header,-edit)
#[repr(C)] // To prevent auto organize fields.
#[derive(Clone, Debug, Default)]
struct Elf64Shdr {
    sh_name: u32,
    sh_type: u32,
    sh_flags: u64,
    sh_addr: usize,
    sh_offset: usize,
    sh_size: usize,
    sh_link: u32,
    sh_info: u32,
    sh_addralign: usize,
    sh_entsize: usize,
}

/// Reallocation entries
#[repr(C)] // To prevent auto organize fields.
#[derive(Clone, Debug, Default)]
pub struct Elf64Rela {
    r_offset: u64,
    r_info: u64,
    r_addend: i64,
}

/// [Program header](https://en.wikipedia.org/wiki/Executable_and_Linkable_Format#:~:text=ELF%20Header%20(size).-,Program%20header,-edit)
#[repr(C)] // To prevent auto organize fields.
#[derive(Clone, Debug, Default)]
struct Elf64Phdr {
    ph_type: u32,
    /// Segment-dependent flags (position for 64-bit structure).
    ph_flags: u32,
    /// Offset of the segment in the file image.
    ph_off: u64,
    /// Virtual address of the segment in memory.
    ph_vaddr: u64,
    /// On systems where physical address is relevant, reserved for segment's physical address.
    ph_paddr: u64,
    /// Size in bytes of the segment in the file image. May be 0.
    ph_filesz: u64,
    /// Size in bytes of the segment in memory. May be 0.
    ph_memsz: u64,
    /// 0 and 1 specify no alignment. Otherwise should be a positive,
    /// integral power of 2, with p_vaddr equating p_offset modulus p_align.
    ph_align: u64,
    // padding: u64, // End of Program Header (size).
}

pub const STB_LOCAL: u8 = 0;
pub const STB_GLOBAL: u8 = 1;

pub const STT_NOTYPE: u8 = 0;
pub const STT_OBJECT: u8 = 1;
pub const STT_FUNC: u8 = 2;
pub const STT_SECTION: u8 = 3;
pub const STT_FILE: u8 = 4;
pub const STT_COMMON: u8 = 5;
pub const STT_TLS: u8 = 6;
pub const STT_RELC: u8 = 8;
pub const STT_SRELC: u8 = 9;
pub const STT_LOOS: u8 = 10;
pub const STT_HIOS: u8 = 12;
pub const STT_LOPROC: u8 = 13;
pub const STT_HIPROC: u8 = 14;

pub const SHT_NULL: u32 = 0;
pub const SHT_PROGBITS: u32 = 1;
pub const SHT_SYMTAB: u32 = 2;
pub const SHT_STRTAB: u32 = 3;
pub const SHT_RELA: u32 = 4;

pub const SHF_WRITE: u64 = 0x1;
pub const SHF_ALLOC: u64 = 0x2;
pub const SHF_EXECINSTR: u64 = 0x4;
pub const SHF_MERGE: u64 = 0x10;
pub const SHF_STRINGS: u64 = 0x20;
pub const SHF_INFO_LINK: u64 = 0x40;
pub const SHF_LINK_ORDER: u64 = 0x80;
pub const SHF_OS_NONCONFORMING: u64 = 0x100;
pub const SHF_GROUP: u64 = 0x200;
pub const SHF_TLS: u64 = 0x400;

pub const R_X86_64_NONE: u64 = 0;
pub const R_X86_64_64: u64 = 1;
pub const R_X86_64_PC32: u64 = 2;
pub const R_X86_64_GOT32: u64 = 3;
pub const R_X86_64_PLT32: u64 = 4;
pub const R_X86_64_COPY: u64 = 5;
pub const R_X86_64_GLOB_DAT: u64 = 6;
pub const R_X86_64_JUMP_SLOT: u64 = 7;
pub const R_X86_64_RELATIVE: u64 = 8;
pub const R_X86_64_GOTPCREL: u64 = 9;
pub const R_X86_64_32: u64 = 10;
pub const R_X86_64_32S: u64 = 11;
pub const R_X86_64_16: u64 = 12;
pub const R_X86_64_PC16: u64 = 13;
pub const R_X86_64_8: u64 = 14;
pub const R_X86_64_PC8: u64 = 15;
pub const R_X86_64_PC64: u64 = 24;

pub const STV_DEFAULT: u8 = 0;
pub const STV_INTERNAL: u8 = 1;
pub const STV_HIDDEN: u8 = 2;
pub const STV_PROTECTED: u8 = 3;

impl<'a> Elf<'a> {
    pub fn new(out_file: &'a str, keep_locals: bool) -> Self {
        let mut e = Self {
            out_file,
            keep_locals,
            ..Default::default()
        };

        for (name, _) in USER_DEFINED_SECTIONS.lock().unwrap().iter() {
            e.user_defined_section_names.push(name.clone());
            e.user_defined_section_idx
                .insert(name.clone(), e.user_defined_section_idx.len() + 1);
        }

        e
    }
}

pub fn align_to(n: usize, align: usize) -> usize {
    (n + align - 1) / align * align
}

fn add_padding(code: &mut Vec<u8>) {
    let padding = align_to(code.len(), 16) - code.len();
    code.extend(std::iter::repeat(0).take(padding));
}

impl Elf<'_> {
    fn elf_symbol(&mut self, symbol_binding: u8, off: &mut usize, string: &mut String) {
        for (symbol_name, symbol) in USER_DEFINED_SYMBOLS.lock().unwrap().clone() {
            if symbol.binding != symbol_binding {
                continue;
            }

            if symbol.binding == STB_LOCAL {
                if !self.keep_locals
                    && symbol.binding == STB_LOCAL
                    && symbol_name.to_uppercase().starts_with(".L")
                {
                    continue;
                }
                self.local_symbols_count += 1;
            }

            self.symtab_symbol_indexes
                .insert(symbol_name.clone(), self.symtab_symbol_indexes.len());

            *off += string.len() + 1;
            let st_shndx = self.user_defined_section_idx[symbol.section] as u16;
            let st_name = if symbol.symbol_type == STT_SECTION {
                0
            } else {
                *off as u32
            };

            self.symtab.push(Elf64Sym {
                st_name,
                st_info: (symbol.binding << 4) + (symbol.symbol_type & 0xf),
                st_other: symbol.visibility,
                st_shndx,
                st_value: symbol.addr,
                ..Default::default()
            });

            self.strtab
                .extend_from_slice(format!("{symbol_name}\0").as_bytes());
            *string = symbol_name;
        }
    }

    fn elf_rela_symbol(&mut self, off: &mut usize, string: &mut String) {
        for symbol_name in &self.rela_symbols {
            *off += string.len() + 1;
            self.symtab_symbol_indexes
                .insert(symbol_name.to_owned(), self.symtab_symbol_indexes.len());

            self.symtab.push(Elf64Sym {
                st_name: *off as u32,
                st_info: (STB_GLOBAL << 4) + (STT_NOTYPE & 0xf),
                st_shndx: 0,
                ..Default::default()
            });

            self.strtab
                .extend_from_slice(format!("{symbol_name}\0").as_bytes());
            *string = symbol_name.to_string();
        }
    }

    pub fn rela_text_users(&mut self) {
        for r in RELA_TEXT_USERS.lock().unwrap().clone() {
            let mut index = 0;
            let mut r_addend = if [
                R_X86_64_32S,
                R_X86_64_32,
                R_X86_64_64,
                R_X86_64_32,
                R_X86_64_16,
                R_X86_64_8,
            ]
            .contains(&r.rtype)
            {
                0
            } else if r.rtype == R_X86_64_PC32 {
                (r.offset - r.instr.code.len()) as i64
            } else {
                -4
            };

            // Skip already resolved instruction.
            if r.is_already_resolved {
                continue;
            }

            if let Some(s) = USER_DEFINED_SYMBOLS.lock().unwrap().get(r.uses) {
                if s.binding == STB_GLOBAL {
                    index = self.symtab_symbol_indexes[r.uses];
                } else {
                    r_addend += s.addr as i64;
                    index = self.symtab_symbol_indexes[s.section];
                }
            } else {
                index = self.symtab_symbol_indexes[r.uses];
            }

            let rela_section_name = format!(".rela{}", r.instr.section);
            self.rela
                .entry(rela_section_name.clone())
                .or_insert_with(Vec::new)
                .push(Elf64Rela {
                    r_offset: (r.instr.addr + r.offset) as u64,
                    r_info: ((index as u64) << 32) + r.rtype,
                    r_addend: r_addend + r.adjust as i64,
                });

            if !self.rela_section_names.contains(&rela_section_name) {
                self.rela_section_names.push(rela_section_name);
            }
        }
    }

    pub fn collect_rela_symbols(&mut self) {
        for rela in RELA_TEXT_USERS.lock().unwrap().clone() {
            if !self.rela_symbols.contains(&rela.uses.to_owned()) {
                if USER_DEFINED_SYMBOLS.lock().unwrap().contains_key(rela.uses) {
                    continue;
                }
                self.rela_symbols.push(rela.uses.to_string());
            }
        }
    }

    pub fn build_symtab_strtab(&mut self) {
        // null symbol
        self.strtab.push(0x00);
        self.symtab.push(Elf64Sym {
            st_name: 0,
            st_info: (STB_LOCAL << 4) + (STT_NOTYPE & 0xf),
            ..Default::default()
        });
        self.symtab_symbol_indexes
            .insert(String::new(), self.symtab_symbol_indexes.len());
        self.local_symbols_count += 1;

        let mut off = 0;
        let mut string = String::new();

        self.elf_symbol(STB_LOCAL, &mut off, &mut string); // local
        self.elf_rela_symbol(&mut off, &mut string); // rela local
        self.elf_symbol(STB_GLOBAL, &mut off, &mut string); // global

        add_padding(&mut self.strtab);
    }

    pub fn build_shstrtab(&mut self) {
        // null
        self.shstrtab.push(0x00);
        self.section_name_offs.insert(String::new(), 0);

        // custom sections
        let mut name_offs = 1;
        for name in &self.user_defined_section_names {
            self.section_name_offs.insert(name.clone(), name_offs);
            name_offs += name.len() + 1;

            self.shstrtab.extend_from_slice(name.as_bytes());
            self.shstrtab.push(0x00);
        }

        for name in &[".strtab", ".symtab", ".shstrtab"] {
            self.section_name_offs.insert(name.to_string(), name_offs);
            name_offs += name.len() + 1;

            self.shstrtab.extend_from_slice(name.as_bytes());
            self.shstrtab.push(0x00);
        }

        for name in self.rela.keys() {
            self.section_name_offs.insert(name.clone(), name_offs);
            name_offs += name.len() + 1;

            self.shstrtab.extend_from_slice(name.as_bytes());
            self.shstrtab.push(0x00);
        }

        add_padding(&mut self.shstrtab);
    }

    pub fn build_headers(&mut self) {
        let mut section_offs = mem::size_of::<Elf64Ehdr>();
        let mut section_idx = HashMap::new();
        section_idx.insert(String::new(), 0);

        // null section
        self.section_headers.push(Elf64Shdr {
            sh_name: self.section_name_offs[""] as u32,
            sh_type: SHT_NULL,
            ..Default::default()
        });

        // user-defined sections
        for name in &self.user_defined_section_names {
            let user_symbols = USER_DEFINED_SECTIONS.lock().unwrap();
            let section = match user_symbols.get(name) {
                Some(section) => section,
                None => panic!("unkown section {name}"),
            };

            self.section_headers.push(Elf64Shdr {
                sh_name: self.section_name_offs[name] as u32,
                sh_type: SHT_PROGBITS,
                sh_flags: section.flags,
                sh_offset: section_offs,
                sh_size: section.code.len(),
                sh_addralign: 1,
                ..Default::default()
            });
            section_offs += section.code.len();
            section_idx.insert(name.clone(), section_idx.len());
        }

        let strtab_ofs = section_offs;
        let strtab_size = self.strtab.len();
        section_idx.insert(".strtab".to_string(), section_idx.len());

        // strtab
        self.section_headers.push(Elf64Shdr {
            sh_name: self.section_name_offs[".strtab"] as u32,
            sh_type: SHT_STRTAB,
            sh_offset: strtab_ofs,
            sh_size: strtab_size,
            sh_addralign: 1,
            ..Default::default()
        });

        let symtab_ofs = section_offs;
        let symtab_size = mem::size_of::<Elf64Sym>() * self.strtab.len();
        section_idx.insert(".symtab".to_string(), section_idx.len());

        // .symbtab
        self.section_headers.push(Elf64Shdr {
            sh_name: self.section_name_offs[".symtab"] as u32,
            sh_type: SHT_SYMTAB,
            sh_offset: symtab_ofs,
            sh_size: symtab_size,
            sh_link: section_idx[".strtab"] as u32,
            sh_info: self.local_symbols_count as u32,
            sh_addralign: 8,
            sh_entsize: mem::size_of::<Elf64Sym>(),
            ..Default::default()
        });

        // Add rela ... to section headers
        for name in &self.rela_section_names {
            let size = self.rela[name].len() * mem::size_of::<Elf64Rela>();
            self.section_headers.push(Elf64Shdr {
                sh_name: self.section_name_offs[name] as u32,
                sh_type: SHT_RELA,
                sh_flags: SHF_INFO_LINK,
                sh_addr: 0,
                sh_offset: section_offs,
                sh_size: size,
                sh_link: section_idx[".symtab"] as u32,
                sh_info: section_idx[&name[5..]] as u32, // target section index. if `.rela.text` the target will be `.text`
                sh_addralign: 8,
                sh_entsize: mem::size_of::<Elf64Rela>(),
            });
            section_offs += size;
        }

        // .shstrtab
        self.section_headers.push(Elf64Shdr {
            sh_name: self.section_name_offs[".shstrtab"] as u32,
            sh_type: SHT_STRTAB,
            sh_flags: 0,
            sh_addr: 0,
            sh_offset: section_offs,
            sh_size: self.shstrtab.len(),
            sh_link: 0,
            sh_info: 0,
            sh_addralign: 1,
            sh_entsize: 0,
        });

        let sectionheader_ofs = section_offs + self.shstrtab.len();

        // elf header
        self.ehdr = Elf64Ehdr {
            e_ident: [
                0x7f, 0x45, 0x4c, 0x46, // Magic number ' ELF' in ascii format
                0x02, // 2 = 64-bit
                0x01, // 1 = little endian
                0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            ],
            e_type: 1, // 1 = re allocatable
            e_machine: 0x3e,
            e_version: 1,
            e_entry: 0,
            e_phoff: 0,
            e_shoff: sectionheader_ofs,
            e_flags: 0,
            e_ehsize: mem::size_of::<Elf64Ehdr>() as u16,
            e_phentsize: mem::size_of::<Elf64Phdr>() as u16,
            e_phnum: 0,
            e_shentsize: mem::size_of::<Elf64Shdr>() as u16,
            e_shnum: self.section_headers.len() as u16,
            e_shstrndx: (self.section_headers.len() - 1) as u16,
        }
    }

    pub fn write_elf(&self) {
        let mut fp = fs::File::create(self.out_file)
            .unwrap_or_else(|_| panic!("Error opening file '{}'", self.out_file));

        // Write ELF header
        fp.write_all(unsafe { any_as_u8_slice(&self.ehdr) })
            .expect("Error writing ELF header");

        // Write user-defined sections
        let user_sections = USER_DEFINED_SECTIONS.lock().unwrap();
        for name in &self.user_defined_section_names {
            let section = user_sections
                .get(name)
                .unwrap_or_else(|| panic!("Unknown section '{}'", name));
            fp.write_all(&section.code)
                .unwrap_or_else(|_| panic!("Error writing section '{}'", name));
        }

        // Write .strtab
        fp.write_all(&self.strtab).expect("Error writing '.strtab'");

        // Write .symtab
        for s in &self.symtab {
            fp.write_all(unsafe { any_as_u8_slice(&s) })
                .expect("Error writing '.symtab'");
        }

        // Write relocation sections
        for name in &self.rela_section_names {
            if let Some(rela_section) = self.rela.get(name) {
                for r in rela_section {
                    fp.write_all(unsafe { any_as_u8_slice(&r) })
                        .expect("Error writing '.rela.text'");
                }
            }
        }

        // Write .shstrtab
        fp.write_all(&self.shstrtab)
            .expect("Error writing '.shstrtab'");

        // Write section headers
        for sh in &self.section_headers {
            fp.write_all(unsafe { any_as_u8_slice(sh) })
                .expect("Error writing section headers");
        }
    }
}
