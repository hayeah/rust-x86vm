extern crate byteorder;
#[macro_use]
extern crate error_chain;
extern crate hexplay;

use std::fs::File;
use std::io::prelude::*;
use hexplay::HexViewBuilder;

mod errors;
mod macho;
mod parser;
mod vm;

use errors::*;


fn run() -> Result<()> {
    let args: Vec<String> = ::std::env::args().collect();

    println!("args: {:?}", args);
    if args.len() < 2 {
        bail!("please specify a MachO binary to load");
    }

    let program = &args[1];

    let mut f = File::open(program).chain_err(|| "cannot open program file")?;
    let mut data: Vec<u8> = vec![];
    f.read_to_end(&mut data).chain_err(|| "error reading bin")?;

    let bin = parser::Macho::parse_bin(&data).chain_err(
        || "parse MachO binary failed",
    )?;

    // println!("bin: {:#?}", bin);

    let text = bin.text().chain_err(|| "cannot find program text")?;
    let view = HexViewBuilder::new(&text).row_width(16).finish();
    println!("text\n: {}", view);

    // 1024 KB
    let mut m = vm::VM::new(1024 * 1024);

    m.run(&bin).chain_err(|| "cannot run binary with VM")?;

    println!("exit: {}", m.exit_status.unwrap());

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
