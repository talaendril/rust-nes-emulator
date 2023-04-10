use std::collections::HashMap;

use lazy_static::lazy_static;
use strum::Display;

#[allow(clippy::upper_case_acronyms, non_camel_case_types)]
#[derive(Debug, Display)]
pub enum Mnemonic {
    ADC,
    AND,
    ASL,
    BCC,
    BCS,
    BEQ,
    BIT,
    BMI,
    BNE,
    BPL,
    BRK,
    BVC,
    BVS,
    CLC,
    CLD,
    CLI,
    CLV,
    CMP,
    CPX,
    CPY,
    DEC,
    DEX,
    DEY,
    EOR,
    INC,
    INX,
    INY,
    JMP,
    JSR,
    LDA,
    LDX,
    LDY,
    LSR,
    NOP,
    ORA,
    PHA,
    PHP,
    PLA,
    PLP,
    ROL,
    ROR,
    RTI,
    RTS,
    SBC,
    SEC,
    SED,
    SEI,
    STA,
    STX,
    STY,
    TAX,
    TAY,
    TSX,
    TXA,
    TXS,
    TYA,

    // 24 unofficial opcodes taken from https://www.nesdev.org/undocumented_opcodes.txt
    // more references:
    // https://www.nesdev.org/wiki/CPU_unofficial_opcodes
    // https://www.nesdev.org/wiki/Programming_with_unofficial_opcodes
    // http://www.oxyron.de/html/opcodes02.html (anywhere H is used means high byte of specified address)
    // https://archive.nes.science/nesdev-forums/f3/t14063.xhtml
    // the actual implementation of these codes is collected from all those sources
    #[strum(serialize = "*AAC")]
    AAC_Unofficial, // also called ANC
    #[strum(serialize = "*AAX")]
    AAX_Unofficial, // also called SAX or AXS
    #[strum(serialize = "*ARR")]
    ARR_Unofficial,
    #[strum(serialize = "*ASR")]
    ASR_Unofficial, // also called ALR
    #[strum(serialize = "*ATX")]
    ATX_Unofficial, // also called LXA or OAL
    #[strum(serialize = "*AXA")]
    AXA_Unofficial, // also called SHA or AHX
    #[strum(serialize = "*AXS")]
    AXS_Unofficial, // also called SBX or SAX
    #[strum(serialize = "*DCP")]
    DCP_Unofficial, // also called DCM
    #[strum(serialize = "*DOP")]
    DOP_Unofficial, // also called NOP or SKB
    #[strum(serialize = "*ISC")]
    ISC_Unofficial, // also called ISB or INS
    #[strum(serialize = "*KIL")]
    KIL_Unofficial, // also called JAM or HLT
    #[strum(serialize = "*LAR")]
    LAR_Unofficial, // also called LAE or LAS
    #[strum(serialize = "*LAX")]
    LAX_Unofficial,
    #[strum(serialize = "*NOP")]
    NOP_Unofficial,
    #[strum(serialize = "*RLA")]
    RLA_Unofficial,
    #[strum(serialize = "*RRA")]
    RRA_Unofficial,
    #[strum(serialize = "*SBC")]
    SBC_Unofficial,
    #[strum(serialize = "*SLO")]
    SLO_Unofficial, // also called ASO
    #[strum(serialize = "*SRE")]
    SRE_Unofficial, // also called LSE
    #[strum(serialize = "*SXA")]
    SXA_Unofficial, // also called SHX or XAS
    #[strum(serialize = "*SYA")]
    SYA_Unofficial, // also called SHY or SAY
    #[strum(serialize = "*TOP")]
    TOP_Unofficial, // also called NOP or SKW
    #[strum(serialize = "*XAA")]
    XAA_Unofficial, // also called ANE
    #[strum(serialize = "*XAS")]
    XAS_Unofficial, // also called SHS or TAS
}

#[derive(Debug)]
pub enum AddressingMode {
    Immediate,
    ZeroPage,
    ZeroPageX,
    ZeroPageY,
    Absolute,
    AbsoluteX,
    AbsoluteY,
    Indirect, // only used by JMP
    IndirectX,
    IndirectY,
    Accumulator,
    Relative,
    Implied,
}

#[derive(Debug)]
pub struct OpCode {
    pub code: u8,
    pub mnemonic: Mnemonic,
    pub bytes: u8,
    pub cycles: u8,
    pub addressing_mode: AddressingMode,
}

impl OpCode {
    fn new(
        code: u8,
        mnemonic: Mnemonic,
        bytes: u8,
        cycles: u8,
        addressing_mode: AddressingMode,
    ) -> OpCode {
        OpCode {
            code,
            mnemonic,
            bytes,
            cycles,
            addressing_mode,
        }
    }
}

lazy_static! {
    // pattern matching is greedy and thus consumes values when they are matched
    // but using ref those values are borrowed rather than moved
    pub static ref CPU_OPS_CODES: Vec<OpCode> = vec![
        // HexCode, Opcode, Bytes, Cycles, Addressing Mode
        OpCode::new(0x69, Mnemonic::ADC, 2, 2, AddressingMode::Immediate),
        OpCode::new(0x65, Mnemonic::ADC, 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0x75, Mnemonic::ADC, 2, 4, AddressingMode::ZeroPageX),
        OpCode::new(0x6d, Mnemonic::ADC, 3, 4, AddressingMode::Absolute),
        OpCode::new(0x7d, Mnemonic::ADC, 3, 4 /*+1 if page crossed*/, AddressingMode::AbsoluteX),
        OpCode::new(0x79, Mnemonic::ADC, 3, 4 /*+1 if page crossed*/, AddressingMode::AbsoluteY),
        OpCode::new(0x61, Mnemonic::ADC, 2, 6, AddressingMode::IndirectX),
        OpCode::new(0x71, Mnemonic::ADC, 2, 5 /*+1 if page crossed*/, AddressingMode::IndirectY),

        OpCode::new(0x29, Mnemonic::AND, 2, 2, AddressingMode::Immediate),
        OpCode::new(0x25, Mnemonic::AND, 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0x35, Mnemonic::AND, 2, 4, AddressingMode::ZeroPageX),
        OpCode::new(0x2d, Mnemonic::AND, 3, 4, AddressingMode::Absolute),
        OpCode::new(0x3d, Mnemonic::AND, 3, 4 /*+1 if page crossed*/, AddressingMode::AbsoluteX),
        OpCode::new(0x39, Mnemonic::AND, 3, 4 /*+1 if page crossed*/, AddressingMode::AbsoluteY),
        OpCode::new(0x21, Mnemonic::AND, 2, 6, AddressingMode::IndirectX),
        OpCode::new(0x31, Mnemonic::AND, 2, 5 /*+1 if page crossed*/, AddressingMode::IndirectY),

        OpCode::new(0x0A, Mnemonic::ASL, 1, 2, AddressingMode::Accumulator),
        OpCode::new(0x06, Mnemonic::ASL, 2, 5, AddressingMode::ZeroPage),
        OpCode::new(0x16, Mnemonic::ASL, 2, 6, AddressingMode::ZeroPageX),
        OpCode::new(0x0E, Mnemonic::ASL, 3, 6, AddressingMode::Absolute),
        OpCode::new(0x1E, Mnemonic::ASL, 3, 7, AddressingMode::AbsoluteX),

        OpCode::new(0x24, Mnemonic::BIT, 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0x2C, Mnemonic::BIT, 3, 4, AddressingMode::Absolute),

        OpCode::new(0xc6, Mnemonic::DEC, 2, 5, AddressingMode::ZeroPage),
        OpCode::new(0xd6, Mnemonic::DEC, 2, 6, AddressingMode::ZeroPageX),
        OpCode::new(0xce, Mnemonic::DEC, 3, 6, AddressingMode::Absolute),
        OpCode::new(0xde, Mnemonic::DEC, 3, 7, AddressingMode::AbsoluteX),

        OpCode::new(0xca, Mnemonic::DEX, 1, 2, AddressingMode::Implied),
        OpCode::new(0x88, Mnemonic::DEY, 1, 2, AddressingMode::Implied),

        OpCode::new(0x49, Mnemonic::EOR, 2, 2, AddressingMode::Immediate),
        OpCode::new(0x45, Mnemonic::EOR, 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0x55, Mnemonic::EOR, 2, 4, AddressingMode::ZeroPageX),
        OpCode::new(0x4d, Mnemonic::EOR, 3, 4, AddressingMode::Absolute),
        OpCode::new(0x5d, Mnemonic::EOR, 3, 4 /*+1 if page crossed*/, AddressingMode::AbsoluteX),
        OpCode::new(0x59, Mnemonic::EOR, 3, 4 /*+1 if page crossed*/, AddressingMode::AbsoluteY),
        OpCode::new(0x41, Mnemonic::EOR, 2, 6, AddressingMode::IndirectX),
        OpCode::new(0x51, Mnemonic::EOR, 2, 5 /*+1 if page crossed*/, AddressingMode::IndirectY),

        OpCode::new(0xe6, Mnemonic::INC, 2, 5, AddressingMode::ZeroPage),
        OpCode::new(0xf6, Mnemonic::INC, 2, 6, AddressingMode::ZeroPageX),
        OpCode::new(0xee, Mnemonic::INC, 3, 6, AddressingMode::Absolute),
        OpCode::new(0xfe, Mnemonic::INC, 3, 7, AddressingMode::AbsoluteX),

        OpCode::new(0xe8, Mnemonic::INX, 1, 2, AddressingMode::Implied),
        OpCode::new(0xc8, Mnemonic::INY, 1, 2, AddressingMode::Implied),

        OpCode::new(0xa9, Mnemonic::LDA, 2, 2, AddressingMode::Immediate),
        OpCode::new(0xa5, Mnemonic::LDA, 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0xb5, Mnemonic::LDA, 2, 4, AddressingMode::ZeroPageX),
        OpCode::new(0xad, Mnemonic::LDA, 3, 4, AddressingMode::Absolute),
        OpCode::new(0xbd, Mnemonic::LDA, 3, 4 /*+1 if page crossed*/, AddressingMode::AbsoluteX),
        OpCode::new(0xb9, Mnemonic::LDA, 3, 4 /*+1 if page crossed*/, AddressingMode::AbsoluteY),
        OpCode::new(0xa1, Mnemonic::LDA, 2, 6, AddressingMode::IndirectX),
        OpCode::new(0xb1, Mnemonic::LDA, 2, 5 /*+1 if page crossed*/, AddressingMode::IndirectY),

        OpCode::new(0xa2, Mnemonic::LDX, 2, 2, AddressingMode::Immediate),
        OpCode::new(0xa6, Mnemonic::LDX, 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0xb6, Mnemonic::LDX, 2, 4, AddressingMode::ZeroPageY),
        OpCode::new(0xae, Mnemonic::LDX, 3, 4, AddressingMode::Absolute),
        OpCode::new(0xbe, Mnemonic::LDX, 3, 4 /*+1 if page crossed*/, AddressingMode::AbsoluteY),

        OpCode::new(0xa0, Mnemonic::LDY, 2, 2, AddressingMode::Immediate),
        OpCode::new(0xa4, Mnemonic::LDY, 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0xb4, Mnemonic::LDY, 2, 4, AddressingMode::ZeroPageX),
        OpCode::new(0xac, Mnemonic::LDY, 3, 4, AddressingMode::Absolute),
        OpCode::new(0xbc, Mnemonic::LDY, 3, 4 /*+1 if page crossed*/, AddressingMode::AbsoluteX),

        OpCode::new(0x4A, Mnemonic::LSR, 1, 2, AddressingMode::Accumulator),
        OpCode::new(0x46, Mnemonic::LSR, 2, 5, AddressingMode::ZeroPage),
        OpCode::new(0x56, Mnemonic::LSR, 2, 6, AddressingMode::ZeroPageX),
        OpCode::new(0x4E, Mnemonic::LSR, 3, 6, AddressingMode::Absolute),
        OpCode::new(0x5E, Mnemonic::LSR, 3, 7, AddressingMode::AbsoluteX),

        OpCode::new(0xea, Mnemonic::NOP, 1, 2, AddressingMode::Implied),

        OpCode::new(0x09, Mnemonic::ORA, 2, 2, AddressingMode::Immediate),
        OpCode::new(0x05, Mnemonic::ORA, 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0x15, Mnemonic::ORA, 2, 4, AddressingMode::ZeroPageX),
        OpCode::new(0x0d, Mnemonic::ORA, 3, 4, AddressingMode::Absolute),
        OpCode::new(0x1d, Mnemonic::ORA, 3, 4 /*+1 if page crossed*/, AddressingMode::AbsoluteX),
        OpCode::new(0x19, Mnemonic::ORA, 3, 4 /*+1 if page crossed*/, AddressingMode::AbsoluteY),
        OpCode::new(0x01, Mnemonic::ORA, 2, 6, AddressingMode::IndirectX),
        OpCode::new(0x11, Mnemonic::ORA, 2, 5 /*+1 if page crossed*/, AddressingMode::IndirectY),

        OpCode::new(0x2a, Mnemonic::ROL, 1, 2, AddressingMode::Accumulator),
        OpCode::new(0x26, Mnemonic::ROL, 2, 5, AddressingMode::ZeroPage),
        OpCode::new(0x36, Mnemonic::ROL, 2, 6, AddressingMode::ZeroPageX),
        OpCode::new(0x2e, Mnemonic::ROL, 3, 6, AddressingMode::Absolute),
        OpCode::new(0x3e, Mnemonic::ROL, 3, 7, AddressingMode::AbsoluteX),

        OpCode::new(0x6a, Mnemonic::ROR, 1, 2, AddressingMode::Accumulator),
        OpCode::new(0x66, Mnemonic::ROR, 2, 5, AddressingMode::ZeroPage),
        OpCode::new(0x76, Mnemonic::ROR, 2, 6, AddressingMode::ZeroPageX),
        OpCode::new(0x6e, Mnemonic::ROR, 3, 6, AddressingMode::Absolute),
        OpCode::new(0x7e, Mnemonic::ROR, 3, 7, AddressingMode::AbsoluteX),

        OpCode::new(0xe9, Mnemonic::SBC, 2, 2, AddressingMode::Immediate),
        OpCode::new(0xe5, Mnemonic::SBC, 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0xf5, Mnemonic::SBC, 2, 4, AddressingMode::ZeroPageX),
        OpCode::new(0xed, Mnemonic::SBC, 3, 4, AddressingMode::Absolute),
        OpCode::new(0xfd, Mnemonic::SBC, 3, 4 /*+1 if page crossed*/, AddressingMode::AbsoluteX),
        OpCode::new(0xf9, Mnemonic::SBC, 3, 4 /*+1 if page crossed*/, AddressingMode::AbsoluteY),
        OpCode::new(0xe1, Mnemonic::SBC, 2, 6, AddressingMode::IndirectX),
        OpCode::new(0xf1, Mnemonic::SBC, 2, 5 /*+1 if page crossed*/, AddressingMode::IndirectY),

        OpCode::new(0x85, Mnemonic::STA, 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0x95, Mnemonic::STA, 2, 4, AddressingMode::ZeroPageX),
        OpCode::new(0x8d, Mnemonic::STA, 3, 4, AddressingMode::Absolute),
        OpCode::new(0x9d, Mnemonic::STA, 3, 5, AddressingMode::AbsoluteX),
        OpCode::new(0x99, Mnemonic::STA, 3, 5, AddressingMode::AbsoluteY),
        OpCode::new(0x81, Mnemonic::STA, 2, 6, AddressingMode::IndirectX),
        OpCode::new(0x91, Mnemonic::STA, 2, 6, AddressingMode::IndirectY),

        OpCode::new(0x86, Mnemonic::STX, 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0x96, Mnemonic::STX, 2, 4, AddressingMode::ZeroPageY),
        OpCode::new(0x8e, Mnemonic::STX, 3, 4, AddressingMode::Absolute),

        OpCode::new(0x84, Mnemonic::STY, 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0x94, Mnemonic::STY, 2, 4, AddressingMode::ZeroPageX),
        OpCode::new(0x8c, Mnemonic::STY, 3, 4, AddressingMode::Absolute),

        OpCode::new(0xaa, Mnemonic::TAX, 1, 2, AddressingMode::Implied),
        OpCode::new(0xa8, Mnemonic::TAY, 1, 2, AddressingMode::Implied),
        OpCode::new(0x8a, Mnemonic::TXA, 1, 2, AddressingMode::Implied),
        OpCode::new(0x98, Mnemonic::TYA, 1, 2, AddressingMode::Implied),
        OpCode::new(0xba, Mnemonic::TSX, 1, 2, AddressingMode::Implied),
        OpCode::new(0x9a, Mnemonic::TXS, 1, 2, AddressingMode::Implied),

        // STACK
        OpCode::new(0x08, Mnemonic::PHP, 1, 3, AddressingMode::Implied),
        OpCode::new(0x28, Mnemonic::PLP, 1, 3, AddressingMode::Implied),
        OpCode::new(0x48, Mnemonic::PHA, 1, 3, AddressingMode::Implied),
        OpCode::new(0x68, Mnemonic::PLA, 1, 3, AddressingMode::Implied),

        // SUBROUTINE
        OpCode::new(0x20, Mnemonic::JSR, 3, 6, AddressingMode::Absolute),
        OpCode::new(0x40, Mnemonic::RTI, 1, 6, AddressingMode::Implied),
        OpCode::new(0x60, Mnemonic::RTS, 1, 6, AddressingMode::Implied),

        // COMPARE
        OpCode::new(0xc9, Mnemonic::CMP, 2, 2, AddressingMode::Immediate),
        OpCode::new(0xc5, Mnemonic::CMP, 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0xd5, Mnemonic::CMP, 2, 4, AddressingMode::ZeroPageX),
        OpCode::new(0xcd, Mnemonic::CMP, 3, 4, AddressingMode::Absolute),
        OpCode::new(0xdd, Mnemonic::CMP, 3, 4 /*+1 if page crossed*/, AddressingMode::AbsoluteX),
        OpCode::new(0xd9, Mnemonic::CMP, 3, 4 /*+1 if page crossed*/, AddressingMode::AbsoluteY),
        OpCode::new(0xc1, Mnemonic::CMP, 2, 6, AddressingMode::IndirectX),
        OpCode::new(0xd1, Mnemonic::CMP, 2, 5 /*+1 if page crossed*/, AddressingMode::IndirectY),

        OpCode::new(0xe0, Mnemonic::CPX, 2, 2, AddressingMode::Immediate),
        OpCode::new(0xe4, Mnemonic::CPX, 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0xec, Mnemonic::CPX, 3, 4, AddressingMode::Absolute),

        OpCode::new(0xc0, Mnemonic::CPY, 2, 2, AddressingMode::Immediate),
        OpCode::new(0xc4, Mnemonic::CPY, 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0xcc, Mnemonic::CPY, 3, 4, AddressingMode::Absolute),

        // BRANCHING
        OpCode::new(0x00, Mnemonic::BRK, 1, 7, AddressingMode::Implied),
        OpCode::new(0x10, Mnemonic::BPL, 2, 2 /*+1 if branch succeeds, +2 if to a new page*/, AddressingMode::Relative),
        OpCode::new(0x30, Mnemonic::BMI, 2, 2 /*+1 if branch succeeds, +2 if to a new page*/, AddressingMode::Relative),
        OpCode::new(0x50, Mnemonic::BVC, 2, 2 /*+1 if branch succeeds, +2 if to a new page*/, AddressingMode::Relative),
        OpCode::new(0x70, Mnemonic::BVS, 2, 2 /*+1 if branch succeeds, +2 if to a new page*/, AddressingMode::Relative),
        OpCode::new(0x90, Mnemonic::BCC, 2, 2 /*+1 if branch succeeds, +2 if to a new page*/, AddressingMode::Relative),
        OpCode::new(0xb0, Mnemonic::BCS, 2, 2 /*+1 if branch succeeds, +2 if to a new page*/, AddressingMode::Relative),
        OpCode::new(0xd0, Mnemonic::BNE, 2, 2 /*+1 if branch succeeds, +2 if to a new page*/, AddressingMode::Relative),
        OpCode::new(0xf0, Mnemonic::BEQ, 2, 2 /*+1 if branch succeeds, +2 if to a new page*/, AddressingMode::Relative),

        // JUMPS
        OpCode::new(0x4c, Mnemonic::JMP, 3, 3, AddressingMode::Absolute),
        OpCode::new(0x6c, Mnemonic::JMP, 3, 5, AddressingMode::Indirect),

        // SET
        OpCode::new(0x38, Mnemonic::SEC, 1, 2, AddressingMode::Implied),
        OpCode::new(0x78, Mnemonic::SEI, 1, 2, AddressingMode::Implied),
        OpCode::new(0xf8, Mnemonic::SED, 1, 2, AddressingMode::Implied),

        // CLEAR
        OpCode::new(0x18, Mnemonic::CLC, 1, 2, AddressingMode::Implied),
        OpCode::new(0x58, Mnemonic::CLI, 1, 2, AddressingMode::Implied),
        OpCode::new(0xb8, Mnemonic::CLV, 1, 2, AddressingMode::Implied),
        OpCode::new(0xd8, Mnemonic::CLD, 1, 2, AddressingMode::Implied),

        // UNOFFICIAL
        OpCode::new(0x0b, Mnemonic::AAC_Unofficial, 2, 2, AddressingMode::Immediate),
        OpCode::new(0x2b, Mnemonic::AAC_Unofficial, 2, 2, AddressingMode::Immediate),

        OpCode::new(0x87, Mnemonic::AAX_Unofficial, 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0x97, Mnemonic::AAX_Unofficial, 2, 4, AddressingMode::ZeroPageY),
        OpCode::new(0x83, Mnemonic::AAX_Unofficial, 2, 6, AddressingMode::IndirectX),
        OpCode::new(0x8f, Mnemonic::AAX_Unofficial, 3, 4, AddressingMode::Absolute),

        OpCode::new(0x6b, Mnemonic::ARR_Unofficial, 2, 2, AddressingMode::Immediate),

        OpCode::new(0x4b, Mnemonic::ASR_Unofficial, 2, 2, AddressingMode::Immediate),

        OpCode::new(0xab, Mnemonic::ATX_Unofficial, 2, 2, AddressingMode::Immediate),

        OpCode::new(0x9f, Mnemonic::AXA_Unofficial, 3, 5, AddressingMode::AbsoluteY),
        OpCode::new(0x93, Mnemonic::AXA_Unofficial, 2, 6, AddressingMode::IndirectY),

        OpCode::new(0xcb, Mnemonic::AXS_Unofficial, 2, 2, AddressingMode::Immediate),

        OpCode::new(0xc7, Mnemonic::DCP_Unofficial, 2, 5, AddressingMode::ZeroPage),
        OpCode::new(0xd7, Mnemonic::DCP_Unofficial, 2, 6, AddressingMode::ZeroPageX),
        OpCode::new(0xcf, Mnemonic::DCP_Unofficial, 3, 6, AddressingMode::Absolute),
        OpCode::new(0xdf, Mnemonic::DCP_Unofficial, 3, 7, AddressingMode::AbsoluteX),
        OpCode::new(0xdb, Mnemonic::DCP_Unofficial, 3, 7, AddressingMode::AbsoluteY),
        OpCode::new(0xc3, Mnemonic::DCP_Unofficial, 2, 8, AddressingMode::IndirectX),
        OpCode::new(0xd3, Mnemonic::DCP_Unofficial, 2, 8, AddressingMode::IndirectY),

        OpCode::new(0x04, Mnemonic::DOP_Unofficial, 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0x14, Mnemonic::DOP_Unofficial, 2, 4, AddressingMode::ZeroPageX),
        OpCode::new(0x34, Mnemonic::DOP_Unofficial, 2, 4, AddressingMode::ZeroPageX),
        OpCode::new(0x44, Mnemonic::DOP_Unofficial, 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0x54, Mnemonic::DOP_Unofficial, 2, 4, AddressingMode::ZeroPageX),
        OpCode::new(0x64, Mnemonic::DOP_Unofficial, 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0x74, Mnemonic::DOP_Unofficial, 2, 4, AddressingMode::ZeroPageX),
        OpCode::new(0x80, Mnemonic::DOP_Unofficial, 2, 2, AddressingMode::Immediate),
        OpCode::new(0x82, Mnemonic::DOP_Unofficial, 2, 2, AddressingMode::Immediate),
        OpCode::new(0x89, Mnemonic::DOP_Unofficial, 2, 2, AddressingMode::Immediate),
        OpCode::new(0xc2, Mnemonic::DOP_Unofficial, 2, 2, AddressingMode::Immediate),
        OpCode::new(0xd4, Mnemonic::DOP_Unofficial, 2, 4, AddressingMode::ZeroPageX),
        OpCode::new(0xe2, Mnemonic::DOP_Unofficial, 2, 2, AddressingMode::ZeroPage),
        OpCode::new(0xf4, Mnemonic::DOP_Unofficial, 2, 4, AddressingMode::ZeroPageX),

        OpCode::new(0xe7, Mnemonic::ISC_Unofficial, 2, 5, AddressingMode::ZeroPage),
        OpCode::new(0xf7, Mnemonic::ISC_Unofficial, 2, 6, AddressingMode::ZeroPageX),
        OpCode::new(0xef, Mnemonic::ISC_Unofficial, 3, 6, AddressingMode::Absolute),
        OpCode::new(0xff, Mnemonic::ISC_Unofficial, 3, 7, AddressingMode::AbsoluteX),
        OpCode::new(0xfb, Mnemonic::ISC_Unofficial, 3, 7, AddressingMode::AbsoluteY),
        OpCode::new(0xe3, Mnemonic::ISC_Unofficial, 2, 8, AddressingMode::IndirectX),
        OpCode::new(0xf3, Mnemonic::ISC_Unofficial, 2, 8, AddressingMode::IndirectY),

        OpCode::new(0x02, Mnemonic::KIL_Unofficial, 1, 0, AddressingMode::Implied),
        OpCode::new(0x12, Mnemonic::KIL_Unofficial, 1, 0, AddressingMode::Implied),
        OpCode::new(0x22, Mnemonic::KIL_Unofficial, 1, 0, AddressingMode::Implied),
        OpCode::new(0x32, Mnemonic::KIL_Unofficial, 1, 0, AddressingMode::Implied),
        OpCode::new(0x42, Mnemonic::KIL_Unofficial, 1, 0, AddressingMode::Implied),
        OpCode::new(0x52, Mnemonic::KIL_Unofficial, 1, 0, AddressingMode::Implied),
        OpCode::new(0x62, Mnemonic::KIL_Unofficial, 1, 0, AddressingMode::Implied),
        OpCode::new(0x72, Mnemonic::KIL_Unofficial, 1, 0, AddressingMode::Implied),
        OpCode::new(0x92, Mnemonic::KIL_Unofficial, 1, 0, AddressingMode::Implied),
        OpCode::new(0xb2, Mnemonic::KIL_Unofficial, 1, 0, AddressingMode::Implied),
        OpCode::new(0xd2, Mnemonic::KIL_Unofficial, 1, 0, AddressingMode::Implied),
        OpCode::new(0xf2, Mnemonic::KIL_Unofficial, 1, 0, AddressingMode::Implied),

        OpCode::new(0xbb, Mnemonic::LAR_Unofficial, 3, 4 /*+1 if page crossed*/, AddressingMode::AbsoluteY),

        OpCode::new(0xa7, Mnemonic::LAX_Unofficial, 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0xb7, Mnemonic::LAX_Unofficial, 2, 4, AddressingMode::ZeroPageY),
        OpCode::new(0xaf, Mnemonic::LAX_Unofficial, 3, 4, AddressingMode::Absolute),
        OpCode::new(0xbf, Mnemonic::LAX_Unofficial, 3, 4 /*+1 if page crossed*/, AddressingMode::AbsoluteY),
        OpCode::new(0xa3, Mnemonic::LAX_Unofficial, 2, 6, AddressingMode::IndirectX),
        OpCode::new(0xb3, Mnemonic::LAX_Unofficial, 2, 5 /*+1 if page crossed*/, AddressingMode::IndirectY),

        OpCode::new(0x1a, Mnemonic::NOP_Unofficial, 1, 2, AddressingMode::Implied),
        OpCode::new(0x3a, Mnemonic::NOP_Unofficial, 1, 2, AddressingMode::Implied),
        OpCode::new(0x5a, Mnemonic::NOP_Unofficial, 1, 2, AddressingMode::Implied),
        OpCode::new(0x7a, Mnemonic::NOP_Unofficial, 1, 2, AddressingMode::Implied),
        OpCode::new(0xda, Mnemonic::NOP_Unofficial, 1, 2, AddressingMode::Implied),
        OpCode::new(0xfa, Mnemonic::NOP_Unofficial, 1, 2, AddressingMode::Implied),

        OpCode::new(0x27, Mnemonic::RLA_Unofficial, 2, 5, AddressingMode::ZeroPage),
        OpCode::new(0x37, Mnemonic::RLA_Unofficial, 2, 6, AddressingMode::ZeroPageX),
        OpCode::new(0x2f, Mnemonic::RLA_Unofficial, 3, 6, AddressingMode::Absolute),
        OpCode::new(0x3f, Mnemonic::RLA_Unofficial, 3, 7, AddressingMode::AbsoluteX),
        OpCode::new(0x3b, Mnemonic::RLA_Unofficial, 3, 7, AddressingMode::AbsoluteY),
        OpCode::new(0x23, Mnemonic::RLA_Unofficial, 2, 8, AddressingMode::IndirectX),
        OpCode::new(0x33, Mnemonic::RLA_Unofficial, 2, 8, AddressingMode::IndirectY),

        OpCode::new(0x67, Mnemonic::RRA_Unofficial, 2, 5, AddressingMode::ZeroPage),
        OpCode::new(0x77, Mnemonic::RRA_Unofficial, 2, 6, AddressingMode::ZeroPageX),
        OpCode::new(0x6f, Mnemonic::RRA_Unofficial, 3, 6, AddressingMode::Absolute),
        OpCode::new(0x7f, Mnemonic::RRA_Unofficial, 3, 7, AddressingMode::AbsoluteX),
        OpCode::new(0x7b, Mnemonic::RRA_Unofficial, 3, 7, AddressingMode::AbsoluteY),
        OpCode::new(0x63, Mnemonic::RRA_Unofficial, 2, 8, AddressingMode::IndirectX),
        OpCode::new(0x73, Mnemonic::RRA_Unofficial, 2, 8, AddressingMode::IndirectY),

        OpCode::new(0xeb, Mnemonic::SBC_Unofficial, 2, 2, AddressingMode::Immediate),

        OpCode::new(0x07, Mnemonic::SLO_Unofficial, 2, 5, AddressingMode::ZeroPage),
        OpCode::new(0x17, Mnemonic::SLO_Unofficial, 2, 6, AddressingMode::ZeroPageX),
        OpCode::new(0x0f, Mnemonic::SLO_Unofficial, 3, 6, AddressingMode::Absolute),
        OpCode::new(0x1f, Mnemonic::SLO_Unofficial, 3, 7, AddressingMode::AbsoluteX),
        OpCode::new(0x1b, Mnemonic::SLO_Unofficial, 3, 7, AddressingMode::AbsoluteY),
        OpCode::new(0x03, Mnemonic::SLO_Unofficial, 2, 8, AddressingMode::IndirectX),
        OpCode::new(0x13, Mnemonic::SLO_Unofficial, 2, 8, AddressingMode::IndirectY),

        OpCode::new(0x47, Mnemonic::SRE_Unofficial, 2, 5, AddressingMode::ZeroPage),
        OpCode::new(0x57, Mnemonic::SRE_Unofficial, 2, 6, AddressingMode::ZeroPageX),
        OpCode::new(0x4f, Mnemonic::SRE_Unofficial, 3, 6, AddressingMode::Absolute),
        OpCode::new(0x5f, Mnemonic::SRE_Unofficial, 3, 7, AddressingMode::AbsoluteX),
        OpCode::new(0x5b, Mnemonic::SRE_Unofficial, 3, 7, AddressingMode::AbsoluteY),
        OpCode::new(0x43, Mnemonic::SRE_Unofficial, 2, 8, AddressingMode::IndirectX),
        OpCode::new(0x53, Mnemonic::SRE_Unofficial, 2, 8, AddressingMode::IndirectY),

        OpCode::new(0x9e, Mnemonic::SXA_Unofficial, 3, 5, AddressingMode::AbsoluteY),

        OpCode::new(0x9c, Mnemonic::SYA_Unofficial, 3, 5, AddressingMode::AbsoluteY),

        OpCode::new(0x0c, Mnemonic::TOP_Unofficial, 3, 4, AddressingMode::Absolute),
        OpCode::new(0x1c, Mnemonic::TOP_Unofficial, 3, 4 /*+1 if page crossed*/, AddressingMode::AbsoluteX),
        OpCode::new(0x3c, Mnemonic::TOP_Unofficial, 3, 4 /*+1 if page crossed*/, AddressingMode::AbsoluteX),
        OpCode::new(0x5c, Mnemonic::TOP_Unofficial, 3, 4 /*+1 if page crossed*/, AddressingMode::AbsoluteX),
        OpCode::new(0x7c, Mnemonic::TOP_Unofficial, 3, 4 /*+1 if page crossed*/, AddressingMode::AbsoluteX),
        OpCode::new(0xdc, Mnemonic::TOP_Unofficial, 3, 4 /*+1 if page crossed*/, AddressingMode::AbsoluteX),
        OpCode::new(0xfc, Mnemonic::TOP_Unofficial, 3, 4 /*+1 if page crossed*/, AddressingMode::AbsoluteX),

        OpCode::new(0x8b, Mnemonic::XAA_Unofficial, 2, 3, AddressingMode::Immediate),

        OpCode::new(0x9b, Mnemonic::XAS_Unofficial, 3, 5, AddressingMode::AbsoluteY),
    ];

    pub static ref OPCODES_MAP: HashMap<u8, &'static OpCode> = {
        let mut map = HashMap::new();
        for op_code in &*CPU_OPS_CODES {
            map.insert(op_code.code, op_code);
        }
        map
    };
}
