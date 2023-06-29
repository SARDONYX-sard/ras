mod elf;
mod encoder;
mod globals;
mod lexer;
mod token;
mod utils;

use std::fs;

use clap::{arg, command, Parser};

use crate::elf::Elf;
use crate::encoder::Encoder;
use crate::lexer::Lexer;

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// assembly file path name
    #[clap(value_parser)]
    name: String,
    /// Output file path name
    #[arg(short, long, default_value_t = format!("./out.o"))]
    out_file: String,
    /// Keeps local symbols (e.g., those starting with `.L`
    #[arg(short, long, default_value_t = false)]
    keep_locals: bool,
}

fn main() {
    let args = Args::parse();

    let file_name = args.name.as_str();
    let program = fs::read_to_string(file_name).expect("Should have been able to read the file");

    let mut l = Lexer::new(file_name, program.as_str());
    let mut en = Encoder::new(&mut l, file_name);
    // en.encode();
    en.assign_addresses();

    let out_file = args.out_file.as_str();
    let mut e = Elf::new(out_file, args.keep_locals);
    e.collect_rela_symbols();
    e.build_symtab_strtab();
    e.rela_text_users();
    e.build_shstrtab();
    e.build_headers();
    e.write_elf();
}
