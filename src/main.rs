use minifb::{Key, Window, WindowOptions, Scale};

const WIDTH: usize = 64;
const HEIGHT: usize = 32;

fn main() {
    let mut buffer: [u32; WIDTH * HEIGHT] = [0; WIDTH * HEIGHT];

    let mut window_options = WindowOptions::default();
    window_options.scale = Scale::X4;

    let mut window = Window::new(
        "CHIP-8 Interpreter",
        WIDTH,
        HEIGHT,
        window_options,
    )
    .unwrap_or_else(|e| {
        panic!("{}", e);
    });

    // Limit to max ~60 fps update rate
    window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

    while window.is_open() && !window.is_key_down(Key::Escape) {
        for i in buffer.iter_mut() {
            *i = 0; // write something more funny here!
        }

        // We unwrap here as we want this code to exit if it fails. Real applications may want to handle this in a different way
        window
            .update_with_buffer(&buffer, WIDTH, HEIGHT)
            .unwrap();
    }
}
