extern crate byteorder;
#[macro_use]
extern crate error_chain;

use std::fs::File;
use std::io::prelude::*;

mod errors;
mod macho;
mod parser;

use errors::*;

fn run() -> Result<()> {
    let mut f = File::open("../program").chain_err(
        || "cannot open program file",
    )?;
    let mut bin: Vec<u8> = vec![];
    f.read_to_end(&mut bin).chain_err(|| "error reading bin")?;

    let p = parser::MachOParser::new(bin);
    let h = p.parse_header().chain_err(|| "failed to parse header")?;
    println!("header: {:?}", h);

    let lcs = p.parse_load_commands(&h).chain_err(|| "failed to load commands")?;

    println!("load commands: {:#?}", lcs);

    return Ok(());
}

fn main() {
    if let Err(ref e) = run() {
        use std::io::Write;
        use error_chain::ChainedError; // trait which holds `display_chain`
        let stderr = &mut ::std::io::stderr();
        let errmsg = "Error writing to stderr";

        writeln!(stderr, "{}", e.display_chain()).expect(errmsg);
        ::std::process::exit(1);
    }
}
