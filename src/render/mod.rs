use crate::ppu::NesPPU;

use self::{frame::Frame, palette::SYSTEM_PALETE};

pub mod frame;
pub mod palette;

pub fn render(ppu: &NesPPU, frame: &mut Frame) {
    let bank = ppu.ctrl.get_background_pattern_addr();

    for i in 0..0x03C0 {
        // for now only first name table (0x03c0 = 960 which is the length of 1 nametable and the number of tiles in a NES background screen)
        // a name table has 32 columns and 30 rows
        let tile_x = i % 32; // column in name table
        let tile_y = i / 32; // row in name table?
        let start = (bank + ppu.data.vram[i] as u16 * 16) as usize;
        let tile = ppu.data.return_chr_rom_slice(start, start + 15);

        // a tile is described using 16 bytes and each row is encoded using 2 bytes that stand 8 byte apart
        // to calculate the color index of the top left pixel you read the 7th bit of 0x0000 (left) = 0
        // and the 7th bit of 0x0008 (right) = 1 and then combine them: (right 7th bit)(left 7th bit) = 10
        // then repeat analog for next pixel (i.e. go to the 6th bit)
        for y in 0..=7 {
            let mut left = tile[y];
            let mut right = tile[y + 8];

            for x in (0..=7).rev() {
                // TODO: I have adjusted this to fit with the explanation instead of the guide, the guide has right and left flipped around
                let value = (1 & right) << 1 | (1 & left);
                right >>= 1;
                left >>= 1;

                // TODO: colors are currently just picked by me and not specified using the palette_table
                let rgb = match value {
                    0 => SYSTEM_PALETE[0x01],
                    1 => SYSTEM_PALETE[0x23],
                    2 => SYSTEM_PALETE[0x27],
                    3 => SYSTEM_PALETE[0x30],
                    _ => panic!("Theoretically unreachable code was reached"),
                };

                frame.set_pixel(tile_x * 8 + x, tile_y * 8 + y, rgb);
            }
        }
    }
}
