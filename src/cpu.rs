//! CPU module
//!
//! Command reference guide: https://www.nesdev.org/obelisk-6502-guide/reference.html
//! NES uses Little-Endian (8 least significant bits before 8 most significant bits)
//! Example:
//!   - to load from memory cell 0x8000 it would look like this: `a9 00 80`
//!
//! Additional help:
//! - |= is bitwise OR assignment
//! - &= is bitwise AND assignment

use bitflags::bitflags;

use crate::opcode::{self, AddressingMode, Mnemonic, OpCode};

bitflags! {
    /// # Status Register (P) http://wiki.nesdev.com/w/index.php/Status_flags
    ///
    ///  7 6 5 4 3 2 1 0
    ///  N V _ B D I Z C
    ///  | |   | | | | +--- Carry Flag
    ///  | |   | | | +----- Zero Flag
    ///  | |   | | +------- Interrupt Disable
    ///  | |   | +--------- Decimal Mode (not used on NES)
    ///  | |   +----------- Break Command
    ///  | +--------------- Overflow Flag
    ///  +----------------- Negative Flag
    ///
    #[derive(Clone)]
    pub struct CpuFlags: u8 {
        const CARRY             = 0b00000001;
        const ZERO              = 0b00000010;
        const INTERRUPT_DISABLE = 0b00000100;
        const DECIMAL_MODE      = 0b00001000;
        const BREAK             = 0b00010000;
        const BREAK2            = 0b00100000;
        const OVERFLOW          = 0b01000000;
        const NEGATIV           = 0b10000000;
    }
}

// reference: https://chubakbidpaa.com/retro/2020/12/15/6502-stack-copy.html
// stack typically starts at 0x0100 and ends at 0x01FF and lives in the 0th page
const STACK_RESET: u8 = 0xFD; // TODO: changed from 0xFF to 0xFD like guide
const STACK: u16 = 0x0100;
const PROGRAM_START_ADDR: u16 = 0x0600; // TODO: changed from 0x8000 to 0x0600 to work
const PROGRAM_INIT_ADDR: u16 = 0xFFFC;
const MEMORY_LENGTH: usize = 0xFFFF;

#[allow(clippy::upper_case_acronyms)]
pub struct CPU {
    // accumulator
    pub register_a: u8,
    // x register
    pub register_x: u8,
    // y register
    pub register_y: u8,
    // processor status
    pub status: CpuFlags,
    // program counter
    pub program_counter: u16,
    // stack pointer
    pub stack_register: u8,
    // 16 byte memory
    memory: [u8; MEMORY_LENGTH],
}

pub trait Mem {
    fn mem_read(&self, addr: u16) -> u8;

    fn mem_write(&mut self, addr: u16, data: u8);

    /// Reads the 16 byte integer at the given `pos` using Little-Endian methods.
    fn mem_read_u16(&self, pos: u16) -> u16 {
        let lo = self.mem_read(pos);
        let hi = self.mem_read(pos + 1);

        u16::from_le_bytes([lo, hi])
    }

    /// Writes a 16 byte integer at the given `pos` using Little-Endian methods.
    fn mem_write_u16(&mut self, pos: u16, data: u16) {
        let [lo, hi] = data.to_le_bytes();
        self.mem_write(pos, lo);
        self.mem_write(pos + 1, hi);
    }
}

impl Mem for CPU {
    fn mem_read(&self, addr: u16) -> u8 {
        self.memory[addr as usize]
    }

    fn mem_write(&mut self, addr: u16, data: u8) {
        self.memory[addr as usize] = data;
    }
}

trait Stack {
    fn push_to_stack(&mut self, value: u8);

    fn pull_from_stack(&mut self) -> u8;

    fn push_to_stack_u16(&mut self, value: u16) {
        let [lo, hi] = value.to_le_bytes();

        self.push_to_stack(hi);
        self.push_to_stack(lo);
    }

    fn pull_from_stack_u16(&mut self) -> u16 {
        let lo = self.pull_from_stack();
        let hi = self.pull_from_stack();

        u16::from_le_bytes([lo, hi])
    }
}

impl Stack for CPU {
    fn push_to_stack(&mut self, value: u8) {
        self.mem_write(STACK + self.stack_register as u16, value);
        // because 6502 uses a descending stack we need to subtract one from the stack register (or pointer) after the write
        self.stack_register = self.stack_register.wrapping_sub(1);
    }

    fn pull_from_stack(&mut self) -> u8 {
        // because of the descending stack we need to add one to the stack register (or pointer) before the read
        self.stack_register = self.stack_register.wrapping_add(1);
        self.mem_read(STACK + self.stack_register as u16)
    }
}

impl CPU {
    pub fn new() -> Self {
        CPU {
            register_a: 0,
            register_x: 0,
            register_y: 0,
            status: CpuFlags::from_bits_truncate(0b100100), // break and interrupt disable should be set
            program_counter: 0,
            stack_register: STACK_RESET,
            memory: [0; MEMORY_LENGTH],
        }
    }

    /// This function gets called on the RESET INTERRUPT signal.
    /// It resets all registers and sets the program counter to the value in address 0xFFFC.
    pub fn reset(&mut self) {
        self.register_a = 0;
        self.register_x = 0;
        self.register_y = 0;
        self.stack_register = STACK_RESET;
        self.status = CpuFlags::from_bits_truncate(0b100100);

        self.program_counter = self.mem_read_u16(PROGRAM_INIT_ADDR);
    }

    /// Loads the given program into the correct memory slot and write the start of the program code
    /// into address 0xFFFC.
    pub fn load(&mut self, program: Vec<u8>) {
        // copy program into memory starting at address 0x8000
        self.memory[PROGRAM_START_ADDR as usize..(PROGRAM_START_ADDR as usize + program.len())]
            .copy_from_slice(&program[..]);
        // we put the start of the program ROM code address into 0xFFFC so the reset will always be put towards this value
        self.mem_write_u16(PROGRAM_INIT_ADDR, PROGRAM_START_ADDR);
    }

    /// Calls the load function, then resets the cpu state and afterwards executes the loaded program.
    pub fn load_and_run(&mut self, program: Vec<u8>) {
        self.load(program);
        self.reset();
        self.run()
    }

    pub fn run(&mut self) {
        self.run_with_callback(|_| {});
    }

    /// Get the instruction opcode from memory and exectute accordingly.
    pub fn run_with_callback<F>(&mut self, mut callback: F)
    where
        F: FnMut(&mut CPU),
    {
        let opcodes = &(*opcode::OPCODES_MAP);

        loop {
            callback(self);

            let code = self.mem_read(self.program_counter);
            self.program_counter += 1;
            let program_counter_state = self.program_counter;

            let opcode = opcodes
                .get(&code)
                .unwrap_or_else(|| panic!("OpCode {:x} is not recognized", code));

            match opcode.mnemonic {
                Mnemonic::ADC => self.adc(&opcode.addressing_mode),
                Mnemonic::AND => self.and(&opcode.addressing_mode),
                Mnemonic::ASL => self.asl(&opcode.addressing_mode),
                Mnemonic::BIT => self.bit(&opcode.addressing_mode),
                Mnemonic::DEC => self.dec(&opcode.addressing_mode),
                Mnemonic::DEX => self.dex(),
                Mnemonic::DEY => self.dey(),
                Mnemonic::EOR => self.eor(&opcode.addressing_mode),
                Mnemonic::INC => self.inc(&opcode.addressing_mode),
                Mnemonic::INX => self.inx(),
                Mnemonic::INY => self.iny(),
                Mnemonic::LDA => self.lda(&opcode.addressing_mode),
                Mnemonic::LDX => self.ldx(&opcode.addressing_mode),
                Mnemonic::LDY => self.ldy(&opcode.addressing_mode),
                Mnemonic::LSR => self.lsr(&opcode.addressing_mode),
                Mnemonic::NOP => (), // noop
                Mnemonic::ORA => self.ora(&opcode.addressing_mode),
                Mnemonic::ROL => self.rol(&opcode.addressing_mode),
                Mnemonic::ROR => self.ror(&opcode.addressing_mode),
                Mnemonic::SBC => self.sbc(&opcode.addressing_mode),
                Mnemonic::STA => self.sta(&opcode.addressing_mode),
                Mnemonic::STX => self.stx(&opcode.addressing_mode),
                Mnemonic::STY => self.sty(&opcode.addressing_mode),
                Mnemonic::TAX => self.tax(),
                Mnemonic::TAY => self.tay(),
                Mnemonic::TXA => self.txa(),
                Mnemonic::TYA => self.tya(),
                Mnemonic::TSX => self.tsx(),
                Mnemonic::TXS => self.txs(),

                // Stack
                Mnemonic::PHA => self.pha(),
                Mnemonic::PHP => self.php(),
                Mnemonic::PLA => self.pla(),
                Mnemonic::PLP => self.plp(),

                // Subroutine
                Mnemonic::JSR => self.jsr(opcode),
                Mnemonic::RTI => self.rti(),
                Mnemonic::RTS => self.rts(),

                // Compare
                Mnemonic::CMP => self.compare(&opcode.addressing_mode, self.register_a),
                Mnemonic::CPX => self.compare(&opcode.addressing_mode, self.register_x),
                Mnemonic::CPY => self.compare(&opcode.addressing_mode, self.register_y),

                // Branching
                Mnemonic::BRK => return,
                Mnemonic::JMP => self.jump(&opcode.addressing_mode),
                Mnemonic::BPL => self.branch(!self.status.contains(CpuFlags::NEGATIV)),
                Mnemonic::BMI => self.branch(self.status.contains(CpuFlags::NEGATIV)),
                Mnemonic::BVC => self.branch(!self.status.contains(CpuFlags::OVERFLOW)),
                Mnemonic::BVS => self.branch(self.status.contains(CpuFlags::OVERFLOW)),
                Mnemonic::BCC => self.branch(!self.status.contains(CpuFlags::CARRY)),
                Mnemonic::BCS => self.branch(self.status.contains(CpuFlags::CARRY)),
                Mnemonic::BNE => self.branch(!self.status.contains(CpuFlags::ZERO)),
                Mnemonic::BEQ => self.branch(self.status.contains(CpuFlags::ZERO)),

                // Sets
                Mnemonic::SEC => self.status.insert(CpuFlags::CARRY),
                Mnemonic::SEI => self.status.insert(CpuFlags::INTERRUPT_DISABLE),
                Mnemonic::SED => self.status.insert(CpuFlags::DECIMAL_MODE),

                // Clears
                Mnemonic::CLC => self.status.remove(CpuFlags::CARRY),
                Mnemonic::CLI => self.status.remove(CpuFlags::INTERRUPT_DISABLE),
                Mnemonic::CLV => self.status.remove(CpuFlags::OVERFLOW),
                Mnemonic::CLD => self.status.remove(CpuFlags::DECIMAL_MODE), // we ignore decimal mode but I just added it anyway
            }

            // prevent updating of program_counter after branches/jumps
            if program_counter_state == self.program_counter {
                self.program_counter += (opcode.bytes - 1) as u16;
            }
        }
    }

    /// Returns address for a corresponding [`AddressingMode`].
    /// Address is derived from the [`progam_counter`](CPU) of CPU.
    fn get_operand_address(&self, mode: &AddressingMode) -> u16 {
        match mode {
            AddressingMode::Immediate => self.program_counter,
            AddressingMode::ZeroPage => self.mem_read(self.program_counter) as u16,
            AddressingMode::ZeroPageX => {
                let pos = self.mem_read(self.program_counter);
                pos.wrapping_add(self.register_x) as u16
            }
            AddressingMode::ZeroPageY => {
                let pos = self.mem_read(self.program_counter);
                pos.wrapping_add(self.register_y) as u16
            }
            AddressingMode::Absolute => self.mem_read_u16(self.program_counter),
            AddressingMode::AbsoluteX => {
                let base = self.mem_read_u16(self.program_counter);
                base.wrapping_add(self.register_x as u16)
            }
            AddressingMode::AbsoluteY => {
                let base = self.mem_read_u16(self.program_counter);
                base.wrapping_add(self.register_y as u16)
            }
            // JMP is the only instruction to use Indirect AddressingMode in the 6502
            AddressingMode::Indirect => {
                let base = self.mem_read_u16(self.program_counter);
                // http://www.6502.org/tutorials/6502opcodes.html#JMP => an indirect jump must never use a vector beginning on the last byte of a page
                // Note: 16 bit address space consists of 256 pages of 1 byte memory locations
                // this means we are on the last byte of a page (0x00FF masking means last byte of this page)
                if base & 0x00FF == 0x00FF {
                    let lo = self.mem_read(base);
                    let hi = self.mem_read(base & 0xFF00);
                    u16::from_le_bytes([lo, hi])
                } else {
                    self.mem_read_u16(base)
                }
            }
            AddressingMode::IndirectX => {
                let base = self.mem_read(self.program_counter);
                let ptr = base.wrapping_add(self.register_x);
                let lo = self.mem_read(ptr as u16);
                let hi = self.mem_read(ptr.wrapping_add(1) as u16);
                u16::from_le_bytes([lo, hi])
            }
            AddressingMode::IndirectY => {
                let base = self.mem_read(self.program_counter);
                let lo = self.mem_read(base as u16);
                let hi = self.mem_read(base.wrapping_add(1) as u16);
                let deref_base = u16::from_le_bytes([lo, hi]);
                deref_base.wrapping_add(self.register_y as u16)
            }
            // these modes are handled differently, we don't want this branch to be called so we panic.
            AddressingMode::Accumulator | AddressingMode::Relative | AddressingMode::Implied => panic!(
                "opcodes using mode {:?} are not supported within this function and should be handled separately",
                mode
            )
        }
    }

    fn get_accumulator_or_memory(&self, mode: &AddressingMode) -> (u8, Option<u16>) {
        if let AddressingMode::Accumulator = mode {
            (self.register_a, None)
        } else {
            let addr = self.get_operand_address(mode);
            (self.mem_read(addr), Some(addr))
        }
    }

    fn set_register_a(&mut self, value: u8) {
        self.register_a = value;
        self.set_zero_flag_with(self.register_a);
        self.set_negative_flag_with(self.register_a);
    }

    fn set_register_x(&mut self, value: u8) {
        self.register_x = value;
        self.set_zero_flag_with(self.register_x);
        self.set_negative_flag_with(self.register_x);
    }

    fn set_register_y(&mut self, value: u8) {
        self.register_y = value;
        self.set_zero_flag_with(self.register_y);
        self.set_negative_flag_with(self.register_y);
    }

    fn set_zero_flag_with(&mut self, register_value: u8) {
        // this sets/unsets the ZERO FLAG while keeping all other flags
        if register_value == 0 {
            self.status.insert(CpuFlags::ZERO);
        } else {
            self.status.remove(CpuFlags::ZERO);
        }
    }

    fn set_negative_flag_with(&mut self, register_value: u8) {
        // this sets/unsets the NEGATIVE FLAG if the final bit is set
        if register_value & 0b1000_0000 != 0 {
            self.status.insert(CpuFlags::NEGATIV);
        } else {
            self.status.remove(CpuFlags::NEGATIV);
        }
    }

    fn add_with_carry(&mut self, value: u8) {
        let sum = self.register_a as u16
            + value as u16
            + if self.status.contains(CpuFlags::CARRY) {
                1
            } else {
                0
            };

        if sum > u8::MAX as u16 {
            self.status.insert(CpuFlags::CARRY);
        } else {
            self.status.remove(CpuFlags::CARRY);
        }

        // this is the most complex part of this function
        // the overflow flag in 6502 is explained properly here: http://www.righto.com/2012/12/the-6502-overflow-flag-explained.html
        // why this formula was chosen is also explained in that article but I'll copy the important line below
        // **Overflow occurs if (M ^ result) & (N ^ result) & 0x80 is nonzero** (^ being XOR)
        if (self.register_a ^ sum as u8) & (value ^ sum as u8) & 0x80 != 0 {
            self.status.insert(CpuFlags::OVERFLOW);
        } else {
            self.status.remove(CpuFlags::OVERFLOW);
        }

        self.set_register_a(sum as u8);
    }

    /// Add with Carry and save into accumulator
    fn adc(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);

        self.add_with_carry(value);
    }

    /// Bitwise AND of value inside accumulator and value stored at address (calculated via `mode`).
    fn and(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);

        self.set_register_a(value & self.register_a);
    }

    /// Arithmetic Shift Left, shifts bits to the left
    fn asl(&mut self, mode: &AddressingMode) {
        let (mut value, addr) = self.get_accumulator_or_memory(mode);

        if value >> 7 == 1 {
            self.status.insert(CpuFlags::CARRY);
        } else {
            self.status.remove(CpuFlags::CARRY);
        }

        value <<= 1;

        match addr {
            None => self.set_register_a(value),
            Some(addr_val) => {
                self.mem_write(addr_val, value);
                self.set_zero_flag_with(value);
                self.set_negative_flag_with(value);
            }
        }
    }

    /// Bit Test Operation. Take value in a specific memory cell and bitwise AND it with the accumulator.
    /// Depending on the result set the ZERO, NEGATIV and OVERFLOW flags.
    fn bit(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);

        let result = value & self.register_a;

        // TODO: maybe add a set function for other flags as well?
        self.set_zero_flag_with(result);
        self.set_negative_flag_with(result);
        self.status
            .set(CpuFlags::OVERFLOW, result & 0b0100_0000 > 0);
    }

    /// Decrements the value stored in memory found with `mode`
    fn dec(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let mut value = self.mem_read(addr);

        value = value.wrapping_sub(1);
        self.mem_write(addr, value);

        self.set_zero_flag_with(value);
        self.set_negative_flag_with(value);
    }

    /// Decrements the value store in register x
    fn dex(&mut self) {
        self.register_x = self.register_x.wrapping_sub(1);

        self.set_zero_flag_with(self.register_x);
        self.set_negative_flag_with(self.register_x);
    }

    /// Decrements the value store in register y
    fn dey(&mut self) {
        self.register_y = self.register_y.wrapping_sub(1);

        self.set_zero_flag_with(self.register_y);
        self.set_negative_flag_with(self.register_y);
    }

    /// Bitwise XOR of value inside accumulator and value stored at address (calculated via `mode`).
    fn eor(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);

        self.set_register_a(value ^ self.register_a);
    }

    /// Increment value at memory address (calculated via `mode`).
    fn inc(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let mut value = self.mem_read(addr);

        value = value.wrapping_add(1);
        self.mem_write(addr, value);

        self.set_zero_flag_with(value);
        self.set_negative_flag_with(value);
    }

    /// Increment value in register x.
    fn inx(&mut self) {
        self.register_x = self.register_x.wrapping_add(1);

        self.set_zero_flag_with(self.register_x);
        self.set_negative_flag_with(self.register_x);
    }

    /// Increment value in register y.
    fn iny(&mut self) {
        self.register_y = self.register_y.wrapping_add(1);

        self.set_zero_flag_with(self.register_y);
        self.set_negative_flag_with(self.register_y);
    }

    /// Load value stored at address (calculated via `mode`) into accumulator.
    fn lda(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);

        self.set_register_a(value);
    }

    /// Load value stored at address (calculated via `mode`) into register x.
    fn ldx(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);

        self.set_register_x(value);
    }

    /// Load value stored at address (calculated via `mode`) into register y.
    fn ldy(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);

        self.set_register_y(value);
    }

    /// Logical Shift Right, shifts bits to the right
    fn lsr(&mut self, mode: &AddressingMode) {
        let (mut value, addr) = self.get_accumulator_or_memory(mode);

        if value & 0b0000_0001 == 1 {
            self.status.insert(CpuFlags::CARRY);
        } else {
            self.status.remove(CpuFlags::CARRY);
        }

        value >>= 1;

        match addr {
            None => self.set_register_a(value),
            Some(addr_val) => {
                self.mem_write(addr_val, value);
                self.set_zero_flag_with(value);
                self.set_negative_flag_with(value);
            }
        }
    }

    /// Bitwise inclusive OR of value inside accumulator and value stored at address (calculated via `mode`).
    fn ora(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);

        self.set_register_a(value | self.register_a);
    }

    /// Rotate Left, shifts bits to the left and fills bit 0 with carry flag.
    fn rol(&mut self, mode: &AddressingMode) {
        let (mut value, addr) = self.get_accumulator_or_memory(mode);
        let saved_carry = self.status.contains(CpuFlags::CARRY);

        if value & 0b0000_0001 == 1 {
            self.status.insert(CpuFlags::CARRY);
        } else {
            self.status.remove(CpuFlags::CARRY);
        }

        value <<= 1;
        if saved_carry {
            value += 1; // another way would be `value |= 1`
        }

        match addr {
            None => self.set_register_a(value),
            Some(addr_val) => {
                self.mem_write(addr_val, value);
                self.set_zero_flag_with(value);
                self.set_negative_flag_with(value);
            }
        }
    }

    /// Rotate Right, shifts bits to the right and fills bit 7 with carry flag.
    fn ror(&mut self, mode: &AddressingMode) {
        let (mut value, addr) = self.get_accumulator_or_memory(mode);
        let saved_carry = self.status.contains(CpuFlags::CARRY);

        if value & 0b0000_0001 == 1 {
            self.status.insert(CpuFlags::CARRY);
        } else {
            self.status.remove(CpuFlags::CARRY);
        }

        value >>= 1;
        if saved_carry {
            value |= 0b1000_0000; // could also use +128 (or 2^7) which represents the 7th bit
        }

        match addr {
            None => self.set_register_a(value),
            Some(addr_val) => {
                self.mem_write(addr_val, value);
                self.set_zero_flag_with(value);
                self.set_negative_flag_with(value);
            }
        }
    }

    /// Subtract with Carry and save into accumulator
    fn sbc(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);

        // 6502 uses the 1's complement, 2's complement would be to add 1
        // I found this to be helpful: https://retro64.altervista.org/blog/an-introduction-to-6502-math-addiction-subtraction-and-more/
        // but I have to say I am still skeptical, the CARRY flag acts as a reverse BORROW flag here
        self.add_with_carry(!value);
    }

    /// Stores content of accumulator into memory.
    fn sta(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        self.mem_write(addr, self.register_a);
    }

    /// Stores content of register x into memory.
    fn stx(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        self.mem_write(addr, self.register_x);
    }

    /// Stores content of register y into memory.
    fn sty(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        self.mem_write(addr, self.register_y);
    }

    /// Transfer value in accumulator to register x.
    fn tax(&mut self) {
        self.set_register_x(self.register_a);
    }

    /// Transfer value in accumulator to register y.
    fn tay(&mut self) {
        self.set_register_y(self.register_a);
    }

    /// Transfer value in register x to accumulator.
    fn txa(&mut self) {
        self.set_register_a(self.register_x);
    }

    /// Transfer value in register y to accumulator.
    fn tya(&mut self) {
        self.set_register_a(self.register_y);
    }

    /// Transfer value in stack register (or pointer) to register x.
    fn tsx(&mut self) {
        self.set_register_x(self.stack_register);
    }

    /// Transfer value in register x to stack register (or pointer).
    fn txs(&mut self) {
        self.stack_register = self.register_x;
    }

    /// Pushes copy of the accumulator on the stack.
    fn pha(&mut self) {
        self.push_to_stack(self.register_a);
    }

    /// Pushes copy of the status flags to the stack.
    /// Sets the BREAK and BREAK2 flags on the copy before push.
    fn php(&mut self) {
        // reference: https://www.nesdev.org/wiki/Status_flags#The_B_flag
        // the PHP instruction always pushes BREAK2 and BREAK flags set to the stack
        // Sidenote: BREAK2 is always pushed as 1.
        let mut flags = self.status.clone();
        flags.insert(CpuFlags::BREAK);
        flags.insert(CpuFlags::BREAK2);
        self.push_to_stack(flags.bits());
    }

    /// Pulls a value from the stack and sets it into the accumulator.
    fn pla(&mut self) {
        let value = self.pull_from_stack();
        self.set_register_a(value);
    }

    /// Pulls a value from the stack and sets it into processor status.
    fn plp(&mut self) {
        // we disregard the bits 5 and 4 (BREAK2 and BREAK) when pulling this value form the stack
        let value = self.pull_from_stack();
        self.status = CpuFlags::from_bits_truncate(value & 0b1100_1111);
        // below 2 lines shouldn't be necessary but whatever I'll just add it
        self.status.remove(CpuFlags::BREAK);
        self.status.remove(CpuFlags::BREAK2);
    }

    /// Push address - 1 of return point onto stack and set program counter to target address.
    fn jsr(&mut self, op_code: &OpCode) {
        let addr = self.get_operand_address(&op_code.addressing_mode);
        self.push_to_stack_u16(self.program_counter + (op_code.bytes as u16 - 1) - 1);
        self.program_counter = addr;
    }

    /// Pulls processor flags and program counter from stack.
    /// Called at the end of interrupt processing subroutine.
    fn rti(&mut self) {
        let saved_flags = self.pull_from_stack();

        self.status = CpuFlags::from_bits_truncate(saved_flags & 0b1100_1111);
        self.status.remove(CpuFlags::BREAK);
        self.status.remove(CpuFlags::BREAK2);

        self.program_counter = self.pull_from_stack_u16();
    }

    /// Pulls program counter from stack to return to the calling routine.
    fn rts(&mut self) {
        // reference https://www.nesdev.org/obelisk-6502-guide/reference.html#RTS says pc minus 1
        // but the guide adds 1 => TODO: is this correct?
        self.program_counter = self.pull_from_stack_u16() + 1;
    }

    /// General Branching entry
    fn branch(&mut self, flag: bool) {
        if flag {
            // we are casting this u8 to i8 since branching uses relative addressing mode
            // relative addressing mode interprets the value in memory for branches
            // as a signed 8 bit relative offset which will be added to the program_counter
            // reference: https://www.nesdev.org/obelisk-6502-guide/addressing.html#REL
            let jump = self.mem_read(self.program_counter) as i8;
            self.program_counter = self
                .program_counter
                .wrapping_add(1) // we add one because branching instructions are 2 byte long
                .wrapping_add(jump as u16);
        }
    }

    /// Jumps to the memory address value calculated with `mode`.
    /// A bit of a different one, because it has the INDIRECT addressing mode.
    /// This mode means that the address to jump to is stored in the address that is supplied as
    /// parameter. Not only that, but it also cannot supply an address pointing to the last
    /// byte of a page, so it requires different logic.
    /// Check the comment in [`Self::get_operand_address()`] for more information.
    fn jump(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        self.program_counter = addr;
    }

    /// Compares the value of the memory cell found with `mode` to the `compare_to` value.
    /// The comparison inside the actual 6502 processor happens with a subtraction.
    /// Because we are in a higher level than a processor and want to emulate said processor
    /// we can skip the subtraction until we need to set the NEGATIVE bit flag.
    /// More information: http://www.6502.org/tutorials/compare_beyond.html#2.1
    fn compare(&mut self, mode: &AddressingMode, compare_to: u8) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);

        self.status.set(CpuFlags::CARRY, compare_to >= value);
        // self.status.set(CpuFlags::ZERO, compare_to == value);
        self.set_zero_flag_with(compare_to.wrapping_sub(value));
        self.set_negative_flag_with(compare_to.wrapping_sub(value));
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_0xa9_lda_immediate_load_data() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x05, 0x00]);

        assert!(cpu.status.bits() & 0b0000_0010 == 0b00);
        assert!(cpu.status.bits() & 0b1000_0000 == 0);
    }

    #[test]
    fn test_0xa9_lda_zero_flag() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x00, 0x00]);

        assert!(cpu.status.bits() & 0b0000_0010 == 0b10);
    }

    #[test]
    fn test_lda_from_memory() {
        let mut cpu = CPU::new();
        let data = 0x55;
        cpu.mem_write(0x10, data);

        cpu.load_and_run(vec![0xa5, 0x10, 0x00]);

        assert_eq!(cpu.register_a, data);
    }

    #[test]
    fn test_0xaa_tax_transfer_data() {
        let mut cpu = CPU::new();
        let test_value = 5;
        cpu.load_and_run(vec![0xa9, test_value, 0xaa, 0x00]);

        assert!(cpu.register_x == test_value);
        assert!(cpu.status.bits() & 0b0000_0010 == 0b00);
        assert!(cpu.status.bits() & 0b1000_0000 == 0);
    }

    #[test]
    fn test_5_ops_working_together() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0xc0, 0xaa, 0xe8, 0x00]);

        assert_eq!(cpu.register_x, 0xc1);
    }

    #[test]
    fn test_inx_overflow() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0xff, 0xaa, 0xe8, 0xe8, 0x00]);

        assert_eq!(cpu.register_x, 1);
    }

    #[test]
    fn test_sta_working() {
        let mut cpu = CPU::new();
        let data = 10;
        let addr = 0x0f;

        cpu.load_and_run(vec![0xa9, data, 0x85, addr, 0x00]);

        assert_eq!(cpu.memory[addr as usize], data);
    }
}
