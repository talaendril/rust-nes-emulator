pub struct AddrRegister {
    hi: u8,
    lo: u8,
    hi_ptr: bool,
}

impl AddrRegister {
    pub fn new() -> Self {
        AddrRegister {
            hi: 0,
            lo: 0,
            hi_ptr: true,
        }
    }

    fn set(&mut self, data: u16) {
        let [lo, hi] = data.to_le_bytes();
        self.hi = hi;
        self.lo = lo;
    }

    pub fn update(&mut self, data: u8) {
        if self.hi_ptr {
            self.hi = data;
        } else {
            self.lo = data;
        }

        if self.get() > 0x3fff {
            // mirror down addr above 0x3fff
            self.set(self.get() & 0b0011_1111_1111_1111);
        }
        self.hi_ptr = !self.hi_ptr;
    }

    pub fn get(&self) -> u16 {
        u16::from_le_bytes([self.lo, self.hi])
    }

    pub fn increment(&mut self, inc: u8) {
        let old_lo = self.lo;
        self.lo = self.lo.wrapping_add(inc);

        if old_lo > self.lo {
            self.hi = self.hi.wrapping_add(1);
        }

        if self.get() > 0x3fff {
            // mirror down addr above 0x3fff
            self.set(self.get() & 0b0011_1111_1111_1111);
        }
    }

    pub fn reset_latch(&mut self) {
        self.hi_ptr = true;
    }
}
