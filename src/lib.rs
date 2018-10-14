
//! # `LOLWUT`
//!
//! This is a reimplementation of Redis's `LOLWUT5` command. This command is
//! documented and implemented in the redis source code
//! [here](https://github.com/antirez/redis/blob/91685eeeb/src/lolwut5.c).
//!
//! This implementation is mostly a port, and it does re-use code/comments.

#![allow(dead_code, unused_variables)]

use std::{
    char,
    f32::consts::PI,
};

/// This structure represents out canvas.
pub struct Canvas<'px> {
    pixels: &'px mut [u8],
    width:  u32,
    height: u32,
}

#[derive(Debug, Copy, Clone)]
pub enum CanvasError {
    /// WIP.
    NotImplYet,
    /// The provided buffer does not have enough bytes to be used in a canvas.
    PixelBufferTooSmall { needed: usize, actual: usize },
    /// Attempted to access a pixel at (x, y), which is out of bounds.
    PixelOutOfBounds { x: u32, y: u32, width: u32, height: u32 },
}
use self::CanvasError::*;

pub type LolwutResult<T> = Result<T, CanvasError>;

impl <'px> Canvas<'px> {

    /// Create a Canvas ready to write pixels to.
    ///
    /// `buf` must be large enough to hold `width` and `height` pixels.
    pub fn create(width: u32, height: u32, buf: &'px mut [u8]) -> LolwutResult<Canvas> {
        // We cannot create a canvas if the buffer is too small.
        // We use a saturating mul to simplify the logic in the error case. This
        // way, an overflow (which is already going to cause problems) is checked
        // as an error so long as `buf.len()` < `usize::MAX`.
        // In practice, I don't know of any platforms that allow you to allocate
        // a buffer of length `usize::MAX`, so we don't need to handle this case.
        let px_count = (width as usize).saturating_mul(height as usize);
        if px_count > buf.len() {
            return Err(PixelBufferTooSmall {
                needed: px_count,
                actual: buf.len()
            });
        }

        Ok(Canvas {
            pixels: buf,
            width,
            height,
        })
    }

    /// Construct an in index into the pixels buffer from an `(x, y)` coordinate.
    /// If the coordinate would be out of bounds, or if overflow occurs, returns
    /// `None`.
    fn index(&self, x: u32, y: u32) -> LolwutResult<usize> {
        // All of our error checking is Option-based. However, what we want is
        // Result-based.
        // Furthermore, every error generates the same Error: PixelOutOfBounds.
        // This lambda lets us use `?` on checked arithmetic, while still
        // keeping the syntax concise.
        // Even in debug builds, this lambda is inlined.
        match || -> Option<usize> {
            let x = x as usize;
            let y = y as usize;
            let width = self.width as usize;
            let height = self.height as usize;

            if x < width && y < height {
                let index = x.checked_add(y.checked_mul(width)?)?;
                if index < self.pixels.len() {
                    return Some(index);
                }
            }

            None
        }() {
            Some(index) =>  Ok(index),
            None        =>  Err(PixelOutOfBounds {
                                x,
                                y,
                                width: self.width,
                                height: self.height
                            }),
        }
    }

    /// Get the pixel at `(x, y)`.
    pub fn get_pixel(&self, x: u32, y: u32) -> LolwutResult<u8> {
        Ok(self.pixels[self.index(x, y)?])
    }

    /// Draw a single pixel at `(x, y)`.
    pub fn draw_pixel(&mut self, x: u32, y: u32, color: u8) -> LolwutResult<()> {
        let index = self.index(x, y)?;
        self.pixels[index] = color;
        Ok(())
    }

    /// Draw a line from `(x1, y1)` to `(x2, y2)` using the Bresenham algorithm.
    pub fn draw_line(&mut self,
                     x1: u32,
                     y1: u32,
                     x2: u32,
                     y2: u32,
                     color: u8)
        -> LolwutResult<()>
    {
        // TODO: Explain this madness.
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
            println!("({}, {})", x, y);
            self.draw_pixel(x as u32, y as u32, color)?;
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

        Ok(())
    }

    /// Draw a square centered at the specified `(x, y)` coordinates, with the
    /// specified rotation angle and size.
    pub fn draw_square(&mut self, x: u32, y: u32, size: f32, angle: f32)
        -> LolwutResult<()>
    {
        let size = ((size as f64) / 1.4142135623).round() as f32;

        // Construct the four corners of the square.
        let mut points: [(u32, u32); 4] = Default::default();
        let mut k = PI/4.0 + angle;
        for j in 0..4 {
            points[j].0 = (k.sin() * size + x as f32).round() as u32;
            points[j].1 = (k.cos() * size + y as f32).round() as u32;
            k += PI/2.0;
        }

        // Draw each of the four connecting lines
        for j in 0..4 {
            let p = points[j];
            let q = points[(j + 1) % 4];
            self.draw_line(p.0, p.1, q.0, q.1, 1)?;
        }

        Ok(())
    }

    /// Generate a `String` with each pixel represented in a grid.
    pub fn render(&self) -> LolwutResult<String> {
        let mut out = String::with_capacity(self.pixels.len());
        for y in 0..self.height {
            for x in 0..self.width {
                let t = match self.get_pixel(x, y)? {
                    0 => '.',
                    1 => '@',
                    _ => '?',
                };
                out.push(t);
            }
            out.push('\n');
        }
        Ok(out)
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
    // Convert to unicode. This is in the U0800-UFFFF range,
    // so we need to emit it like this in three bytes:
    //      1110_xxxx 10xx_xxxx 10xx_xxxx
    let code = 0x2800 + byte as u32;
    let o0: u32 = 0xe0 | ((code >> 12) as u8) as u32;
    let o1: u32 = 0x80 | ((code >>  6) as u8 & 0x3f) as u32;
    let o2: u32 = 0x80 | ( code        as u8 & 0x3f) as u32;
    char::from_u32(o0 << 16 | o1 << 8 | o2).unwrap_or('☂')
}
