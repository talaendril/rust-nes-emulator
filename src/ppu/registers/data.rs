use crate::cartridge::Mirroring;

pub struct DataRegister {
    // I don't want this to be pub but I need it for tests right now, TODO
    pub vram: [u8; 2048],    // internal memory, keeps palette tables
    chr_rom: Vec<u8>,        // visuals of the game stored on the cartridge
    palette_table: [u8; 32], // internal memory, keeps palette tables
    mirroring: Mirroring,
    internal_data_buf: u8,
}

impl DataRegister {
    pub fn new(chr_rom: Vec<u8>, mirroring: Mirroring) -> Self {
        DataRegister {
            vram: [0; 2048],
            chr_rom,
            palette_table: [0; 32],
            mirroring,
            internal_data_buf: 0,
        }
    }

    pub fn read_data(&mut self, addr: u16) -> u8 {
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
            _ => panic!("Unexpected read access to mirrored space {}", addr),
        }
    }

    pub fn write_data(&mut self, addr: u16, data: u8) {
        match addr {
            // TODO: guide uses println!, maybe this is actually called occasionally?
            0..=0x1fff => panic!("Attempt to write to chr rom space: {}", addr),
            0x2000..=0x2fff => self.vram[self.mirror_vram_addr(addr) as usize] = data,
            0x3000..=0x3eff => panic!(
                "addr space 0x3000..0x3eff is not expected to be used, requested = {} ",
                addr
            ),
            // I understand that these addresses mirror each other but I don't understand why it should be important for write specifically
            // This reference here: https://bytes.vokal.io/7-nes-ppu/ mentions that mirroring but I'll just keep it like that for now
            // Addresses $3F10/$3F14/$3F18/$3F1C are mirrors of $3F00/$3F04/$3F08/$3F0C
            // 0x3f10 | 0x3f14 | 0x3f18 | 0x3f1c => {
            //     let add_mirror = addr - 0x10;
            //     self.palette_table[(add_mirror - 0x3f00) as usize] = data;
            // }
            0x3f00..=0x3fff => self.palette_table[(addr - 0x3f00) as usize] = data,
            _ => panic!("Unexpected write access to mirrored space {}", addr),
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
    fn mirror_vram_addr(&self, addr: u16) -> u16 {
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
