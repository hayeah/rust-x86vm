extern crate byteorder;

use std::fs::File;
use std::io::prelude::*;

use std::io::{Cursor, SeekFrom};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

#[macro_use]
extern crate error_chain;

mod errors {
    // Create the Error, ErrorKind, ResultExt, and Result types
    error_chain!{}
}

use errors::*;

const LC_SEGMENT: u32 = 1;
const LC_UNIXTHREAD: u32 = 5;

struct LCSegment<'a> {
    name: &'a [u8],
}

struct TextSection<'a> {
    name: &'a mut [u8; 16],
    address: u32,
    size: u32,
}

impl<'a> TextSection<'a> {
    fn new(name: &'a mut [u8; 16]) -> TextSection<'a> {
        TextSection {
            name: name,
            address: 0,
            size: 0,
        }
    }
}

const HEADERSIZE: usize = 0x1c;
const LC_SEGMENT_HEADER_SIZE: usize = 56;
const TEXT_SECTION_HEADER_SIZE: u64 = 68;

#[derive(Debug)]
struct Header {
    magic: u32,
    cpu_type: u32,
    cpu_subtype: u32,
    file_type: u32,
    load_commands_count: u32,
    load_commands_size: u32,
    flags: u32,
}

#[derive(Debug)]
struct SectionHeader {}

#[derive(Debug)]
enum LoadCommand {
    Segment {
        name: String, // 2 .. 5
        vm_address: u32, // 6
        vm_sizes: u32,
        file_offset: u32,
        file_size: u32,
        max_vm_protection: u32,
        initial_vm_protection: u32,
        number_of_sections: u32,
        flags: u32,

        // sections: Vec<SectionHeader>,
    },

    Symtab {},

    UnixThread {},

    // just put the size/offset here
    Unsupported {
        cmd: u32,
        size: usize,

        // data: Vec<u8>,
    },
}

struct MachOParser {
    data: Vec<u8>,
}

impl MachOParser {
    fn new(data: Vec<u8>) -> MachOParser {
        MachOParser { data: data }
    }

    fn parse_header(&self) -> Result<Header> {
        let mut r = Cursor::new(&self.data);

        let mut buf = [0 as u32; 7];

        r.read_u32_into::<LittleEndian>(&mut buf).chain_err(
            || "header read fail",
        )?;

        let magic = buf[0];

        if magic != 0xFEEDFACE {
            bail!("invalid MachO magic number")
        }

        return Ok(Header {
            magic: buf[0],
            cpu_type: buf[1],
            cpu_subtype: buf[2],
            file_type: buf[3],
            load_commands_count: buf[4],
            load_commands_size: buf[5],
            flags: buf[6],
        });
    }

    fn parse_load_commands(&self, header: &Header) -> Result<Vec<LoadCommand>> {
        let data = &self.data[HEADERSIZE..];
        let mut lcs: Vec<LoadCommand> = Vec::with_capacity(header.load_commands_count as usize);

        let mut pos: usize = 0;
        for _ in 0..header.load_commands_count {
            let segdata = &data[pos..];

            let mut r = Cursor::new(&segdata);

            let mut buf = [0 as u32; 2];
            r.read_u32_into::<LittleEndian>(&mut buf).chain_err(
                || "header read fail",
            )?;

            let cmd = buf[0];
            let size = buf[1] as usize;

            let segdata = &data[pos..pos+size];

            let lc = match cmd {
                1 => self.parse_lc_segment(segdata),
                _ => Ok(LoadCommand::Unsupported {
                    cmd: cmd,
                    size: size,
                    // data: segdata.to_vec(),
                }),
            }.chain_err(|| "load commands parsing error")?;

            lcs.push(lc);

            pos += size as usize;
        }

        return Ok(lcs);
    }

    fn parse_lc_segment(&self, data: &[u8]) -> Result<LoadCommand> {
        let mut r = Cursor::new(data);

        let mut buf = [0 as u32; LC_SEGMENT_HEADER_SIZE / 4];
        r.read_u32_into::<LittleEndian>(&mut buf).chain_err(
            || "LC_SEGMENT read fail",
        )?;

        // read 16 bytes of segment name
        let mut namebuf = [0 as u8; 16];
        {
            let mut p = &mut namebuf[..];
            for word in &buf[2..6] {
                p.write_u32::<LittleEndian>(*word).unwrap();
            }
        }

        let name = String::from(std::str::from_utf8(&namebuf).unwrap().trim_matches('\0'));

        return Ok(LoadCommand::Segment{
            name: name,
            vm_address: buf[6],
            vm_sizes: buf[7],
            file_offset: buf[8],
            file_size: buf[9],
            max_vm_protection: buf[10],
            initial_vm_protection: buf[11],
            number_of_sections: buf[12],
            flags: buf[13],
        });
    }
}

fn run() -> Result<()> {
    let mut f = File::open("../program").chain_err(
        || "cannot open program file",
    )?;
    let mut bin: Vec<u8> = vec![];
    f.read_to_end(&mut bin).chain_err(|| "error reading bin")?;

    let p = MachOParser::new(bin);
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
