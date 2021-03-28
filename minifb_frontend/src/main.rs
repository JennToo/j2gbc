use minifb::{Key, Scale, ScaleMode, Window, WindowOptions};

use j2gbc::SCREEN_SIZE;

fn main() {
    let buffer: Vec<u32> = vec![255; SCREEN_SIZE.0 * SCREEN_SIZE.1];
    let options = WindowOptions {
        borderless: false,
        title: true,
        resize: true,
        scale: Scale::FitScreen,
        scale_mode: ScaleMode::AspectRatioStretch,
        topmost: false,
        transparency: false,
        none: false,
    };

    let mut window = Window::new("j2gbc", SCREEN_SIZE.0, SCREEN_SIZE.1, options).unwrap();

    window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

    while window.is_open() && !window.is_key_down(Key::Escape) {
        window
            .update_with_buffer(&buffer, SCREEN_SIZE.0, SCREEN_SIZE.1)
            .unwrap();
    }
}
