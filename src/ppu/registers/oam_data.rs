/// Note: a few attributes are marked with `pub` visiblity.
/// This is because the emulator needs to intercept the program execution in order to properly draw the screen.
pub struct OamDataRegister {
    pub memory: [u8; 256], // called Object Attribute Memory, keeps sprite state
}

impl OamDataRegister {
    pub fn new() -> Self {
        OamDataRegister { memory: [0; 256] }
    }

    pub fn read_data(&self, addr: u8) -> u8 {
        self.memory[addr as usize]
    }

    pub fn write_data(&mut self, addr: u8, data: u8) {
        self.memory[addr as usize] = data;
    }
}
