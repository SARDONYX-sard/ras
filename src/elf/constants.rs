pub(crate) const STB_LOCAL: u8 = 0;
pub(crate) const STB_GLOBAL: u8 = 1;

pub(crate) const STT_NOTYPE: u8 = 0;
pub(crate) const STT_OBJECT: u8 = 1;
pub(crate) const STT_FUNC: u8 = 2;
pub(crate) const STT_SECTION: u8 = 3;
pub(crate) const STT_FILE: u8 = 4;
pub(crate) const STT_COMMON: u8 = 5;
pub(crate) const STT_TLS: u8 = 6;
pub(crate) const STT_RELC: u8 = 8;
pub(crate) const STT_SRELC: u8 = 9;
pub(crate) const STT_LOOS: u8 = 10;
pub(crate) const STT_HIOS: u8 = 12;
pub(crate) const STT_LOPROC: u8 = 13;
pub(crate) const STT_HIPROC: u8 = 14;

pub(crate) const SHT_NULL: u32 = 0;
pub(crate) const SHT_PROGBITS: u32 = 1;
pub(crate) const SHT_SYMTAB: u32 = 2;
pub(crate) const SHT_STRTAB: u32 = 3;
pub(crate) const SHT_RELA: u32 = 4;

pub(crate) const SHF_WRITE: u64 = 0x1;
pub(crate) const SHF_ALLOC: u64 = 0x2;
pub(crate) const SHF_EXECINSTR: u64 = 0x4;
pub(crate) const SHF_MERGE: u64 = 0x10;
pub(crate) const SHF_STRINGS: u64 = 0x20;
pub(crate) const SHF_INFO_LINK: u64 = 0x40;
pub(crate) const SHF_LINK_ORDER: u64 = 0x80;
pub(crate) const SHF_OS_NONCONFORMING: u64 = 0x100;
pub(crate) const SHF_GROUP: u64 = 0x200;
pub(crate) const SHF_TLS: u64 = 0x400;

pub(crate) const R_X86_64_NONE: u64 = 0;
pub(crate) const R_X86_64_64: u64 = 1;
pub(crate) const R_X86_64_PC32: u64 = 2;
pub(crate) const R_X86_64_GOT32: u64 = 3;
pub(crate) const R_X86_64_PLT32: u64 = 4;
pub(crate) const R_X86_64_COPY: u64 = 5;
pub(crate) const R_X86_64_GLOB_DAT: u64 = 6;
pub(crate) const R_X86_64_JUMP_SLOT: u64 = 7;
pub(crate) const R_X86_64_RELATIVE: u64 = 8;
pub(crate) const R_X86_64_GOTPCREL: u64 = 9;
pub(crate) const R_X86_64_32: u64 = 10;
pub(crate) const R_X86_64_32S: u64 = 11;
pub(crate) const R_X86_64_16: u64 = 12;
pub(crate) const R_X86_64_PC16: u64 = 13;
pub(crate) const R_X86_64_8: u64 = 14;
pub(crate) const R_X86_64_PC8: u64 = 15;
pub(crate) const R_X86_64_PC64: u64 = 24;

pub(crate) const STV_DEFAULT: u8 = 0;
pub(crate) const STV_INTERNAL: u8 = 1;
pub(crate) const STV_HIDDEN: u8 = 2;
pub(crate) const STV_PROTECTED: u8 = 3;
