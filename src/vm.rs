use macho::Bin;
use errors::*;

use byteorder::{LittleEndian, ReadBytesExt};

#[derive(Debug)]
pub struct VM {
    pub memory: Vec<u8>,
    pub registers: X86Registers,

    // interrupt handlers and syscall use this to signal exit
    pub exit_status: ExitStatus,
}

type ExitStatus = Option<usize>;

impl VM {
    pub fn new(memsize: usize) -> VM {
        let mut memory: Vec<u8> = Vec::with_capacity(memsize);
        unsafe {
            memory.set_len(memsize);
        }

        let registers = X86Registers::default();

        return VM {
            memory: memory,
            registers: registers,
            exit_status: None,
        };
    }

    pub fn run(&mut self, bin: &Bin) -> Result<()> {
        let registers = bin.init_registers();

        // init registers
        self.registers = registers;
        self.registers.esp = (self.memory.len() - 1) as u32;

        // load text section into memory
        {
            let text = bin.text_section()?;

            let size = text.size as usize;

            // text's offset in binary
            let offset = text.offset as usize;
            // text's location in memory
            let address = text.address as usize;

            let textmem = &mut self.memory[address..address + size];
            let textdata = &bin.data[offset..offset + size];

            textmem.copy_from_slice(textdata);
        }

        self.exit_status = None;

        return self.exec();
    }

    fn exec(&mut self) -> Result<()> {
        // let regs = &mut self.registers;
        // let mem = &mut self.memory;
        // let eip = &mut regs.eip;

        println!("bin: {:?}", &self.memory[0..10]);

        loop {
            let i = self.registers.eip as usize;
            let op = self.memory[i];

            println!("eip: {:x}", i);
            println!("text: {:?}", &self.memory[i..i + 14]);

            println!("trace op: {:x}", op);

            match op {
                // mov imm8 eax
                0xb8 => {
                    let i2 = i + 5;
                    let mut valbuf = &self.memory[i + 1..i2];
                    let val = valbuf.read_u32::<LittleEndian>().unwrap();
                    self.registers.eax = val;
                    self.registers.eip = i2 as u32;
                }

                // push imm8
                0x6a => {
                    let i2 = i + 2;
                    let val = self.memory[i + 1];
                    self.memory[self.registers.esp as usize] = val;
                    self.registers.esp -= 1;
                    self.registers.eip = i2 as u32;
                }

                // int
                0xcd => {
                    let i2 = i + 2;
                    let val = self.memory[i + 1];
                    self.handle_interrupt(val)?;

                    if !self.exit_status.is_none() {
                        return Ok(());
                    }

                    self.registers.eip = i2 as u32;
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
                let status = self.memory[(self.registers.esp + 1) as usize];
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
