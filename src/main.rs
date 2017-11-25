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


const HEADERSIZE: usize = 0x1c;
const LC_SEGMENT_HEADER_SIZE: usize = 56;
const LC_SEGMENT_SECTION_HEADER_SIZE: usize = 68;

fn readname(data: &[u8]) -> String {
    unsafe {
        let s = std::str::from_utf8_unchecked(data).trim_matches('\0');
        return String::from(s);
    }
}

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
struct SectionHeader {
    // 0..16
    section_name: String,
    // 16..32
    segment_name: String,
    // 32
    address: u32,
    size: u32,
    offset: u32,
    alignment: u32,
    // relocations_ffset: u32,
}

impl SectionHeader {
    fn parse(data: &[u8]) -> Result<SectionHeader> {
        let section_name = readname(&data[0..16]);
        let segment_name = readname(&data[0..16]);

        let mut words = [0 as u32; LC_SEGMENT_SECTION_HEADER_SIZE / 4];

        let mut r = Cursor::new(data);
        r.read_u32_into::<LittleEndian>(&mut words).chain_err(
            || "section header read fail",
        )?;

        return Ok(SectionHeader{
            section_name: section_name,
            segment_name: segment_name,
            address: words[8],
            size: words[9],
            offset: words[10],
            alignment: words[11],
        });
    }
}

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

        section_headers: Vec<SectionHeader>,
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
            || "LC_SEGMENT read failed",
        )?;

        // read 16 bytes as segment name from offset 8
        let name = readname(&data[8..8+16]);

        let number_of_sections = buf[12];

        let mut section_headers: Vec<SectionHeader> = Vec::with_capacity(number_of_sections as usize);
        let sections_data = &data[LC_SEGMENT_HEADER_SIZE..];
        let mut section_pos = 0;
        for _ in 0..number_of_sections {
            let sh = SectionHeader::parse(&sections_data[section_pos..section_pos+LC_SEGMENT_SECTION_HEADER_SIZE]).chain_err(|| "section header parse failed")?;

            section_headers.push(sh);

            section_pos += LC_SEGMENT_SECTION_HEADER_SIZE;
        }

        return Ok(LoadCommand::Segment{
            name: name,
            vm_address: buf[6],
            vm_sizes: buf[7],
            file_offset: buf[8],
            file_size: buf[9],
            max_vm_protection: buf[10],
            initial_vm_protection: buf[11],
            number_of_sections: number_of_sections,
            flags: buf[13],
            section_headers: section_headers,
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
