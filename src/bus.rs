//! Below is a look into the bus memory.
//! ```
//! | ______________|$10000 | ______________|
//! | PRG-ROM       |       |               |
//! | Upper Bank    |       |               |
//! |_ _ _ _ _ _ _ _| $C000 | PRG-ROM       |
//! | PRG-ROM       |       |               |
//! | Lower Bank    |       |               |
//! |_______________| $8000 |_______________|
//! | SRAM          |       | SRAM          |
//! |_______________| $6000 |_______________|
//! | Expansion ROM |       | Expansion ROM |
//! |_______________| $4020 |_______________|
//! | I/O Registers |       |               |
//! |_ _ _ _ _ _ _ _| $4000 |               |
//! | Mirrors       |       | I/O Registers |
//! | $2000-$2007   |       |               |
//! |_ _ _ _ _ _ _ _| $2008 |               |
//! | I/O Registers |       |               |
//! |_______________| $2000 |_______________|
//! | Mirrors       |       |               |
//! | $0000-$07FF   |       |               |
//! |_ _ _ _ _ _ _ _| $0800 |               |
//! | RAM           |       | RAM           |
//! |_ _ _ _ _ _ _ _| $0200 |               |
//! | Stack         |       |               |
//! |_ _ _ _ _ _ _ _| $0100 |               |
//! | Zero Page     |       |               |
//! |_______________| $0000 |_______________|
//! ```
use crate::{cartridge::Rom, cpu::Mem, ppu::NesPPU};

const RAM: u16 = 0x0000;
const RAM_MIRRORS_END: u16 = 0x1FFF;

const PPU_CTRL_REGISTER: u16 = 0x2000;
const PPU_MASK_REGISTER: u16 = 0x2001;
const PPU_STATUS_REGISTER: u16 = 0x2002;
const PPU_OAM_ADDRESS_REGISTER: u16 = 0x2003;
const PPU_OAM_DATA_REGISTER: u16 = 0x2004;
const PPU_SCROLL_REGISTER: u16 = 0x2005;
const PPU_ADDR_REGISTER: u16 = 0x2006;
const PPU_DATA_REGISTER: u16 = 0x2007;
const PPU_REGISTERS_MIRROR_START: u16 = 0x2008;
const PPU_REGISTERS_MIRRORS_END: u16 = 0x3FFF;
const PPU_DIRECT_MEMORY_ACCESS_REGISTER: u16 = 0x4014;

const PRG_ROM: u16 = 0x8000;
const PRG_ROM_END: u16 = 0xFFFF;

pub struct Bus {
    cpu_vram: [u8; 2048],
    prg_rom: Vec<u8>,
    ppu: NesPPU,
}

impl Bus {
    pub fn new(rom: Rom) -> Self {
        let ppu = NesPPU::new(rom.chr_rom, rom.screen_mirroring);

        Bus {
            cpu_vram: [0; 2048],
            prg_rom: rom.prg_rom,
            ppu,
        }
    }

    /// PRG Rom Size might be 16 KiB or 32 KiB.
    /// Because [0x8000 … 0x10000] mapped region is 32 KiB of addressable space, the upper 16 KiB needs to be mapped to the lower 16 KiB (if a game has only 16 KiB of PRG ROM)
    fn read_prg_rom(&self, mut addr: u16) -> u8 {
        addr -= 0x8000;
        if self.prg_rom.len() == 0x4000 && addr >= 0x4000 {
            // mirror if needed
            addr %= 0x4000;
        }
        self.prg_rom[addr as usize]
    }
}

impl Mem for Bus {
    fn mem_read(&mut self, addr: u16) -> u8 {
        match addr {
            // this matches any address that satisfies: RAM <= addr <= RAM_MIRRORS_END
            RAM..=RAM_MIRRORS_END => {
                // NES only has 11 bits for addressing so we mask it accordingly
                let mirror_down_addr = addr & 0b0000_0111_1111_1111;
                self.cpu_vram[mirror_down_addr as usize]
            }
            PPU_CTRL_REGISTER
            | PPU_MASK_REGISTER
            | PPU_OAM_ADDRESS_REGISTER
            | PPU_SCROLL_REGISTER
            | PPU_ADDR_REGISTER
            | PPU_DIRECT_MEMORY_ACCESS_REGISTER => {
                panic!("Attempt to read from write-only PPU address {:x}", addr);
            }
            PPU_STATUS_REGISTER => self.ppu.read_status_register(),
            PPU_OAM_DATA_REGISTER => self.ppu.read_oam_data_register(),
            PPU_DATA_REGISTER => self.ppu.read_data_register(),
            PPU_REGISTERS_MIRROR_START..=PPU_REGISTERS_MIRRORS_END => {
                // because every register after 0x2007 is mirrored to the previous 8 byte we need to mirror down
                // example: 0x3456 is the same as 0x2006
                let mirror_down_addr = addr & 0b0010_0000_0000_0111;
                self.mem_read(mirror_down_addr)
            }
            PRG_ROM..=PRG_ROM_END => self.read_prg_rom(addr),

            _ => {
                println!("Ignoring mem access at {}", addr);
                0
            }
        }
    }

    fn mem_write(&mut self, addr: u16, data: u8) {
        match addr {
            RAM..=RAM_MIRRORS_END => {
                let mirror_down_addr = addr & 0b111_1111_1111;
                self.cpu_vram[mirror_down_addr as usize] = data;
            }
            PPU_CTRL_REGISTER => self.ppu.write_to_ctrl_register(data),
            PPU_MASK_REGISTER => self.ppu.write_to_mask_register(data),
            PPU_STATUS_REGISTER => panic!("Attempt to write to read-only PPU status register"),
            PPU_OAM_ADDRESS_REGISTER => self.ppu.write_to_oam_addr_register(data),
            PPU_OAM_DATA_REGISTER => self.ppu.write_to_oam_data_register(data),
            PPU_SCROLL_REGISTER => self.ppu.write_to_scroll_register(data),
            PPU_ADDR_REGISTER => self.ppu.write_to_addr_register(data),
            PPU_DATA_REGISTER => self.ppu.write_to_data_register(data),
            PPU_REGISTERS_MIRROR_START..=PPU_REGISTERS_MIRRORS_END => {
                let mirror_down_addr = addr & 0b0010_0000_0000_0111;
                self.mem_write(mirror_down_addr, data);
            }
            PRG_ROM..=PRG_ROM_END => panic!("Attempt to write to Cartridge ROM space"),

            _ => println!("Ignoring mem write-access at {}", addr),
        }
    }
}
