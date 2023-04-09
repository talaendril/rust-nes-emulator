pub struct ScrollRegister {
    horizontal_offset: u8,
    vertical_offset: u8,
    vertical: bool, // look into [`AddrRegister`] for more information
}

impl ScrollRegister {
    pub fn new() -> Self {
        ScrollRegister {
            horizontal_offset: 0,
            vertical_offset: 0,
            vertical: true,
        }
    }

    pub fn write(&mut self, offset: u8) {
        if self.vertical {
            self.vertical_offset = offset;
        } else {
            self.horizontal_offset = offset;
        }

        self.vertical = !self.vertical;
    }

    pub fn reset_latch(&mut self) {
        self.vertical = true;
    }
}
