use bus::Bus;
use cartridge::Rom;
use cpu::{Mem, CPU};
use rand::Rng;
use sdl2::{
    event::Event,
    keyboard::Keycode,
    pixels::{Color, PixelFormatEnum},
    EventPump,
};

use trace::trace;

mod bus;
mod cartridge;
mod cpu;
mod opcode;
mod trace;

fn main() {
    // init sdl2
    // let sdl_context = sdl2::init().unwrap();
    // let video_subsystem = sdl_context.video().unwrap();
    // let window = video_subsystem
    //     .window("Snake game", (32.0 * 10.0) as u32, (32.0 * 10.0) as u32)
    //     .position_centered()
    //     .build()
    //     .unwrap();

    // let mut canvas = window.into_canvas().present_vsync().build().unwrap();
    // let mut event_pump = sdl_context.event_pump().unwrap();
    // canvas.set_scale(10.0, 10.0).unwrap();

    // let texture_creator = canvas.texture_creator();
    // let mut texture = texture_creator
    //     .create_texture_target(PixelFormatEnum::RGB24, 32, 32)
    //     .unwrap();

    /*
    TODO:
    it seems like there are too many calls happenining between LDA calls (0xa5, 0xff; where 0xff contains the input)
    I don't know if this is by design and my rust just runs slow because it is executed on windows
    or if it's because of an error inside my instructions that cause errors (program_counter errors maybe?)
    What makes me think that it is a combination of both is the fact that I have run the guide's code
    and while it was faster it wasnt as fast as shown in the gif.
     */
    let bytes = std::fs::read("nestest.nes").unwrap();
    let rom = Rom::new(&bytes).unwrap();
    let bus = Bus::new(rom);

    let mut cpu = CPU::new(bus);
    cpu.reset();
    cpu.program_counter = 0xC000;

    // let mut screen_state = [0_u8; 32 * 3 * 32];
    // let mut rng = rand::thread_rng();

    // I think now that cpu is moved it should be possible to use again afterwards
    // but I am not sure
    cpu.run_with_callback(move |cpu| {
        println!("{}", trace(cpu));
        // handle_user_input(cpu, &mut event_pump);

        // cpu.mem_write(0xfe, rng.gen_range(1..16));

        // if read_screen_state(cpu, &mut screen_state) {
        //     texture.update(None, &screen_state, 32 * 3).unwrap();
        //     canvas.copy(&texture, None, None).unwrap();
        //     canvas.present();
        // }

        // ::std::thread::sleep(std::time::Duration::new(0, 70_000));
    });
}

fn read_screen_state(cpu: &CPU, frame: &mut [u8; 32 * 3 * 32]) -> bool {
    let mut frame_idx = 0;
    let mut update = false;
    for i in 0x0200..0x600 {
        let color_idx = cpu.mem_read(i);
        let (b1, b2, b3) = color(color_idx).rgb();
        if frame[frame_idx] != b1 || frame[frame_idx + 1] != b2 || frame[frame_idx + 2] != b3 {
            frame[frame_idx] = b1;
            frame[frame_idx + 1] = b2;
            frame[frame_idx + 2] = b3;
            update = true;
        }
        frame_idx += 3;
    }

    update
}

fn handle_user_input(cpu: &mut CPU, event_pump: &mut EventPump) {
    let input_address = 0xff;
    for event in event_pump.poll_iter() {
        match event {
            Event::Quit { .. }
            | Event::KeyDown {
                keycode: Some(Keycode::Escape),
                ..
            } => std::process::exit(0),
            Event::KeyDown {
                keycode: Some(Keycode::W),
                ..
            } => cpu.mem_write(input_address, 0x77),
            Event::KeyDown {
                keycode: Some(Keycode::S),
                ..
            } => cpu.mem_write(input_address, 0x73),
            Event::KeyDown {
                keycode: Some(Keycode::A),
                ..
            } => cpu.mem_write(input_address, 0x61),
            Event::KeyDown {
                keycode: Some(Keycode::D),
                ..
            } => cpu.mem_write(input_address, 0x64),
            _ => { /* do nothing */ }
        }
    }
}

fn color(byte: u8) -> Color {
    match byte {
        0 => sdl2::pixels::Color::BLACK,
        1 => sdl2::pixels::Color::WHITE,
        2 | 9 => sdl2::pixels::Color::GREY,
        3 | 10 => sdl2::pixels::Color::RED,
        4 | 11 => sdl2::pixels::Color::GREEN,
        5 | 12 => sdl2::pixels::Color::BLUE,
        6 | 13 => sdl2::pixels::Color::MAGENTA,
        7 | 14 => sdl2::pixels::Color::YELLOW,
        _ => sdl2::pixels::Color::CYAN,
    }
}
