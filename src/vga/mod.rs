use crate::{
    mem::Volatile,
    sync::{Lazy, Mutex},
};
use core::fmt::{self, Write};

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    x86_64::instructions::interrupts::without_interrupts(|| {
        WRITER.lock().write_fmt(args).unwrap();
    });
}

static WRITER: Lazy<Mutex<Writer>> = Lazy::new(|| Mutex::new(Writer::new()));

pub struct Writer {
    buffer: &'static mut Buffer,
    col: usize,
    color: Color,
}

impl Writer {
    pub fn new() -> Self {
        Writer {
            col: 0,
            color: Color::new(FgColor::White, BgColor::Black, false),
            buffer: unsafe { &mut *(0xB8000 as *mut Buffer) },
        }
    }

    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.col >= BUFFER_WIDTH {
                    self.new_line();
                }

                let row = BUFFER_HEIGHT - 1;
                let col = self.col;

                let color = self.color;
                self.buffer.chars[row][col].write(ScreenChar {
                    character: byte,
                    color,
                });
                self.col += 1;
            }
        }
    }
    pub fn write_bytes(&mut self, s: &[u8]) {
        for b in s {
            match b {
                0x20..=0x7E | b'\n' => self.write_byte(*b),
                _ => self.write_byte(0xFE),
            }
        }
    }

    fn new_line(&mut self) {
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                let character = self.buffer.chars[row][col].read();
                self.buffer.chars[row - 1][col].write(character);
            }
        }
        self.clear_row(BUFFER_HEIGHT - 1);
        self.col = 0;
    }
    fn clear_row(&mut self, row: usize) {
        let blank = ScreenChar {
            character: b' ',
            color: self.color,
        };
        for col in 0..BUFFER_WIDTH {
            self.buffer.chars[row][col].write(blank);
        }
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_bytes(s.as_bytes());
        Ok(())
    }
}

impl Default for Writer {
    fn default() -> Self {
        Self::new()
    }
}

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;
#[repr(transparent)]
struct Buffer {
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct ScreenChar {
    character: u8,
    color: Color,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct Color(u8);

impl Color {
    // | 7     | 6  | 5  | 4  | 3 | 2 | 1 | 0 |
    // | blink | background   | foreground    |
    pub fn new(fg: FgColor, bg: BgColor, blink: bool) -> Self {
        Self(if blink { 0b1000_0000 } else { 0 } | (bg as u8) << 4 | fg as u8)
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum FgColor {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum BgColor {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
}

#[cfg(test)]
mod tests {
    #[test_case]
    fn println_single() {
        println!("Hello, World!");
    }

    #[test_case]
    fn println_many() {
        for i in 0..200 {
            println!("{}", i);
        }
    }

    #[test_case]
    fn println_output() {
        use crate::vga::{BUFFER_HEIGHT, WRITER};
        use core::fmt::Write;
        use x86_64::instructions::interrupts;

        let s = "Hello, World!";
        interrupts::without_interrupts(|| {
            let mut writer = WRITER.lock();
            writeln!(writer, "\n{}", s).expect("writeln failed");
            for (i, c) in s.chars().enumerate() {
                let sc = writer.buffer.chars[BUFFER_HEIGHT - 2][i].read();
                assert_eq!(char::from(sc.character), c);
            }
        });
    }
}
