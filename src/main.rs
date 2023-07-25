mod elf;
mod encoder;
mod error;
mod globals;
mod lexer;
mod utils;

use crate::elf::Elf;
use crate::encoder::parse;
use crate::error::{bail, Result};
use crate::lexer::tokenize;
use clap::{arg, command, Parser};
use std::fs;

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// assembly file path name
    #[clap(value_parser)]
    file_name: String,
    /// Output file path name
    #[arg(short, long, default_value_t = format!("./out.o"))]
    out_file: String,
    /// Keeps local symbols (e.g., those starting with `.L`
    #[arg(short, long, default_value_t = false)]
    keep_locals: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let program = match fs::read_to_string(args.file_name) {
        Ok(src) => src,
        Err(err) => bail!("{err}"),
    };
    let tokens = tokenize(&program)?;
    dbg!(&tokens);

    parse(tokens)?;
    // let mut en = Encoder::new(&mut l, file_name);
    // en.encode();
    // en.assign_addresses();

    let mut e = Elf::new(&args.out_file, args.keep_locals);
    e.collect_rela_symbols();
    e.build_symtab_strtab();
    e.rela_text_users();
    e.build_shstrtab();
    e.build_headers();
    e.write_elf();
    Ok(())
}
