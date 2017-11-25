
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
pub struct X86Registers {
    pub eax: u32,
    pub ebx: u32,
    pub ecx: u32,
    pub edx: u32,

    pub edi: u32,
    pub esi: u32,
    pub ebp: u32,
    pub esp: u32,

    pub ss: u32,
    pub eflags: u32,
    pub eip: u32,
    pub cs: u32,

    pub ds: u32,
    pub es: u32,
    pub fs: u32,
    pub gs: u32,
}

impl X86Registers {
    pub fn from(words: &[u32]) -> X86Registers {
        return X86Registers{
            eax: words[0],
            ebx: words[1],
            ecx: words[2],
            edx: words[3],

            edi: words[4],
            esi: words[5],
            ebp: words[6],
            esp: words[7],

            ss: words[8],
            eflags: words[9],
            eip: words[10],
            cs: words[11],

            ds: words[12],
            es: words[13],
            fs: words[14],
            gs: words[15],
        }
    }
}

#[derive(Debug)]
pub enum LoadCommand {
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

    UnixThread {
        flavor: u32,
        count: u32,

        registers: X86Registers,
    },

    // just put the size/offset here
    Unsupported {
        cmd: u32,
        size: usize,

        // data: Vec<u8>,
    },
}

