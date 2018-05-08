use sdl2::ttf;
use sdl2::rect::Rect;
use sdl2::render::{TextureCreator, WindowCanvas};
use sdl2::pixels::Color;
use sdl2::video::WindowContext;

use emu::system::System;
use emu::cpu::Register8;

pub struct Debug<'a> {
    font: ttf::Font<'a, 'static>,
}

impl<'a> Debug<'a> {
    pub fn new(ctx: &ttf::Sdl2TtfContext) -> Result<Debug, String> {
        Ok(Debug {
            font: ctx.load_font("MOZART_0.ttf", 24)
                .map_err(|e| e.to_string())?,
        })
    }

    pub fn draw(
        &mut self,
        canvas: &mut WindowCanvas,
        texture_creator: &TextureCreator<WindowContext>,
        system: &System,
    ) -> Result<(), String> {
        let line_spacing = self.font.height() + 4;
        let column = 1000;
        self.draw_line(
            canvas,
            texture_creator,
            "Registers:",
            (column, 0 * line_spacing),
        )?;
        self.draw_line(
            canvas,
            texture_creator,
            &format!(
                " A: 0x{:02x}   F: 0x{:02x}    SP: {}",
                system.cpu[Register8::A],
                system.cpu[Register8::F],
                system.cpu.sp
            ),
            (column, 1 * line_spacing),
        )?;
        self.draw_line(
            canvas,
            texture_creator,
            &format!(
                " B: 0x{:02x}   C: 0x{:02x}    PC: {}",
                system.cpu[Register8::B],
                system.cpu[Register8::C],
                system.cpu.pc
            ),
            (column, 2 * line_spacing),
        )?;
        self.draw_line(
            canvas,
            texture_creator,
            &format!(
                " D: 0x{:02x}   E: 0x{:02x}   IME: {}",
                system.cpu[Register8::D],
                system.cpu[Register8::E],
                system.cpu.interrupt_master_enable
            ),
            (column, 3 * line_spacing),
        )?;
        self.draw_line(
            canvas,
            texture_creator,
            &format!(
                " H: 0x{:02x}   L: 0x{:02x}",
                system.cpu[Register8::H],
                system.cpu[Register8::L]
            ),
            (column, 4 * line_spacing),
        )?;

        Ok(())
    }

    fn draw_line(
        &mut self,
        canvas: &mut WindowCanvas,
        texture_creator: &TextureCreator<WindowContext>,
        line: &str,
        (x, y): (i32, i32),
    ) -> Result<(), String> {
        let s = self.font
            .render(line)
            .solid(Color::RGB(255, 255, 255))
            .map_err(|e| e.to_string())?;
        let w = s.width();
        let h = s.height();
        let t = texture_creator
            .create_texture_from_surface(s)
            .map_err(|e| e.to_string())?;
        let target = Rect::new(x, y, w, h);
        canvas.copy(&t, None, target).map_err(|e| e.to_string())
    }
}
