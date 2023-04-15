use crate::ppu::NesPPU;

use self::{frame::Frame, palette::SYSTEM_PALETE};

pub mod frame;
pub mod palette;

pub fn render(ppu: &NesPPU, frame: &mut Frame) {
    render_background(ppu, frame);
    render_sprites(ppu, frame);
}

fn render_background(ppu: &NesPPU, frame: &mut Frame) {
    let bank = ppu.ctrl.get_background_pattern_addr();

    for i in 0..0x03C0 {
        // for now only first name table (0x03c0 = 960 which is the length of 1 nametable and the number of tiles in a NES background screen)
        // a name table has 32 columns and 30 rows
        let tile_column = i % 32;
        let tile_row = i / 32;
        let start = (bank + ppu.data.vram[i] as u16 * 16) as usize;
        let tile = ppu.data.return_chr_rom_slice(start, start + 15);
        let palette = bg_palette(ppu, tile_column, tile_row);

        // a tile is described using 16 bytes and each row is encoded using 2 bytes that stand 8 byte apart
        // to calculate the color index of the top left pixel you read the 7th bit of 0x0000 (left) = 0
        // and the 7th bit of 0x0008 (right) = 1 and then combine them: (right 7th bit)(left 7th bit) = 10
        // then repeat analog for next pixel (i.e. go to the 6th bit)
        for y in 0..=7 {
            let mut left = tile[y];
            let mut right = tile[y + 8];

            for x in (0..=7).rev() {
                let value = (1 & right) << 1 | (1 & left);
                right >>= 1;
                left >>= 1;

                // TODO: colors are currently just picked by me and not specified using the palette_table
                let rgb = match value {
                    0 => SYSTEM_PALETE[palette[0] as usize],
                    1 => SYSTEM_PALETE[palette[1] as usize],
                    2 => SYSTEM_PALETE[palette[2] as usize],
                    3 => SYSTEM_PALETE[palette[3] as usize],
                    _ => panic!("Theoretically unreachable code was reached"),
                };

                frame.set_pixel(tile_column * 8 + x, tile_row * 8 + y, rgb);
            }
        }
    }
}

fn render_sprites(ppu: &NesPPU, frame: &mut Frame) {
    for i in (0..ppu.oam_data.memory.len()).step_by(4).rev() {
        let tile_idx = ppu.oam_data.memory[i + 1];
        let tile_x = ppu.oam_data.memory[i + 3] as usize;
        let tile_y = ppu.oam_data.memory[i] as usize;

        let flip_vertical = ppu.oam_data.memory[i + 2] >> 7 & 1 == 1;
        let flip_horizontal = ppu.oam_data.memory[i + 2] >> 6 & 1 == 1;
        let palette_idx = ppu.oam_data.memory[i + 2] & 0b11;
        let sprite_palette = sprite_palette(ppu, palette_idx);

        let bank = ppu.ctrl.get_sprite_pattern_addr();
        let start = (bank + tile_idx as u16 * 16) as usize;

        let tile = &ppu.data.chr_rom[start..=(start + 15)];

        for y in 0..=7 {
            let mut left = tile[y];
            let mut right = tile[y + 8];

            'inner: for x in (0..=7).rev() {
                let value = (1 & right) << 1 | (1 & left);
                left >>= 1;
                right >>= 1;

                let rgb = match value {
                    0 => continue 'inner,
                    1 => SYSTEM_PALETE[sprite_palette[1] as usize],
                    2 => SYSTEM_PALETE[sprite_palette[2] as usize],
                    3 => SYSTEM_PALETE[sprite_palette[3] as usize],
                    _ => panic!("Theoretically unreachable code was reached"),
                };

                match (flip_horizontal, flip_vertical) {
                    (false, false) => frame.set_pixel(tile_x + x, tile_y + y, rgb),
                    (true, false) => frame.set_pixel(tile_x + 7 - x, tile_y + y, rgb),
                    (false, true) => frame.set_pixel(tile_x + x, tile_y + 7 - y, rgb),
                    (true, true) => frame.set_pixel(tile_x + 7 - x, tile_y + 7 - y, rgb),
                }
            }
        }
    }
}

fn sprite_palette(ppu: &NesPPU, palette_idx: u8) -> [u8; 4] {
    let start = 0x11 + (palette_idx * 4) as usize;
    [
        0,
        ppu.data.palette_table[start],
        ppu.data.palette_table[start + 1],
        ppu.data.palette_table[start + 2],
    ]
}

/// 1 byte in the attribute table controls the palettes of 4 neighboring meta-tiles.
/// 1 meta tile is a space composed of 2x2 tiles which means that this 1 byte controls which palettes are used for 4x4 tile blocks (32x32 pixels).
/// Each byte is split into four 2-bit blocks and each block assigns a background palette for 4 neighboring tiles.
/// Returns the colors of that palette. Each palette contains 3 colors, but 0b00 means transparency.
///
/// Imagine like this:
/// This is a meta tile represented by this byte: `0b11_01_10_00` which is blue, green, yellow, red.
/// Colors represent which color this tile is rendered as.
/// ```
///  ___________________________________
/// |  red   |  red   | yellow | yellow |
/// |  red   |  red   | yellow | yellow |
/// |_________________|_________________|
/// | green  | green  |  blue  |  blue  |
/// | green  | green  |  blue  |  blue  |
/// |_________________|_________________|
/// ```
fn bg_palette(ppu: &NesPPU, tile_column: usize, tile_row: usize) -> [u8; 4] {
    let attr_table_idx = tile_row / 4 * 8 + tile_column / 4;
    let attr_byte = ppu.data.vram[0x3c0 + attr_table_idx]; // TODO: 0x3c0 is still hardcoded to the first nametable

    // here we actually calculate which part of the byte is important for us
    let palette_idx = match (tile_column % 4 / 2, tile_row % 4 / 2) {
        (0, 0) => attr_byte,
        (1, 0) => attr_byte >> 2,
        (0, 1) => attr_byte >> 4,
        (1, 1) => attr_byte >> 6,
        (_, _) => panic!("Theoretically unreachable code has been reached"),
    } & 0b11;

    let palette_start = 1 + palette_idx as usize * 4;

    [
        ppu.data.palette_table[0],
        ppu.data.palette_table[palette_start],
        ppu.data.palette_table[palette_start + 1],
        ppu.data.palette_table[palette_start + 2],
    ]
}
