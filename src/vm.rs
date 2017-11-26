use macho::Bin;
use errors::*;

use byteorder::{ByteOrder, LittleEndian, ReadBytesExt};
use hexplay::HexViewBuilder;

#[derive(Debug)]
pub struct VM {
    pub memory: Vec<u32>,
    pub registers: X86Registers,

    // interrupt handlers and syscall use this to signal exit
    pub exit_status: ExitStatus,
}

type ExitStatus = Option<usize>;

fn hexdump(buf: &[u8]) {
    let view = HexViewBuilder::new(&buf).row_width(16).finish();
    println!("{}", view);
}

fn getbyte(mem: &[u32], i: u32) -> u8 {
    println!(
        "word[{} byte={}]: {:x} byte={:x}",
        i as usize / 4,
        i,
        mem[(i as usize) / 4],
        mem[(i as usize) / 4] >> ((i % 4) * 8),
    );
    return ((mem[(i as usize) / 4] >> ((i % 4) * 8)) & 0xff) as u8;
}

fn readu32(buf: &[u8]) -> u32 {
    return LittleEndian::read_u32(buf);
}

impl VM {
    pub fn new(memsize: usize) -> VM {
        if memsize % 4 != 0 {
            panic!("VM memory must by multiples of 4 bytes (32 bit)")
        }
        let mut memory: Vec<u32> = Vec::with_capacity(memsize / 4);
        unsafe {
            memory.set_len(memsize / 4);
        }

        let registers = X86Registers::default();

        return VM {
            memory: memory,
            registers: registers,
            exit_status: None,
        };
    }

    pub fn run(&mut self, bin: &Bin) -> Result<()> {
        let text = bin.text_section()?;
        let text_address = bin.text_address()?;

        // init registers if using LC_UNIXTHREAD
        if let Some(ref unixthread) = bin.load_commands.unixthread {
            self.registers = unixthread.registers.clone();
        }

        if let Some(ref main) = bin.load_commands.main {
            self.registers.eip = main.entry_offset as u32 + text_address;
        }

        self.registers.esp = (self.memory.len() - 1) as u32;

        // load text section into memory
        {
            let text = bin.text_section()?;

            let size = text.size as usize;

            // text's offset in binary
            let offset = text.offset as usize;
            // text's location in memory
            let address = text.address as usize;

            let textdata = &bin.data[offset..offset + size];

            unsafe {
                let ptr = self.memory.as_ptr() as *mut u8;
                let memu8 = ::std::slice::from_raw_parts_mut(ptr, self.memory.len() * 4);
                let textmem = &mut memu8[address..address + size];
                textmem.copy_from_slice(textdata);

                // println!("text: {:?}", &bin.data[offset..offset + size]);
            }
        }

        self.exit_status = None;

        return self.exec();
    }

    // provide a view of the vm memory in bytes
    fn memu8(&mut self) -> &mut [u8] {
        unsafe {
            let ptr = self.memory.as_ptr() as *mut u8;
            let mem = ::std::slice::from_raw_parts_mut(ptr, self.memory.len() * 4);
            return mem;
        }
    }

    fn push(&mut self, val: u32) {
        self.memory[(self.registers.esp / 4) as usize] = val;
        self.registers.esp -= 4;
    }

    fn exec(&mut self) -> Result<()> {
        // unsafe byte view into the vm memory
        let memu8: &[u8];

        unsafe {
            let ptr = self.memory.as_ptr() as *mut u8;
            memu8 = ::std::slice::from_raw_parts_mut(ptr, self.memory.len() * 4);
        }

        loop {
            let i = self.registers.eip as usize;
            let op = memu8[i];

            println!("eip: {:x}", i);
            hexdump(&memu8[i..i + 16]);

            match op {
                // mov imm8 eax
                0xb8 => {
                    self.registers.eax = readu32(&memu8[i + 1..]);
                    self.registers.eip += 5;
                }

                // push imm8
                0x6a => {
                    let val = memu8[i + 1] as u32;
                    self.push(val);
                    self.registers.eip += 2;
                }

                // push reg
                0x50 => {
                    let val = self.registers.eax;
                    self.push(val);
                    self.registers.eip += 1;
                }

                // int
                0xcd => {
                    let val = memu8[i + 1];
                    self.handle_interrupt(val)?;

                    if !self.exit_status.is_none() {
                        return Ok(());
                    }

                    self.registers.eip += 2;
                }

                _ => panic!("invalid instruction"),
            }
        }

        return Ok(());
    }

    // TODO: abstract this into its own "OS Emulator" object
    fn handle_interrupt(&mut self, n: u8) -> Result<()> {
        match n {
            // syscall
            0x80 => {
                // MacOS / BSD convention. eax syscall number, arguments on stack.
                let n = self.registers.eax;
                return self.syscall(n as u8);
            }
            _ => panic!("unknown interrupt"),
        };

        return Ok(());
    }

    fn syscall(&mut self, n: u8) -> Result<()> {
        match n {
            // exit
            0x1 => {
                let status = self.memory[((self.registers.esp + 4) / 4) as usize];
                // TODO convert u8 into int8 (not truncating)
                self.exit_status = Some(status as usize);
            }

            _ => panic!("unknown syscall"),
        };

        return Ok(());
    }
}



#[derive(Debug, Default, Clone)]
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
        return X86Registers {
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
        };
    }

    pub fn init(&mut self, words: &[u32]) {
        self.eax = words[0];
        self.ebx = words[1];
        self.ecx = words[2];
        self.edx = words[3];

        self.edi = words[4];
        self.esi = words[5];
        self.ebp = words[6];
        self.esp = words[7];

        self.ss = words[8];
        self.eflags = words[9];
        self.eip = words[10];
        self.cs = words[11];

        self.ds = words[12];
        self.es = words[13];
        self.fs = words[14];
        self.gs = words[15];
    }
}
