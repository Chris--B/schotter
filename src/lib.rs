
//! # `LOLWUT`
//!
//! This is a reimplementation of Redis's `LOLWUT5` command. This command is
//! documented and implemented in the redis source code
//! [here](https://github.com/antirez/redis/blob/91685eeeb/src/lolwut5.c).
//!
//! This implementation is mostly a port, and it does re-use code/comments.

use std::{
    error,
    fmt,
    ptr,
    str,
    f32::consts::PI,
};

use rand::prelude::*;

/// The Canvas represents the area that's drawn in. Each pixel is either:
///     1 - and "on"
///     0 - and "off"
/// Other values will silently turn into 1.
pub struct Canvas {
    pixels: Vec<u8>,
    width:  i32,
    height: i32,
}

/// An error related to the Canvas creation.
#[derive(Debug, Copy, Clone)]
pub enum CanvasError {
    /// The provided buffer does not have enough bytes to be used as the
    /// backing memory for a canvas.
    PixelBufferTooSmall {
        needed: usize,
        actual: usize
    },

    /// The canvas needs to have certain dimensions,
    /// but the dimensions provided are too small.
    CanvasTooSmall {
        needed_width: i32,
        actual_width: i32,
        needed_height: i32,
        actual_height: i32,
    }
}
use self::CanvasError::*;

impl fmt::Display for CanvasError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl error::Error for CanvasError {}

impl Canvas {

    /// Create a Canvas of the specified size
    pub fn create(width: u32, height: u32)
        -> Result<Canvas, CanvasError>
    {
        let px_count = width * height;

        Ok(Canvas {
            pixels: vec![0; px_count as usize],
            width:  width as i32,
            height: height as i32,
        })
    }

    /// Create a Canvas large enough and render Schotter onto it
    pub fn create_and_render_schotter(
        console_cols: i32,
        squares_per_row: i32,
        squares_per_col: i32
    ) -> Result<Canvas, CanvasError>
    {
        let needed_width:  i32 = 2 * console_cols;
        let padding:       f32 = if needed_width > 4 { 2.0 } else { 0.0 };
        let square_side:   f32 = (needed_width as f32 - 2.0 * padding)
                                   / squares_per_row as f32;
        let needed_height: i32 = (square_side * squares_per_col as f32
                                   + 2.0 * padding).round() as i32;

        let mut canvas = Canvas::create(needed_width as u32, needed_height as u32)?;
        canvas.draw_schotter(console_cols, squares_per_row, squares_per_col)?;

        Ok(canvas)
    }


    // We want `clear()` and `fill()` to be dumb `memcpy()`s. Rust doesn't expose
    // a safe wrapper around memcpy yet, so we write the bytes directly.
    // This is unsafe in the general case - writing an arbitrary byte to
    // arbitrary types can go horribly wrong.
    // We are writing 0 and 1 into a fixed size buffer with known bounds, so
    // this is safe.

    /// Set all pixel values to clear
    pub fn clear(&mut self) {
        unsafe {
            ptr::write_bytes(&mut self.pixels[0], 0, self.pixels.len());
        }
    }

    /// Set all pixel values to set
    pub fn fill(&mut self) {
        unsafe {
            ptr::write_bytes(&mut self.pixels[0], 1, self.pixels.len());
        }
    }

    /// Construct an index into the pixels buffer from an `(x, y)` coordinate.
    /// If the coordinate would be out of bounds, or if overflow occurs,
    /// return `None`.
    fn index(&self, x: i32, y: i32) -> Option<usize> {
        if 0 <= x && x < self.width as i32 &&
           0 <= y && y < self.height as i32
        {
            let x = x as usize;
            let y = y as usize;
            let width = self.width as usize;
            let height = self.height as usize;

            // Because we're in bounds, we cannot overflow.
            let index = x + y * width;
            if index < self.pixels.len() {
                return Some(index);
            }
        }
        None
    }

    /// Get the pixel at `(x, y)`. Out of bounds pixels are read as empty (0).
    pub fn get_pixel(&self, x: i32, y: i32) -> u8 {
        match self.index(x, y) {
            Some(index) => self.pixels[index],
            None        => 0,
        }
    }

    /// Draw a single pixel at `(x, y)`. Out of bounds writes are ignored.
    pub fn draw_pixel(&mut self, x: i32, y: i32, color: u8) {
        match self.index(x, y) {
            Some(index) => self.pixels[index] = color,
            None        => {},
        }
    }

    /// Draw a line from `(x1, y1)` to `(x2, y2)` using the Bresenham algorithm.
    pub fn draw_line(&mut self, x1: i32, y1: i32, x2: i32, y2: i32, color: u8) {
        // TODO: Explain how this works.
        //      https://en.wikipedia.org/wiki/Bresenham%27s_line_algorithm
        let sx = if x1 < x2 { 1 } else { -1 };
        let sy = if y1 < y2 { 1 } else { -1 };
        let x2 = x2 as isize;
        let y2 = y2 as isize;
        let dx = (x2 - x1 as isize).abs();
        let dy = (y2 - y1 as isize).abs();

        let mut x = x1 as isize;
        let mut y = y1 as isize;
        let mut err = dx - dy;

        loop {
            self.draw_pixel(x as i32, y as i32, color);
            if x == x2 && y == y2 { break; }

            let e2 = 2 * err;
            if e2 > -dy {
                err -= dy;
                x += sx;
            }
            if e2 < dx {
                err += dx;
                y += sy;
            }
        }
    }

    /// Draw a square centered at the specified `(x, y)` coordinates, with the
    /// specified rotation angle and size.
    pub fn draw_square(&mut self, x: i32, y: i32, size: f32, angle: f32) {
        // `size`, as passed into this function, represents the scaling of a
        // unit square.
        // We will operate on four equally spaced points on a unit circle that
        // represent our square's corners.
        // We must adjust this `size` by the ratio between our square's diagonal
        // and the radius of the circle that encloses it to get the correct
        // scaling in the final square.
        // The square has unit side lengths, and thus has a diagonal of sqrt(2).
        let size = ((size as f64) / 1.4142135623).round() as f32;

        // We construct the four corners of the square by using our parametric
        // equations for the circle at four equally-spaced `k` values.
        let mut points: [(i32, i32); 4] = Default::default();
        // The first point of a non-rotated square is at t=PI/4. When we rotate
        // the square, we just offset this initial radian value.
        let mut k = PI/4.0 + angle;
        for j in 0..4 {
            points[j].0 = (k.sin() * size + x as f32).round() as i32;
            points[j].1 = (k.cos() * size + y as f32).round() as i32;
            k += PI/2.0;
        }

        // Each of the four points needs to be connected. We connect them in
        // counter-clockwise order
        for j in 0..4 {
            let p = points[j];
            let q = points[(j + 1) % 4];
            self.draw_line(p.0, p.1, q.0, q.1, 1);
        }
    }

    /// Draw Georg Ness's "Schotter"
    ///
    /// "Schotter" is a tiled arrangement of squares that grow increasingly
    /// chaotic as you advance down the image.
    pub fn draw_schotter(&mut self,
                         console_cols:    i32,
                         squares_per_row: i32,
                         squares_per_col: i32)
        -> Result<(), CanvasError>
    {
        let needed_width:  i32 = 2 * console_cols;
        let padding:       f32 = if needed_width > 4 { 2.0 } else { 0.0 };
        let square_side:   f32 = (needed_width as f32 - 2.0 * padding)
                                   / squares_per_row as f32;
        let needed_height: i32 = (square_side * squares_per_col as f32
                                   + 2.0 * padding).round() as i32;

        if needed_width  > self.width as i32  ||
           needed_height > self.height as i32
        {
            return Err(CanvasTooSmall {
                needed_width,
                needed_height,
                actual_width: self.width,
                actual_height: self.height,
            });
        }

        for y in 0..squares_per_col {
            // This scaling factor is chosen per row, and increases as you go
            // down the rows. (Row number increases downward).
            let factor = (y + 1) as f32 / (squares_per_col + 1) as f32;
            for x in 0..squares_per_row {
                let mut sx = (x as f32 * square_side +
                              square_side/2.0 + padding).round() as i32;
                let mut sy = (y as f32 * square_side +
                              square_side/2.0 + padding).round() as i32;

                let mut r1: f32 = random::<f32>() * factor;
                if random() { r1 = -r1; }

                let mut r2: f32 = random::<f32>() * factor;
                if random() { r2 = -r2; }

                let mut r3: f32 = random::<f32>() * factor;
                if random() { r3 = -r3; }

                let angle = r1;
                sx += (r2 * square_side / 3.0).round() as i32;
                sy += (r3 * square_side / 3.0).round() as i32;
                self.draw_square(sx as i32, sy as i32, square_side, angle);
            }
        }

        Ok(())
    }

    /// Render the canvas into a multi-line string. Pixels are either "on" or
    /// "off".
    ///
    /// This string is designed to be seen by humans, and should not be used
    /// in lieu of iterating over the pixels with `Canvas::get_pixel`.
    /// On  pixels are rendered as a dot, or other dark, solid marking.
    /// Off pixels are rendered as empty space or white space.
    pub fn render(&self) -> String {
        let mut out = String::with_capacity(self.pixels.len());
        // Iterate over the range in 2x4 vertical blocks.
        // TODO: Check edge case when height % 4 != 0, and width % 2 != 0.
        for y in (0..self.height).step_by(4) {
            for x in (0..self.width).step_by(2) {
                let x = x as i32;
                let y = y as i32;
                // Each bit in the byte corresponds to a different pixel in the
                // tile. The ordering here is specially chosen so that this
                // maps cleanly to the Braille character set.
                let mut byte: u8 = 0;
                if self.get_pixel(x,   y)   != 0 { byte |= 1 << 0; }
                if self.get_pixel(x,   y+1) != 0 { byte |= 1 << 1; }
                if self.get_pixel(x,   y+2) != 0 { byte |= 1 << 2; }
                if self.get_pixel(x+1, y)   != 0 { byte |= 1 << 3; }
                if self.get_pixel(x+1, y+1) != 0 { byte |= 1 << 4; }
                if self.get_pixel(x+1, y+2) != 0 { byte |= 1 << 5; }
                if self.get_pixel(x,   y+3) != 0 { byte |= 1 << 6; }
                if self.get_pixel(x+1, y+3) != 0 { byte |= 1 << 7; }
                out.push(translate_pixels_group(byte));
            }
            out.push('\n');
        }
        out
    }
}

/// Translate a group of 8 pixels (2x4 rectangle) into their corresponding
/// braille character.
///
/// Each bit of `byte` should correspond to the pixels arranged as follows:
/// ```norun
/// [0] [3]
/// [1] [4]
/// [2] [5]
/// [6] [7]
/// ```
pub fn translate_pixels_group(byte: u8) -> char {
    // See: https://en.wikipedia.org/wiki/Braille_Patterns

    // Braille has a standard 6-dot cell for conveying symbols, which was later
    // expanded into the 8-cell. Each of the 8 cells (as labeled above) can be
    // either raised or not, hence the obvious mapping to a byte.
    // Note that some fonts may render these with hollow dots in place of space,
    // and that will change the appearance of the script.

    // Convert to unicode. This is in the U0800-UFFFF range,
    // so we need to emit it like this in three bytes:
    //      1110_xxxx 10xx_xxxx 10xx_xxxx
    let code = 0x2800 + byte as u32;
    let out: [u8; 3] = [
        0xe0 | ((code >> 12) as u8),
        0x80 | ((code >>  6) as u8 & 0x3f),
        0x80 | ( code        as u8 & 0x3f),
    ];
    unsafe {
        str::from_utf8_unchecked(&out).chars().nth(0).unwrap_or('☂')
    }
}

#[cfg(test)]
mod t {
    use super::*;

    #[test]
    fn check_translate_pixels_group() {
        let braille = [
            '\u{2800}', // BRAILLE PATTERN BLANK
            // 6-dot patterns
                 '⠁', '⠂', '⠃', '⠄', '⠅', '⠆', '⠇',
            '⠈', '⠉', '⠊', '⠋', '⠌', '⠍', '⠎', '⠏',
            '⠐', '⠑', '⠒', '⠓', '⠔', '⠕', '⠖', '⠗',
            '⠘', '⠙', '⠚', '⠛', '⠜', '⠝', '⠞', '⠟',
            '⠠', '⠡', '⠢', '⠣', '⠤', '⠥', '⠦', '⠧',
            '⠨', '⠩', '⠪', '⠫', '⠬', '⠭', '⠮', '⠯',
            '⠰', '⠱', '⠲', '⠳', '⠴', '⠵', '⠶', '⠷',
            '⠸', '⠹', '⠺', '⠻', '⠼', '⠽', '⠾', '⠿',
            // 8-dot patterns
            '⡀', '⡁', '⡂', '⡃', '⡄', '⡅', '⡆', '⡇',
            '⡈', '⡉', '⡊', '⡋', '⡌', '⡍', '⡎', '⡏',
            '⡐', '⡑', '⡒', '⡓', '⡔', '⡕', '⡖', '⡗',
            '⡘', '⡙', '⡚', '⡛', '⡜', '⡝', '⡞', '⡟',
            '⡠', '⡡', '⡢', '⡣', '⡤', '⡥', '⡦', '⡧',
            '⡨', '⡩', '⡪', '⡫', '⡬', '⡭', '⡮', '⡯',
            '⡰', '⡱', '⡲', '⡳', '⡴', '⡵', '⡶', '⡷',
            '⡸', '⡹', '⡺', '⡻', '⡼', '⡽', '⡾', '⡿',
            '⢀', '⢁', '⢂', '⢃', '⢄', '⢅', '⢆', '⢇',
            '⢈', '⢉', '⢊', '⢋', '⢌', '⢍', '⢎', '⢏',
            '⢐', '⢑', '⢒', '⢓', '⢔', '⢕', '⢖', '⢗',
            '⢘', '⢙', '⢚', '⢛', '⢜', '⢝', '⢞', '⢟',
            '⢠', '⢡', '⢢', '⢣', '⢤', '⢥', '⢦', '⢧',
            '⢨', '⢩', '⢪', '⢫', '⢬', '⢭', '⢮', '⢯',
            '⢰', '⢱', '⢲', '⢳', '⢴', '⢵', '⢶', '⢷',
            '⢸', '⢹', '⢺', '⢻', '⢼', '⢽', '⢾', '⢿',
            '⣀', '⣁', '⣂', '⣃', '⣄', '⣅', '⣆', '⣇',
            '⣈', '⣉', '⣊', '⣋', '⣌', '⣍', '⣎', '⣏',
            '⣐', '⣑', '⣒', '⣓', '⣔', '⣕', '⣖', '⣗',
            '⣘', '⣙', '⣚', '⣛', '⣜', '⣝', '⣞', '⣟',
            '⣠', '⣡', '⣢', '⣣', '⣤', '⣥', '⣦', '⣧',
            '⣨', '⣩', '⣪', '⣫', '⣬', '⣭', '⣮', '⣯',
            '⣰', '⣱', '⣲', '⣳', '⣴', '⣵', '⣶', '⣷',
            '⣸', '⣹', '⣺', '⣻', '⣼', '⣽', '⣾', '⣿',
        ];

        // Exhaustively test every possible u8 value.
        for hex in 0..0x100i32 {
            let hex = hex as u8;
            let c = braille[hex as usize];
            let actual = translate_pixels_group(hex);
            println!("0x{}, '{}' =? '{}'", hex, c, actual);
            assert_eq!(c, actual,
                       "'{}'!='{}' (0x{:x} != 0x{:x})",
                       c, actual,
                       c as i32, actual as i32);
        }
    }
}
