//! Important notes:
//! PPU renders 262 scanlines per frame, of those 0-240 are visible, the rest are vertical overscans.
//! Each scanline lasts 341 PPU clock cycles and each clock cycle produces one pixel.
//! After the 240th scanline the PPU triggers VBlank NMI on the CPU and accesses no more memory.
//! Which means that usually game state updates happen during scanlines 241-262.
//!
//! The PPU also exposes 8 I/O registers that the CPU uses for communication. These registers
//! are mapped to 0x2000 - 0x2007 in the CPU memory map and then mirrored every 8 bytes from
//! 0x2008 - 0x3FFF.

pub mod registers;

use crate::cartridge::Mirroring;

use self::registers::{address::AddrRegister, controller::ControlRegister};

pub struct NesPPU {
    pub chr_rom: Vec<u8>,        // visuals of the game stored on the cartridge
    pub palette_table: [u8; 32], // internal memory, keeps palette tables
    pub vram: [u8; 2048],        // internal memory, keeps palette tables
    pub oam_data: [u8; 256],     // oam => called Object Attribute Memory, keeps sprite state
    pub mirroring: Mirroring,
    pub ctrl: ControlRegister, // register at 0x2000, write-only
    addr: AddrRegister,        // register at 0x2006
    internal_data_buf: u8,
}

impl NesPPU {
    pub fn new(chr_rom: Vec<u8>, mirroring: Mirroring) -> Self {
        NesPPU {
            chr_rom,
            palette_table: [0; 32],
            vram: [0; 2048],
            oam_data: [0; 256],
            mirroring,
            ctrl: ControlRegister::new(),
            addr: AddrRegister::new(),
            internal_data_buf: 0,
        }
    }

    pub fn write_to_ppu_addr(&mut self, value: u8) {
        self.addr.update(value);
    }

    fn increment_vram_addr(&mut self) {
        self.addr
            .increment(self.ctrl.get_vram_addr_increment_value());
    }

    pub fn read_data(&mut self) -> u8 {
        let addr = self.addr.get();
        self.increment_vram_addr();

        match addr {
            // pattern tables => chr rom access
            0..=0x1fff => {
                let result = self.internal_data_buf;
                self.internal_data_buf = self.chr_rom[addr as usize];
                result
            }
            // name tables => vram tables
            0x2000..=0x2fff => {
                let result = self.internal_data_buf;
                self.internal_data_buf = self.vram[self.mirror_vram_addr(addr) as usize];
                result
            }
            0x3000..=0x3eff => panic!(
                "addr space 0x3000..0x3eff is not expected to be used, requested = {} ",
                addr
            ),
            // palette tables
            0x3f00..=0x3fff => self.palette_table[(addr - 0x3f00) as usize],
            _ => panic!("unexpected access to mirrored space {}", addr),
        }
    }

    /// NES uses 1 KiB of VRAM to represent the state of a single screen. Since it has 2 KiB it can store 2 screen states.
    /// Range 0x2000-0x3f00 on the PPU memory map is reserved for nametables (screen states). Two additional screens have
    /// to be mapped to the existing ones on the PPU memory map. The mapping depends on the mirroring type.
    ///
    /// Horizontal:
    /// [0x2000-0x2400] and [0x2400-0x2800] should be mapped to the first 1 KiB of VRAM.
    /// [0x2800-0x2C00] and [0x2C00-0x3F00] should be mapped to the second 1 KiB of VRAM.
    ///  [ A ] [ a ]
    ///  [ B ] [ b ]
    ///
    /// Vertical:
    ///  [ A ] [ B ]
    ///  [ a ] [ b ]
    pub fn mirror_vram_addr(&self, addr: u16) -> u16 {
        let mirrored_vram = addr & 0b0010_1111_1111_1111; // mirror down 0x3000-0x3eff to 0x2000 - 0x2eff
        let vram_index = mirrored_vram - 0x2000; // to vram vector
        let name_table = vram_index / 0x400; // to name table index

        match (&self.mirroring, name_table) {
            (Mirroring::VERTICAL, 2) | (Mirroring::VERTICAL, 3) => vram_index - 0x800,
            (Mirroring::HORIZONTAl, 2) => vram_index - 0x400,
            (Mirroring::HORIZONTAl, 1) => vram_index - 0x400,
            (Mirroring::HORIZONTAl, 3) => vram_index - 0x800,
            _ => vram_index,
        }
    }
}
