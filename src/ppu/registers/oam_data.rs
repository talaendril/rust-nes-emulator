pub struct OamDataRegister {
    oam_data: [u8; 256], // called Object Attribute Memory, keeps sprite state
}

impl OamDataRegister {
    pub fn new() -> Self {
        OamDataRegister { oam_data: [0; 256] }
    }

    pub fn read_data(&self, addr: u8) -> u8 {
        self.oam_data[addr as usize]
    }

    pub fn write_data(&mut self, addr: u8, data: u8) {
        self.oam_data[addr as usize] = data;
    }
}
