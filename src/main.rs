use minifb::{Key, KeyRepeat, Scale, Window, WindowOptions};
mod chip8;
use chip8::Chip8;
use std::io::Error;
use std::path::Path;

const WIDTH: usize = 64;
const HEIGHT: usize = 32;

fn main() -> Result<(), Error> {
    let mut buffer: [u32; WIDTH * HEIGHT] = [0; WIDTH * HEIGHT];

    let mut window_options = WindowOptions::default();
    window_options.scale = Scale::X8;

    let mut window = Window::new("CHIP-8 Interpreter", WIDTH, HEIGHT, window_options)
        .unwrap_or_else(|e| {
            panic!("{}", e);
        });

    // Limit to max ~60 fps update rate
    //window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

    let mut chip8 = Chip8::new();
    chip8.load_program(Path::new("roms/Tetris [Fran Dachille, 1991].ch8"))?;

    while window.is_open() && !window.is_key_down(Key::Escape) {
        window.update();
        let pressed_keys = window.get_keys_pressed(KeyRepeat::No).unwrap();
        for key in pressed_keys.iter() {
            if let Some(key) = to_hexadecimal_keypad(*key) {
                chip8.keys[key as usize] = true;
            }
        }

        chip8.execute_cycle();
        if chip8.draw {
            for (i, pixel) in chip8.framebuffer.iter().enumerate() {
                buffer[i] = to_0rgb(pixel);
            }
        }

        // We unwrap here as we want this code to exit if it fails. Real applications may want to handle this in a different way
        window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();
    }

    Ok(())
}

// QWERTY only for now.
fn to_hexadecimal_keypad(key: minifb::Key) -> Option<u8> {
    match key {
        Key::Key1 => Some(1),
        Key::Key2 => Some(2),
        Key::Key3 => Some(3),
        Key::Key4 => Some(0xC),
        Key::Q => Some(4),
        Key::W => Some(5),
        Key::E => Some(6),
        Key::R => Some(0xD),
        Key::A => Some(7),
        Key::S => Some(8),
        Key::D => Some(9),
        Key::F => Some(0xE),
        Key::Z => Some(0xA),
        Key::X => Some(0),
        Key::C => Some(0xB),
        Key::V => Some(0xF),
        _ => None,
    }
}

fn to_0rgb(pixel: &u8) -> u32 {
    if *pixel == 1 {
        return 0x00FFFFFF;
    } else {
        return 0;
    }
}
