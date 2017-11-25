extern crate byteorder;

use std::fs::File;
use std::io::prelude::*;

use std::io::{Cursor, SeekFrom};
use byteorder::{LittleEndian, ReadBytesExt};

#[macro_use]
extern crate error_chain;

mod errors {
    // Create the Error, ErrorKind, ResultExt, and Result types
    error_chain!{}
}

use errors::*;

struct LoadCommand {
    cmd: u32,
    size: u32,
}

const LC_SEGMENT: u32 = 1;
const LC_UNIXTHREAD: u32 = 5;

struct LCSegment<'a> {
    name: &'a [u8],
}

struct TextSection<'a> {
    name: &'a mut [u8;16],
    address: u32,
    size: u32,
}

impl<'a> TextSection<'a> {
    fn new(name: &'a mut [u8;16]) -> TextSection<'a> {
        TextSection {
            name: name,
            address: 0,
            size: 0,
        }
    }
}

const HEADERSIZE: u64 = 0x1c;
const TEXT_SEGMENT_HEADER_SIZE: u64 = 56;
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

struct MachOParser {
    data: Vec<u8>,
}

impl MachOParser {
    fn new(data: Vec<u8>) -> MachOParser {
        MachOParser {
            data: data,
        }
    }

    fn parse_header(&self) -> Result<Header> {
        let mut r = Cursor::new(&self.data);

        let mut buf = [0 as u32; 7];

        r.read_u32_into::<LittleEndian>(&mut buf).chain_err(|| "header read fail")?;

        let magic = buf[0];

        if magic != 0xFEEDFACE {
            bail!("invalid MachO magic number")
        }

        return Ok(Header{
            magic: buf[0],
            cpu_type: buf[1],
            cpu_subtype: buf[2],
            file_type: buf[3],
            load_commands_count: buf[4],
            load_commands_size: buf[5],
            flags: buf[6],
        });
    }
}

// impl<'a> MachOBin<'a> {
//     fn new(data: &'a [u8]) -> MachOBin {
//         MachOBin {
//             data: data,
//             r: Cursor::new(data),
//         }
//     }

//     fn valid_magic(&mut self) -> bool {
//         // let mut r = Cursor::new(&self.data[0..4]);
//         self.seek(0);
//         self.read() == 0xfeedface
//     }

//     fn process_load_commands(&mut self) {
//         // let mut r = Cursor::new(&self.data[..]);
//         self.seek(0x10);

//         let load_cmd_count: u32 = self.read();
//         let load_cmd_size: u32 = self.read();

//         println!("load cmd(count={},size={})", load_cmd_count, load_cmd_size);

//         self.seek(HEADERSIZE);

//         for _ in 0..load_cmd_count {
//             let segstart = self.r.position();
//             let cmd = self.read();
//             let size = self.read();

//             // self.r.read_exact

//             match cmd {
//                 LC_SEGMENT => {
//                     let mut buf = [0;16];

//                     self.r.read_exact(&mut buf).unwrap();

//                     let name = std::str::from_utf8(&buf).unwrap().trim_matches('\0');

//                     println!("segment name: (len={}) {:?} ", name.len(), name);

//                     if name == "__TEXT" {
//                         self.seek(segstart+(48 as u64));
//                         let sections_count: u32 = self.read();
//                         for i in 0..sections_count {
//                             let mut buf: [u8; 16] = [0; 16];
//                             let mut ts = TextSection::new(&mut buf);
//                             let secstart = segstart + TEXT_SEGMENT_HEADER_SIZE + TEXT_SECTION_HEADER_SIZE * i as u64;
//                             self.parse_text_section(secstart, &mut ts);

//                             let name = std::str::from_utf8(ts.name).unwrap().trim_matches('\0');
//                             println!("section name: (address={:x}) (size={}) {:?} ", ts.address, ts.size, name);
//                         }
//                     }

//                 },
//                 _ => println!("unrecognized load command type: {}", cmd),
//             };

//             self.seek(segstart + size as u64);
//         }
//     }

//     // fn parse_text_sections() {

//     // }

//     fn parse_text_section(&mut self, offset: u64, ts: &mut TextSection) {
//         self.seek(offset);
//         self.r.read_exact(ts.name).unwrap();

//         self.seek(offset+32);
//         let address: u32 = self.read();
//         let size: u32 = self.read();

//         ts.size = size;
//         ts.address = address;
//     }

//     fn seek(&mut self, pos: u64) {
//         self.r.seek(SeekFrom::Start(pos)).unwrap();
//     }

//     fn read(&mut self) -> u32 {
//         return self.r.read_u32::<LittleEndian>().unwrap();
//     }

//     //
// }

fn run() -> Result<()> {
    let mut f = File::open("../program").chain_err(|| "cannot open program file")?;
    let mut bin: Vec<u8> = vec![];
    f.read_to_end(&mut bin).chain_err(|| "error reading bin")?;

    let p = MachOParser::new(bin);
    let h = p.parse_header().chain_err(|| "failed to parse header")?;
    println!("header: {:?}", h);
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
