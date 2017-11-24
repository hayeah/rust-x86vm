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
    bin:  [u8],
}

struct LoadCommandsIterator {
    data: [u8],
}

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
