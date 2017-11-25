extern crate byteorder;

use std::fs::File;
use std::io::prelude::*;

use std::io::{Cursor, SeekFrom};
use byteorder::{LittleEndian, ReadBytesExt};

struct MachOBin<'a> {
    data: &'a [u8],

    r: Cursor<&'a [u8]>,
}

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

impl<'a> MachOBin<'a> {
    fn new(data: &'a [u8]) -> MachOBin {
        MachOBin {
            data: data,
            r: Cursor::new(data),
        }
    }

    fn valid_magic(&mut self) -> bool {
        // let mut r = Cursor::new(&self.data[0..4]);
        self.seek(0);
        self.read() == 0xfeedface
    }

    fn process_load_commands(&mut self) {
        // let mut r = Cursor::new(&self.data[..]);
        self.seek(0x10);

        let load_cmd_count: u32 = self.read();
        let load_cmd_size: u32 = self.read();

        println!("load cmd(count={},size={})", load_cmd_count, load_cmd_size);

        self.seek(HEADERSIZE);

        for _ in 0..load_cmd_count {
            let segstart = self.r.position();
            let cmd = self.read();
            let size = self.read();

            // self.r.read_exact

            match cmd {
                LC_SEGMENT => {
                    let mut buf = [0;16];

                    self.r.read_exact(&mut buf).unwrap();

                    let name = std::str::from_utf8(&buf).unwrap().trim_matches('\0');

                    println!("segment name: (len={}) {:?} ", name.len(), name);

                    if name == "__TEXT" {
                        self.seek(segstart+(48 as u64));
                        let sections_count: u32 = self.read();
                        for i in 0..sections_count {
                            let mut buf: [u8; 16] = [0; 16];
                            let mut ts = TextSection::new(&mut buf);
                            let secstart = segstart + TEXT_SEGMENT_HEADER_SIZE + TEXT_SECTION_HEADER_SIZE * i as u64;
                            self.parse_text_section(secstart, &mut ts);

                            let name = std::str::from_utf8(ts.name).unwrap().trim_matches('\0');
                            println!("section name: (address={:x}) (size={}) {:?} ", ts.address, ts.size, name);
                        }
                    }

                },
                _ => println!("unrecognized load command type: {}", cmd),
            };

            self.seek(segstart + size as u64);
        }
    }

    // fn parse_text_sections() {

    // }

    fn parse_text_section(&mut self, offset: u64, ts: &mut TextSection) {
        self.seek(offset);
        self.r.read_exact(ts.name).unwrap();

        self.seek(offset+32);
        let address: u32 = self.read();
        let size: u32 = self.read();

        ts.size = size;
        ts.address = address;
    }

    fn seek(&mut self, pos: u64) {
        self.r.seek(SeekFrom::Start(pos)).unwrap();
    }

    fn read(&mut self) -> u32 {
        return self.r.read_u32::<LittleEndian>().unwrap();
    }
}

fn main() {
    let mut f = File::open("../program").expect("file not found");

    let mut contents: Vec<u8> = vec![];
    f.read_to_end(&mut contents).expect("cannot read");

    let mut exe = MachOBin::new(&contents[..]);

    println!("magic valid: {}", exe.valid_magic());

    exe.process_load_commands();

    // println!("content length: {}", contents.len());
}
