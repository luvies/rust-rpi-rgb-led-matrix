extern crate libc;

mod c;
pub mod led_matrix_options;

use libc::{c_char, c_int};
use std::error;
use std::ffi::CString;
use std::fmt;
use std::path::Path;
use std::ptr::null;

pub use c::LedColor;

/// A single pixel on a canvas.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pixel {
    pub x: i32,
    pub y: i32,
}

/// A canvas that can be drawn on. If this is the active canvas, then all draws
/// are pushed to the LED matrix without buffering. If this is the offscreen
/// canvas, then all changes are buffered until [LedMatrix::swap] is used.
pub struct LedCanvas {
    handle: *mut c::LedCanvas,
}

/// The current LED matrix.
pub struct LedMatrix {
    handle: *mut c::LedMatrix,
    _options: led_matrix_options::Options,
}

/// A font that can be used to draw text to the LED matrix.
pub struct LedFont {
    handle: *mut c::LedFont,
}

/// The error type for [LedMatrix::new] which is returned upon failure to create
/// a new [LedMatrix].
#[derive(Debug, Clone)]
pub struct NewMatrixError;

/// The error type for [LedFont::new] which is returned upon failure to create
/// a new [LedFont].
#[derive(Debug, Clone)]
pub struct NewFontError {
    kind: NewFontErrorKind,
}

/// The error kind that cause the [LedFont] create to fail.
#[derive(Debug, Clone)]
pub enum NewFontErrorKind {
    /// The path given was invalid.
    BadPath,
    /// The font could not be loaded using the given path.
    LoadFailed,
}

impl LedMatrix {
    /// Attempts to construct a new LED matrix with the provided options (or the
    /// default options).
    pub fn new(options: Option<led_matrix_options::Options>) -> Result<LedMatrix, NewMatrixError> {
        let options = {
            if let Some(o) = options {
                o
            } else {
                led_matrix_options::Options::new()
            }
        };

        let handle = unsafe {
            c::led_matrix_create_from_options(
                &options as *const led_matrix_options::Options,
                null::<c_int>() as *mut c_int,
                null::<c_char>() as *mut *mut *mut c_char,
            )
        };

        if handle.is_null() {
            Err(NewMatrixError)
        } else {
            Ok(LedMatrix {
                handle,
                _options: options,
            })
        }
    }

    /// Gets the active canvas from the LED matrix that can be drawn on.
    pub fn canvas(&self) -> LedCanvas {
        let handle = unsafe { c::led_matrix_get_canvas(self.handle) };

        LedCanvas { handle }
    }

    /// Creates a new canvas that can be drawn on and then swapped in using
    /// [LedMatrix::swap].
    pub fn offscreen_canvas(&self) -> LedCanvas {
        let handle = unsafe { c::led_matrix_create_offscreen_canvas(self.handle) };

        LedCanvas { handle }
    }

    /// Swaps the given canvas to active and returns the now inactive old canvas
    /// for buffering the next frame.
    pub fn swap(&self, canvas: LedCanvas) -> LedCanvas {
        let handle = unsafe { c::led_matrix_swap_on_vsync(self.handle, canvas.handle) };

        LedCanvas { handle }
    }
}

impl Drop for LedMatrix {
    fn drop(&mut self) {
        unsafe {
            c::led_matrix_delete(self.handle);
        }
    }
}

impl LedFont {
    /// Attempts to construct a new LED font.
    pub fn new(bdf_file: &Path) -> Result<LedFont, NewFontError> {
        let string = match bdf_file.to_str() {
            Some(s) => s,
            None => {
                return Err(NewFontError {
                    kind: NewFontErrorKind::BadPath,
                })
            }
        };
        let cstring = CString::new(string).unwrap();

        let handle = unsafe { c::load_font(cstring.as_ptr()) };

        if handle.is_null() {
            Err(NewFontError {
                kind: NewFontErrorKind::LoadFailed,
            })
        } else {
            Ok(LedFont { handle })
        }
    }
}

impl Drop for LedFont {
    fn drop(&mut self) {
        unsafe { c::delete_font(self.handle) }
    }
}

impl LedCanvas {
    /// Gets the size of the canvas.
    pub fn size(&self) -> (i32, i32) {
        let (mut width, mut height): (c_int, c_int) = (0, 0);
        unsafe {
            c::led_canvas_get_size(self.handle, &mut width, &mut height);
        }
        (width as i32, height as i32)
    }

    /// Sets the colour of the given pixel in the canvas.
    pub fn set(&mut self, pixel: Pixel, color: &LedColor) {
        unsafe {
            c::led_canvas_set_pixel(
                self.handle,
                pixel.x as c_int,
                pixel.y as c_int,
                color.red,
                color.green,
                color.blue,
            );
        }
    }

    /// Sets all the pixels to black.
    pub fn clear(&mut self) {
        unsafe {
            c::led_canvas_clear(self.handle);
        }
    }

    /// Fills the canvas with the given colour.
    pub fn fill(&mut self, color: &LedColor) {
        unsafe {
            c::led_canvas_fill(self.handle, color.red, color.green, color.blue);
        }
    }

    /// Draws a line between 2 points on the canvas using the given colour.
    pub fn draw_line(&mut self, begin: Pixel, end: Pixel, color: &LedColor) {
        unsafe {
            c::draw_line(
                self.handle,
                begin.x as c_int,
                begin.y as c_int,
                end.x as c_int,
                end.y as c_int,
                color.red,
                color.green,
                color.blue,
            );
        }
    }

    /// Draws a circle of the given radius around the point using the given
    /// colour.
    pub fn draw_circle(&mut self, center: Pixel, radius: u16, color: &LedColor) {
        unsafe {
            c::draw_circle(
                self.handle,
                center.x as c_int,
                center.y as c_int,
                radius as c_int,
                color.red,
                color.green,
                color.blue,
            );
        }
    }

    /// Draws a line of text on the canvas starting from the given point.
    pub fn draw_text(
        &mut self,
        font: &LedFont,
        text: &str,
        pixel: Pixel,
        color: &LedColor,
        kerning_offset: i32,
        vertical: bool,
    ) -> i32 {
        let ctext = CString::new(text).unwrap();
        unsafe {
            if vertical {
                c::vertical_draw_text(
                    self.handle,
                    font.handle,
                    pixel.x as c_int,
                    pixel.y as c_int,
                    color.red,
                    color.green,
                    color.blue,
                    ctext.as_ptr(),
                    kerning_offset as c_int,
                ) as i32
            } else {
                c::draw_text(
                    self.handle,
                    font.handle,
                    pixel.x as c_int,
                    pixel.y as c_int,
                    color.red,
                    color.green,
                    color.blue,
                    ctext.as_ptr(),
                    kerning_offset as c_int,
                ) as i32
            }
        }
    }
}

impl fmt::Display for NewMatrixError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "failed to create LED matrix")
    }
}

impl error::Error for NewMatrixError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        None
    }
}

impl fmt::Display for NewFontError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "failed to create LED font")
    }
}

impl error::Error for NewFontError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::PI;
    use std::{thread, time};

    const ROWS: u16 = 64;
    const COLS: u16 = 64;
    const CHAIN_LENGTH: u16 = 1;

    fn led_matrix() -> LedMatrix {
        let mut options = led_matrix_options::Options::new();
        options.set_hardware_mapping("adafruit-hat-pwm");
        options.set_chain_length(CHAIN_LENGTH);
        options.set_disable_hardware_pulsing(false);
        options.set_rows(ROWS);
        options.set_cols(COLS);
        LedMatrix::new(Some(options)).unwrap()
    }

    #[test]
    fn matrix_create() {
        let _matrix = led_matrix();
    }

    #[test]
    fn canvas_size() {
        let matrix = led_matrix();
        let canvas = matrix.canvas();
        assert_eq!(canvas.size(), (COLS as i32 * CHAIN_LENGTH as i32, ROWS as i32));
    }

    #[test]
    fn draw_line() {
        let matrix = led_matrix();
        let mut canvas = matrix.canvas();
        let (width, height) = canvas.size();
        let mut color = LedColor {
            red: 127,
            green: 0,
            blue: 0,
        };

        canvas.clear();
        for x in 0..width {
            color.blue = 255 - 3 * x as u8;
            canvas.draw_line(
                Pixel { x, y: 0 },
                Pixel {
                    x: width - 1 - x,
                    y: height - 1,
                },
                &color,
            );
            thread::sleep(time::Duration::new(0, 10000000));
        }
    }

    #[test]
    fn draw_circle() {
        let matrix = led_matrix();
        let mut canvas = matrix.canvas();
        let (width, height) = canvas.size();
        let mut color = LedColor {
            red: 127,
            green: 0,
            blue: 0,
        };
        let (x, y) = (width / 2, height / 2);

        canvas.clear();
        for r in 0..(width / 2) {
            color.green = color.red;
            color.red = color.blue;
            color.blue = (r * r) as u8;
            canvas.draw_circle(Pixel { x, y }, r as u16, &color);
            thread::sleep(time::Duration::new(0, 100000000));
        }
    }

    #[test]
    fn draw_text() {
        let matrix = led_matrix();
        let mut canvas = matrix.canvas();
        let font = LedFont::new(Path::new("test/10x20.bdf")).unwrap();
        let color = LedColor {
            red: 0,
            green: 127,
            blue: 0,
        };
        let (width, height) = canvas.size();
        let text_width = 10 * 9;
        let baseline = height / 2;

        canvas = matrix.offscreen_canvas();
        for x in 0..(2 * width) {
            let x = x % (10 * 9);
            canvas.clear();
            canvas.draw_text(
                &font,
                "Mah boy! ",
                Pixel { x, y: baseline },
                &color,
                0,
                false,
            );
            canvas.draw_text(
                &font,
                "Mah boy! ",
                Pixel {
                    x: x - text_width,
                    y: baseline,
                },
                &color,
                0,
                false,
            );
            canvas.draw_text(
                &font,
                "Mah boy! ",
                Pixel {
                    x: x + text_width,
                    y: baseline,
                },
                &color,
                0,
                false,
            );
            canvas = matrix.swap(canvas);
            thread::sleep(time::Duration::new(0, 50000000));
        }
    }

    #[test]
    fn gradient() {
        let matrix = led_matrix();
        let mut canvas = matrix.canvas();
        let mut color = LedColor {
            red: 0,
            green: 0,
            blue: 0,
        };
        let period = 400;
        let duration = time::Duration::new(3, 0);
        let sleep_duration = duration / period;

        for t in 0..period {
            let t = t as f64;
            color.red = ((PI * t / period as f64).sin() * 255.) as u8;
            color.green = ((2. * PI * t / period as f64).cos() * 255.) as u8;
            color.blue = ((3. * PI * t / period as f64 + 0.3).cos() * 255.) as u8;
            canvas.fill(&color);
            thread::sleep(sleep_duration);
        }
    }

    #[test]
    fn canvas_swap() {
        let matrix = led_matrix();
        let mut canvas = matrix.canvas();
        let mut color = LedColor {
            red: 127,
            green: 127,
            blue: 0,
        };
        canvas.fill(&color);
        canvas = matrix.offscreen_canvas();
        color.blue = 127;
        canvas.fill(&color);
        thread::sleep(time::Duration::new(0, 500000000));
        canvas = matrix.swap(canvas);
        color.red = 0;
        canvas.fill(&color);
        thread::sleep(time::Duration::new(0, 500000000));
        matrix.swap(canvas);
        thread::sleep(time::Duration::new(0, 500000000));
    }
}
