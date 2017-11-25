use std;
use byteorder::{LittleEndian, ReadBytesExt};
use std::io::Cursor;

pub struct MachOParser {
    data: Vec<u8>,
}

use macho::*;
use errors::*;

const HEADERSIZE: usize = 0x1c;
const LC_SEGMENT_HEADER_SIZE: usize = 56;
const LC_SEGMENT_SECTION_HEADER_SIZE: usize = 68;
const LC_UNIXTHREAD_SIZE: usize = 80;

impl MachOParser {
    pub fn new(data: Vec<u8>) -> MachOParser {
        MachOParser { data: data }
    }

    pub fn parse_header(&self) -> Result<Header> {
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

    pub fn parse_load_commands(&self, header: &Header) -> Result<Vec<LoadCommand>> {
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

            let segdata = &data[pos..pos + size];

            let lc = match cmd {
                1 => self.parse_lc_segment(segdata),
                5 => self.parse_lc_unixthread(segdata),
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

    fn parse_lc_unixthread(&self, data: &[u8]) -> Result<LoadCommand> {
        let mut words = [0 as u32; LC_UNIXTHREAD_SIZE / 4];
        let mut r = Cursor::new(data);
        r.read_u32_into::<LittleEndian>(&mut words).chain_err(
            || "LC_UNIXTHREAD read fail",
        )?;

        let registers = X86Registers::from(&words[4..4 + 16]);

        return Ok(LoadCommand::UnixThread {
            flavor: words[2],
            count: words[3],
            registers: registers,
        });
    }

    fn parse_lc_segment(&self, data: &[u8]) -> Result<LoadCommand> {
        let mut r = Cursor::new(data);

        let mut buf = [0 as u32; LC_SEGMENT_HEADER_SIZE / 4];
        r.read_u32_into::<LittleEndian>(&mut buf).chain_err(
            || "LC_SEGMENT read failed",
        )?;

        // read 16 bytes as segment name from offset 8
        let name = readname(&data[8..8 + 16]);

        let number_of_sections = buf[12];

        let mut section_headers: Vec<SectionHeader> =
            Vec::with_capacity(number_of_sections as usize);
        let sections_data = &data[LC_SEGMENT_HEADER_SIZE..];
        let mut section_pos = 0;
        for _ in 0..number_of_sections {
            let sh = parse_section_header(
                &sections_data[section_pos..section_pos + LC_SEGMENT_SECTION_HEADER_SIZE],
            ).chain_err(|| "section header parse failed")?;

            section_headers.push(sh);

            section_pos += LC_SEGMENT_SECTION_HEADER_SIZE;
        }

        return Ok(LoadCommand::Segment {
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

fn parse_section_header(data: &[u8]) -> Result<SectionHeader> {
    let section_name = readname(&data[0..16]);
    let segment_name = readname(&data[0..16]);

    let mut words = [0 as u32; LC_SEGMENT_SECTION_HEADER_SIZE / 4];

    let mut r = Cursor::new(data);
    r.read_u32_into::<LittleEndian>(&mut words).chain_err(
        || "section header read fail",
    )?;

    return Ok(SectionHeader {
        section_name: section_name,
        segment_name: segment_name,
        address: words[8],
        size: words[9],
        offset: words[10],
        alignment: words[11],
    });
}

fn readname(data: &[u8]) -> String {
    unsafe {
        let s = std::str::from_utf8_unchecked(data).trim_matches('\0');
        return String::from(s);
    }
}
