pub struct OamAddressRegister {
    oam_addr: u8,
}

impl OamAddressRegister {
    pub fn new() -> Self {
        OamAddressRegister { oam_addr: 0 }
    }

    pub fn set(&mut self, addr: u8) {
        self.oam_addr = addr;
    }

    pub fn get(&self) -> u8 {
        self.oam_addr
    }

    pub fn increment_addr(&mut self) {
        self.oam_addr = self.oam_addr.wrapping_add(1);
    }
}
