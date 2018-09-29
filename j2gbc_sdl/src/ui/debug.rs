use std;
use std::io::Write;
use std::sync::Mutex;

use log::{set_logger, set_max_level, LevelFilter, Log, Metadata, Record};
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::{TextureCreator, WindowCanvas};
use sdl2::ttf;
use sdl2::video::WindowContext;

use j2gbc::cpu::Register8;
use j2gbc::cpu::{Cpu, Interrupt};
use j2gbc::mem::{Address, MemDevice};
use j2gbc::system::System;

pub struct Debug<'a> {
    font: ttf::Font<'a, 'static>,
    console_scollback: usize,
    command_buffer: String,
    prev_command_buffer: String,
}

impl<'a> Debug<'a> {
    pub fn new(ctx: &ttf::Sdl2TtfContext) -> Result<Debug, String> {
        Ok(Debug {
            font: ctx
                .load_font("j2gbc_sdl/MOZART_0.ttf", 24)
                .map_err(|e| e.to_string())?,
            console_scollback: 0,
            command_buffer: String::new(),
            prev_command_buffer: String::new(),
        })
    }

    pub fn command_keystroke(&mut self, s: &str) {
        self.command_buffer.push_str(s);
    }

    pub fn command_backspace(&mut self) {
        self.command_buffer.pop();
    }

    pub fn scroll_up(&mut self, n: usize) {
        self.console_scollback += n;
    }

    pub fn scroll_down(&mut self, n: usize) {
        if self.console_scollback >= n {
            self.console_scollback -= n;
        } else {
            self.console_scollback = 0;
        }
    }

    pub fn draw(
        &mut self,
        canvas: &mut WindowCanvas,
        texture_creator: &TextureCreator<WindowContext>,
        system: &System,
    ) -> Result<(), String> {
        self.draw_regs(canvas, texture_creator, system)?;
        self.draw_command_buffer(canvas, texture_creator)?;
        self.draw_console(canvas, texture_creator)
    }

    fn draw_command_buffer(
        &mut self,
        canvas: &mut WindowCanvas,
        texture_creator: &TextureCreator<WindowContext>,
    ) -> Result<(), String> {
        let line_spacing = self.font.height() + 4;
        let column = 1000;
        let y = canvas.output_size()?.1 as i32 - (4 * line_spacing);
        let s = self.command_buffer.clone();
        self.draw_line(canvas, texture_creator, &format!("> {}", s), (column, y))
    }

    fn draw_console(
        &mut self,
        canvas: &mut WindowCanvas,
        texture_creator: &TextureCreator<WindowContext>,
    ) -> Result<(), String> {
        let line_spacing = self.font.height() + 4;
        let column = 1000;
        let min_y = 5 * line_spacing;
        let max_y = canvas.output_size()?.1 as i32 - (5 * line_spacing);
        let mut cur_y = max_y;

        let l = DEBUG_LOGGER.log.lock().unwrap();

        for s in l.iter().rev().skip(self.console_scollback) {
            self.draw_line(canvas, texture_creator, s, (column, cur_y))?;
            cur_y -= line_spacing;
            if cur_y < min_y {
                break;
            }
        }

        Ok(())
    }

    fn draw_regs(
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
        if line.len() == 0 {
            return Ok(());
        }
        let s = self
            .font
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

    pub fn start_debugging(&mut self, system: &mut System) {
        for &(a, i) in &system.cpu.last_instructions {
            info!("{}: {}", a, i);
        }
        self.print_next_instruction(&mut system.cpu);
    }

    pub fn run_command(&mut self, system: &mut System) {
        self.console_scollback = 0;
        if self.command_buffer == "" {
            self.command_buffer = self.prev_command_buffer.clone();
        }
        let mut pieces: Vec<String> = self.command_buffer.split(' ').map(String::from).collect();
        let cmd = pieces.remove(0);
        info!("> {}", self.command_buffer);
        self.execute_command(&cmd, &pieces, &mut system.cpu);
        self.prev_command_buffer = self.command_buffer.clone();
        self.command_buffer = String::new();
    }

    fn execute_command(&mut self, cmd: &str, args: &[String], cpu: &mut Cpu) -> bool {
        match cmd {
            "exit" => std::process::exit(0),
            "c" => cpu.debug_halted = false,
            "s" => {
                let _ret = cpu.run_cycle();
                self.print_next_instruction(cpu);
            }
            "sf" => {
                cpu.interrupt_breakpoints.insert(Interrupt::VBlank);
                cpu.debug_halted = false;
            }
            "w" => {
                let address = Address(u16::from_str_radix(args[0].as_str(), 16).unwrap());
                cpu.mmu.watchpoints.insert(address);
            }
            "uw" => {
                let address = Address(u16::from_str_radix(args[0].as_str(), 16).unwrap());
                cpu.mmu.watchpoints.remove(&address);
            }
            "b" => {
                let address = Address(u16::from_str_radix(args[0].as_str(), 16).unwrap());
                cpu.breakpoints.insert(address);
            }
            "peek" => {
                let address = Address(u16::from_str_radix(args[0].as_str(), 16).unwrap());
                info!("{}: {:?}", address, cpu.mmu.read(address));
            }
            "poke" => {
                let address = Address(u16::from_str_radix(args[0].as_str(), 16).unwrap());
                let v = u8::from_str_radix(args[1].as_str(), 16).unwrap();
                info!("{}: {:?}", address, cpu.mmu.write(address, v));
            }
            _ => info!("Unrecognized command: {}", cmd),
        }

        true
    }

    fn print_next_instruction(&mut self, cpu: &mut Cpu) {
        match cpu.fetch_instruction() {
            Result::Ok((i, _)) => info!(" => {}: {}", cpu.pc, i),
            Result::Err(()) => info!("    FAILED TO READ NEXT INSTRUCTION"),
        }
    }
}

struct DebugLogger {
    log: Mutex<Vec<String>>,
    event_file: Mutex<std::fs::File>,
    started: std::time::Instant,
}

lazy_static! {
    static ref DEBUG_LOGGER: DebugLogger = {
        set_max_level(LevelFilter::Debug);
        DebugLogger {
            log: Mutex::new(Vec::new()),
            event_file: Mutex::new(
                std::fs::File::create("events.csv").expect("Failed to open event file"),
            ),
            started: std::time::Instant::now(),
        }
    };
}

impl Log for DebugLogger {
    fn enabled(&self, _: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        let r = record.args().to_string();

        let timestamp = self.started.elapsed();
        write!(
            *self.event_file.lock().unwrap(),
            "{}:{},{}\r\n",
            timestamp.as_secs(),
            timestamp.subsec_nanos(),
            r,
        ).expect("Failed to write to event file");

        if record.target() != "events" {
            let mut l = self.log.lock().unwrap();
            l.push(r);
        }
    }

    fn flush(&self) {}
}

pub fn install_logger() {
    set_logger(&*DEBUG_LOGGER).expect("Failed to install logger");
}
