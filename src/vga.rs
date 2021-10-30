const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

use core::fmt;

use lazy_static::lazy_static;
use spin::Mutex;
use volatile::Volatile;
use x86_64::instructions::interrupts::without_interrupts;

lazy_static! {
    pub static ref VGA_WRITER: Mutex<VGAWriter> =
        Mutex::new(VGAWriter::default());
}

pub struct VGAWriter {
    column_position: usize,
    color_code:      ColorCode,
    buffer:          &'static mut Buffer,
}

impl VGAWriter {
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line();
                }

                let row = BUFFER_HEIGHT - 1;
                let col = self.column_position;
                let color_code = self.color_code;

                self.buffer.chars[row][col]
                    .write(VGAChar { ascii_character: byte, color_code });
                self.column_position += 1;
            },
        }
    }

    pub fn write_str(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                // printable ASCII byte or newline
                0x20..=0x7e | b'\n' => self.write_byte(byte),

                // not part of printable ASCII range
                _ => self.write_byte(0xfe),
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
        self.column_position = 0;
    }

    fn delete(&mut self) {
        let row = BUFFER_HEIGHT - 1;
        let col = self.column_position;
        let color_code = self.color_code;

        self.buffer.chars[row][col - 1]
            .write(VGAChar { ascii_character: b' ', color_code });
        self.column_position -= 1;
    }

    fn clear_row(&mut self, row: usize) {
        let blank =
            VGAChar { ascii_character: b' ', color_code: self.color_code };
        for col in 0..BUFFER_WIDTH {
            self.buffer.chars[row][col].write(blank);
        }
    }
}

pub fn with_color<F: Fn()>(fg: Color, bg: Color, task: F) {
    let mut writer = VGA_WRITER.lock();
    let old_color_code = writer.color_code;
    writer.color_code = ColorCode::new(fg, bg);
    drop(writer);

    task();

    let mut writer = VGA_WRITER.lock();
    writer.color_code = old_color_code;
}

impl Default for VGAWriter {
    fn default() -> Self {
        Self {
            column_position: 0,
            color_code:      ColorCode::new(Color::White, Color::Black),
            buffer:          unsafe { &mut *(0xb8000 as *mut Buffer) },
        }
    }
}

impl fmt::Write for VGAWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_str(s);
        Ok(())
    }
}

#[repr(transparent)]
struct Buffer {
    chars: [[Volatile<VGAChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct VGAChar {
    ascii_character: u8,
    color_code:      ColorCode,
}

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ColorCode(u8);

impl ColorCode {
    fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

#[repr(u8)]
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    Black      = 0,
    Blue       = 1,
    Green      = 2,
    Cyan       = 3,
    Red        = 4,
    Magenta    = 5,
    Brown      = 6,
    LightGray  = 7,
    DarkGray   = 8,
    LightBlue  = 9,
    LightGreen = 10,
    LightCyan  = 11,
    LightRed   = 12,
    Pink       = 13,
    Yellow     = 14,
    White      = 15,
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;

    without_interrupts(|| {
        VGA_WRITER.lock().write_fmt(args).unwrap();
    });
}

#[test_case]
fn test_println() {
    println!("test_println_simple output");
}

#[test_case]
fn test_println_many() {
    for _ in 0..200 {
        println!("test_println_many output");
    }
}

#[test_case]
fn test_println_output() {
    use core::fmt::Write;
    let s = "Some test string that fits on a single line";

    without_interrupts(|| {
        let mut writer = VGA_WRITER.lock();
        writeln!(writer, "\n{}", s).expect("writeln failed");
        for (i, c) in s.chars().enumerate() {
            let screen_char = writer.buffer.chars[BUFFER_HEIGHT - 2][i].read();
            assert_eq!(char::from(screen_char.ascii_character), c);
        }
    });
}
