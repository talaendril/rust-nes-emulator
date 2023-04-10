use bitflags::bitflags;

bitflags! {
    // 7  bit  0
    // ---- ----
    // BGRs bMmG
    // |||| ||||
    // |||| |||+- Greyscale (0: normal color, 1: produce a greyscale display)
    // |||| ||+-- 1: Show background in leftmost 8 pixels of screen, 0: Hide
    // |||| |+--- 1: Show sprites in leftmost 8 pixels of screen, 0: Hide
    // |||| +---- 1: Show background
    // |||+------ 1: Show sprites
    // ||+------- Emphasize red (green on PAL/Dendy)
    // |+-------- Emphasize green (red on PAL/Dendy)
    // +--------- Emphasize blue
    struct MaskRegisterFlags: u8 {
        const GREYSCALE             = 0b0000_0001;
        const LEFTMOST_BACKGROUND   = 0b0000_0010;
        const LEFTMOST_SPRITE       = 0b0000_0100;
        const SHOW_BACKGROUND       = 0b0000_1000;
        const SHOW_SPRITES          = 0b0001_0000;
        const EMPHASIZE_RED         = 0b0010_0000;
        const EMPHASIZE_GREEN       = 0b0100_0000;
        const EMPHASIZE_BLUE        = 0b1000_0000;
    }
}

pub enum Color {
    Red,
    Green,
    Blue,
}

pub struct MaskRegister {
    status: MaskRegisterFlags,
}

impl MaskRegister {
    pub fn new() -> Self {
        MaskRegister {
            status: MaskRegisterFlags::from_bits_truncate(0b00000000),
        }
    }

    // pub fn is_greyscale(&self) -> bool {
    //     self.status.contains(MaskRegisterFlags::GREYSCALE)
    // }

    // pub fn show_leftmost_8pxl_background(&self) -> bool {
    //     self.status.contains(MaskRegisterFlags::LEFTMOST_BACKGROUND)
    // }

    // pub fn show_leftmost_8pxl_sprites(&self) -> bool {
    //     self.status.contains(MaskRegisterFlags::LEFTMOST_SPRITE)
    // }

    // pub fn show_background(&self) -> bool {
    //     self.status.contains(MaskRegisterFlags::SHOW_BACKGROUND)
    // }

    // pub fn show_sprites(&self) -> bool {
    //     self.status.contains(MaskRegisterFlags::SHOW_SPRITES)
    // }

    // pub fn get_emphasized_color(&self) -> Vec<Color> {
    //     let mut result: Vec<Color> = vec![];

    //     if self.status.contains(MaskRegisterFlags::EMPHASIZE_RED) {
    //         result.push(Color::Red);
    //     }

    //     if self.status.contains(MaskRegisterFlags::EMPHASIZE_GREEN) {
    //         result.push(Color::Green);
    //     }

    //     if self.status.contains(MaskRegisterFlags::EMPHASIZE_BLUE) {
    //         result.push(Color::Blue);
    //     }

    //     result
    // }

    pub fn update(&mut self, data: u8) {
        self.status = MaskRegisterFlags::from_bits_truncate(data);
    }
}
