//! Important notes:
//! PPU renders 262 scanlines per frame, of those 0-240 are visible, the rest are vertical overscans.
//! Each scanline lasts 341 PPU clock cycles and each clock cycle produces one pixel.
//! This means that each PPU frame takes 341 * 262 = 89342 PPU clock cycles.
//! After the 240th scanline the PPU triggers VBlank NMI on the CPU and accesses no more memory.
//! Which means that usually game state updates happen during scanlines 241-262.
//! Also the PPU clocks ticks 3 times faster than CPU.
//!
//! The PPU also exposes 8 I/O registers that the CPU uses for communication. These registers
//! are mapped to 0x2000 - 0x2007 in the CPU memory map and then mirrored every 8 bytes from
//! 0x2008 - 0x3FFF.

pub mod registers;

use crate::cartridge::Mirroring;

use self::registers::{
    address::AddrRegister, controller::ControlRegister, data::DataRegister, mask::MaskRegister,
    oam_address::OamAddressRegister, oam_data::OamDataRegister, scroll::ScrollRegister,
    status::StatusRegister,
};

pub struct NesPPU {
    ctrl: ControlRegister,        // register at 0x2000, write-only
    mask: MaskRegister,           // register at 0x2001, write-only
    status: StatusRegister,       // register at 0x2002, read-only
    oam_addr: OamAddressRegister, // register at 0x2003, write-only
    oam_data: OamDataRegister,    // register at 0x2004, read and write
    scroll: ScrollRegister,       // register at 0x2005, write-only => write called twice (16-bit)
    addr: AddrRegister,           // register at 0x2006, write-only => write called twice (16-bit)
    data: DataRegister,           // register at 0x2007, read and write
    scanline: u16,
    cycles: usize,
    nmi_interrupt: Option<u8>,
}

impl NesPPU {
    pub fn new(chr_rom: Vec<u8>, mirroring: Mirroring) -> Self {
        NesPPU {
            ctrl: ControlRegister::new(),
            mask: MaskRegister::new(),
            status: StatusRegister::new(),
            oam_addr: OamAddressRegister::new(),
            oam_data: OamDataRegister::new(),
            scroll: ScrollRegister::new(),
            addr: AddrRegister::new(),
            data: DataRegister::new(chr_rom, mirroring),
            scanline: 0,
            cycles: 0,
            nmi_interrupt: None,
        }
    }

    pub fn tick(&mut self, cycles: u8) -> bool {
        self.cycles += cycles as usize;

        // 341 PPU cycles are needed for 1 scanline to finish
        if self.cycles >= 341 {
            self.cycles -= 341;
            self.scanline += 1;

            // 241st scanline is not visible anymore ans is called vertical overscan
            if self.scanline == 241 {
                self.status.set_vblank_started();
                self.status.remove_sprite_zero_hit();
                if self.ctrl.generate_vblank_nmi() {
                    self.nmi_interrupt = Some(1);
                }

                return false;
            }

            // per frame 262 scanlines are rendered
            if self.scanline >= 262 {
                self.scanline = 0;
                self.nmi_interrupt = None;
                self.status.remove_sprite_zero_hit();
                self.status.clear_vblank_started();
                return true; // frame finished rendering
            }
        }

        false
    }

    pub fn take_nmi_interrupt(&mut self) -> Option<u8> {
        self.nmi_interrupt.take()
    }

    pub fn write_to_ctrl_register(&mut self, bits: u8) {
        let before_nmi_status = self.ctrl.generate_vblank_nmi();
        self.ctrl.update(bits);
        // trigger NMI if GENERATE_NMI bit in control register changes from 0 to 1 and the PPU is in VBLANK_STARTED state
        if !before_nmi_status && self.ctrl.generate_vblank_nmi() && self.status.is_in_vblank() {
            self.nmi_interrupt = Some(1);
        }
    }

    pub fn write_to_mask_register(&mut self, bits: u8) {
        self.mask.update(bits);
    }

    pub fn read_status_register(&mut self) -> u8 {
        let snapshot = self.status.get();
        self.status.clear_vblank_started();
        self.scroll.reset_latch();
        self.addr.reset_latch();
        snapshot
    }

    pub fn write_to_oam_addr_register(&mut self, addr: u8) {
        self.oam_addr.set(addr);
    }

    pub fn read_oam_data_register(&self) -> u8 {
        let addr = self.oam_addr.get();
        self.oam_data.read_data(addr)
    }

    pub fn write_to_oam_data_register(&mut self, value: u8) {
        let addr = self.oam_addr.get();
        self.oam_data.write_data(addr, value);
        self.oam_addr.increment_addr();
    }

    pub fn write_to_scroll_register(&mut self, value: u8) {
        self.scroll.write(value);
    }

    pub fn write_to_addr_register(&mut self, value: u8) {
        self.addr.update(value);
    }

    pub fn read_data_register(&mut self) -> u8 {
        let addr = self.addr.get();
        self.increment_vram_addr();
        self.data.read_data(addr)
    }

    pub fn write_to_data_register(&mut self, data: u8) {
        let addr = self.addr.get();
        self.data.write_data(addr, data);
        self.increment_vram_addr();
    }

    fn increment_vram_addr(&mut self) {
        self.addr
            .increment(self.ctrl.get_vram_addr_increment_value());
    }

    /// This function is a bit of a cheat, it's usually part of the register at 0x4014 called OAM Direct Memory Access Register.
    /// The actual OAM Data Register doesn't seem to be used by most games properly and they rather use this way to write data into memory.
    /// Preferably I would like to extract this into its own file and struct but I need the [`OamDataRegister`] internal memory.
    /// I'll keep it as TODO for now.
    pub fn write_to_oam_dma_register(&mut self, data: &[u8; 256]) {
        for value in data.iter() {
            let addr = self.oam_addr.get();
            self.oam_data.write_data(addr, *value);
            self.oam_addr.increment_addr();
        }
    }
}

#[cfg(test)]
pub mod test {
    use super::*;

    impl DataRegister {
        pub fn set_vram_at_address(&mut self, addr: u16, data: u8) {
            self.vram[addr as usize] = data;
        }

        pub fn get_vram_at_address(&self, addr: u16) -> u8 {
            self.vram[addr as usize]
        }
    }

    fn new_empty_rom() -> NesPPU {
        NesPPU::new(vec![0; 2048], Mirroring::HORIZONTAl)
    }

    #[test]
    fn test_ppu_vram_writes() {
        let mut ppu = new_empty_rom();
        ppu.write_to_addr_register(0x23);
        ppu.write_to_addr_register(0x05);
        ppu.write_to_data_register(0x66);

        assert_eq!(ppu.data.get_vram_at_address(0x0305), 0x66);
    }

    #[test]
    fn test_ppu_vram_reads() {
        let mut ppu = new_empty_rom();
        ppu.write_to_ctrl_register(0);
        ppu.data.set_vram_at_address(0x0305, 0x66);

        ppu.write_to_addr_register(0x23);
        ppu.write_to_addr_register(0x05);

        ppu.read_data_register(); //load_into_buffer
        assert_eq!(ppu.addr.get(), 0x2306);
        assert_eq!(ppu.read_data_register(), 0x66);
    }

    #[test]
    fn test_ppu_vram_reads_cross_page() {
        let mut ppu = new_empty_rom();
        ppu.write_to_ctrl_register(0);
        ppu.data.set_vram_at_address(0x01ff, 0x66);
        ppu.data.set_vram_at_address(0x0200, 0x77);

        ppu.write_to_addr_register(0x21);
        ppu.write_to_addr_register(0xff);

        ppu.read_data_register(); //load_into_buffer
        assert_eq!(ppu.read_data_register(), 0x66);
        assert_eq!(ppu.read_data_register(), 0x77);
    }

    #[test]
    fn test_ppu_vram_reads_step_32() {
        let mut ppu = new_empty_rom();
        ppu.write_to_ctrl_register(0b100);
        ppu.data.set_vram_at_address(0x01ff, 0x66);
        ppu.data.set_vram_at_address(0x01ff + 32, 0x77);
        ppu.data.set_vram_at_address(0x01ff + 64, 0x88);

        ppu.write_to_addr_register(0x21);
        ppu.write_to_addr_register(0xff);

        ppu.read_data_register(); //load_into_buffer
        assert_eq!(ppu.read_data_register(), 0x66);
        assert_eq!(ppu.read_data_register(), 0x77);
        assert_eq!(ppu.read_data_register(), 0x88);
    }

    // Horizontal: https://wiki.nesdev.com/w/index.php/Mirroring
    //   [0x2000 A ] [0x2400 a ]
    //   [0x2800 B ] [0x2C00 b ]
    #[test]
    fn test_vram_horizontal_mirror() {
        let mut ppu = new_empty_rom();
        ppu.write_to_addr_register(0x24);
        ppu.write_to_addr_register(0x05);

        ppu.write_to_data_register(0x66); //write to a

        ppu.write_to_addr_register(0x28);
        ppu.write_to_addr_register(0x05);

        ppu.write_to_data_register(0x77); //write to B

        ppu.write_to_addr_register(0x20);
        ppu.write_to_addr_register(0x05);

        ppu.read_data_register(); //load into buffer
        assert_eq!(ppu.read_data_register(), 0x66); //read from A

        ppu.write_to_addr_register(0x2C);
        ppu.write_to_addr_register(0x05);

        ppu.read_data_register(); //load into buffer
        assert_eq!(ppu.read_data_register(), 0x77); //read from b
    }

    // Vertical: https://wiki.nesdev.com/w/index.php/Mirroring
    //   [0x2000 A ] [0x2400 B ]
    //   [0x2800 a ] [0x2C00 b ]
    #[test]
    fn test_vram_vertical_mirror() {
        let mut ppu = NesPPU::new(vec![0; 2048], Mirroring::VERTICAL);

        ppu.write_to_addr_register(0x20);
        ppu.write_to_addr_register(0x05);

        ppu.write_to_data_register(0x66); //write to A

        ppu.write_to_addr_register(0x2C);
        ppu.write_to_addr_register(0x05);

        ppu.write_to_data_register(0x77); //write to b

        ppu.write_to_addr_register(0x28);
        ppu.write_to_addr_register(0x05);

        ppu.read_data_register(); //load into buffer
        assert_eq!(ppu.read_data_register(), 0x66); //read from a

        ppu.write_to_addr_register(0x24);
        ppu.write_to_addr_register(0x05);

        ppu.read_data_register(); //load into buffer
        assert_eq!(ppu.read_data_register(), 0x77); //read from B
    }

    #[test]
    fn test_read_status_resets_latch() {
        let mut ppu = new_empty_rom();
        ppu.data.set_vram_at_address(0x0305, 0x66);

        ppu.write_to_addr_register(0x21);
        ppu.write_to_addr_register(0x23);
        ppu.write_to_addr_register(0x05);

        ppu.read_data_register(); //load_into_buffer
        assert_ne!(ppu.read_data_register(), 0x66);

        ppu.read_status_register();

        ppu.write_to_addr_register(0x23);
        ppu.write_to_addr_register(0x05);

        ppu.read_data_register(); //load_into_buffer
        assert_eq!(ppu.read_data_register(), 0x66);
    }

    #[test]
    fn test_ppu_vram_mirroring() {
        let mut ppu = new_empty_rom();
        ppu.write_to_ctrl_register(0);
        ppu.data.set_vram_at_address(0x0305, 0x66);

        ppu.write_to_addr_register(0x63); //0x6305 -> 0x2305
        ppu.write_to_addr_register(0x05);

        ppu.read_data_register(); //load into_buffer
        assert_eq!(ppu.read_data_register(), 0x66);
    }

    #[test]
    fn test_read_status_resets_vblank() {
        let mut ppu = new_empty_rom();
        ppu.status.set_vblank_started();

        let status = ppu.read_status_register();

        assert_eq!(status >> 7, 1);
        assert_eq!(ppu.status.get() >> 7, 0);
    }

    #[test]
    fn test_oam_read_write() {
        let mut ppu = new_empty_rom();
        ppu.write_to_oam_addr_register(0x10);
        ppu.write_to_oam_data_register(0x66);
        ppu.write_to_oam_data_register(0x77);

        ppu.write_to_oam_addr_register(0x10);
        assert_eq!(ppu.read_oam_data_register(), 0x66);

        ppu.write_to_oam_addr_register(0x11);
        assert_eq!(ppu.read_oam_data_register(), 0x77);
    }

    #[test]
    fn test_oam_dma() {
        let mut ppu = new_empty_rom();

        let mut data = [0x66; 256];
        data[0] = 0x77;
        data[255] = 0x88;

        ppu.write_to_oam_addr_register(0x10);
        ppu.write_to_oam_dma_register(&data);

        ppu.write_to_oam_addr_register(0xf); //wrap around
        assert_eq!(ppu.read_oam_data_register(), 0x88);

        ppu.write_to_oam_addr_register(0x10);
        assert_eq!(ppu.read_oam_data_register(), 0x77);

        ppu.write_to_oam_addr_register(0x11);
        assert_eq!(ppu.read_oam_data_register(), 0x66);
    }
}
