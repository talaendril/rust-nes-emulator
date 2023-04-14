use crate::render::{frame::Frame, palette::SYSTEM_PALETE};

fn show_tile(chr_rom: &[u8], bank: usize, tile_n: usize) -> Frame {
    assert!(bank <= 1);

    let mut frame = Frame::new();
    let bank = bank * 0x1000; // 0x1000 = 4096 in decimal

    let start = bank + tile_n * 16;
    let tile = &chr_rom[start..=(start + 15)];

    // a tile is described using 16 bytes and each row is encoded using 2 bytes that stand 8 byte apart
    // to calculate the color index of the top left pixel you read the 7th bit of 0x0000 (left) = 0
    // and the 7th bit of 0x0008 (right) = 1 and then combine them: (right 7th bit)(left 7th bit) = 10
    // then repeat analog for next pixel (i.e. go to the 6th bit)
    for y in 0..=7 {
        let mut left = tile[y]; // 0x0000
        let mut right = tile[y + 8]; // 0x0008

        for x in (0..=7).rev() {
            // TODO: I have adjusted this to fit with the explanation instead of the guide
            // the guide has right and left flipped around
            let value = (1 & right) << 1 | (1 & left);
            left >>= 1;
            right >>= 1;

            // TODO: colors are currently just picked by me and not specified using the palette_table
            let rgb = match value {
                0 => SYSTEM_PALETE[0x01],
                1 => SYSTEM_PALETE[0x23],
                2 => SYSTEM_PALETE[0x27],
                3 => SYSTEM_PALETE[0x30],
                _ => panic!("Theoretically unreachable code was reached"),
            };
            frame.set_pixel(x, y, rgb);
        }
    }

    frame
}
