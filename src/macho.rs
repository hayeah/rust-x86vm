use errors::*;
use vm::X86Registers;

#[derive(Debug)]
pub struct Bin {
    pub data: Vec<u8>,
    pub header: Header,
    pub load_commands: LoadCommands,
}

impl Bin {
    pub fn text(&self) -> Result<Vec<u8>> {
        let mut it = self.load_commands.segments.iter();
        let textseg = it.find(|seg| seg.name == "__TEXT").ok_or(
            ErrorKind::ErrNoTextSegment,
        )?;

        let mut it = textseg.section_headers.iter();
        let textsec = it.find(|h| h.section_name == "__text").ok_or(
            ErrorKind::ErrNoTextSegment,
        )?;

        let start = textsec.offset as usize;
        let size = textsec.size as usize;
        let text = &self.data[start..start + size];
        return Ok(text.to_vec());
    }

    pub fn init_registers(&self) -> X86Registers {
        return self.load_commands.unixthread.registers.clone();
    }
}

#[derive(Debug)]
pub struct Header {
    pub magic: u32,
    pub cpu_type: u32,
    pub cpu_subtype: u32,
    pub file_type: u32,
    pub load_commands_count: u32,
    pub load_commands_size: u32,
    pub flags: u32,
}

#[derive(Debug)]
pub struct SectionHeader {
    // 0..16
    pub section_name: String,
    // 16..32
    pub segment_name: String,
    // 32
    pub address: u32,
    pub size: u32,
    pub offset: u32,
    pub alignment: u32,
    // relocations_ffset: u32,
}

#[derive(Debug)]
pub struct LoadCommands {
    pub segments: Vec<Segment>,

    // assumed to have just one
    pub unixthread: UnixThread,

    pub unsupported: Vec<UnsupportedLoadCommand>,
}

#[derive(Debug)]
pub struct Segment {
    pub name: String, // 2 .. 5
    pub vm_address: u32, // 6
    pub vm_sizes: u32,
    pub file_offset: u32,
    pub file_size: u32,
    pub max_vm_protection: u32,
    pub initial_vm_protection: u32,
    pub number_of_sections: u32,
    pub flags: u32,

    pub section_headers: Vec<SectionHeader>,
}

#[derive(Debug)]
pub struct UnixThread {
    pub flavor: u32,
    pub count: u32,
    pub registers: X86Registers,
}

#[derive(Debug)]
pub struct UnsupportedLoadCommand {
    pub cmd: u32,
    pub size: usize,
}
