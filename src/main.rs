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

    let bin = parser::Macho::parse_bin(&bin).chain_err(
        || "parse MachO binary failed",
    )?;

    println!("bin: {:#?}", bin);

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
