
//! # `LOLWUT`
//!
//! This is a reimplementation of Redis's `LOLWUT5` command. This command is
//! documented and implemented in the redis source code
//! [here](https://github.com/antirez/redis/blob/91685eeeb/src/lolwut5.c).
//!
//! This implementation is mostly a port, and it does re-use code/comments.

#![allow(dead_code, unused_variables)]
use std::char;

/// This structure represents out canvas.
pub struct Canvas<'px> {
    pixels: &'px mut [u8],
    width:  u32,
    height: u32,
}

/// A color is represented as either a dot, or the absence of a dot.
#[derive(Debug, Copy, Clone)]
pub enum Color {
    Black = 0,
    White = 1,
}

/// Helper struct to make less ambiguous functions taking a width and height.
///
/// Usage:
/// ```rust
/// # use lolwut::Dims;
/// # fn foo_plain(_width: u32, _height: u32) {}
/// # fn foo_dims(_dims: Dims) {}
/// // Which number is height?
/// foo_plain(65, 120);
///
/// // Unambiguous which dimension is height!
/// foo_dims(Dims { width: 65, height: 120 });
/// ```
#[derive(Debug, Copy, Clone)]
pub struct Dims {
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Copy, Clone)]
pub enum CanvasError {
    NotImplYet,
    PixelBufferTooSmall { needed: usize, actual: usize },
    PixelOutOfBounds { x: u32, y: u32, dims: Dims },
}
use self::CanvasError::*;

pub type LolwutResult<T> = Result<T, CanvasError>;

impl <'px> Canvas<'px> {

    /// Create a Canvas ready to write pixels to. `buf` must be large enough
    /// to hold `dims.width` and `dims.height` pixels.
    pub fn create(dims: Dims, buf: &'px mut [u8]) -> LolwutResult<Canvas> {
        let width = dims.width;
        let height = dims.height;

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

    /// The dimensions of the canvas.
    pub fn dims(&self) -> Dims {
        Dims {
            width:  self.width,
            height: self.height,
        }
    }

    /// Construct an in index into the pixels buffer from an `(x, y)` coordinate.
    /// If the coordinate would be out of bounds, or if overflow occurs, returns
    /// `None`.
    fn index(&self, x: u32, y: u32) -> LolwutResult<usize> {
        let maybe_err = PixelOutOfBounds { x, y, dims: self.dims() };
        let index = y.checked_mul(self.height).ok_or(maybe_err)?
                     .checked_add(x).ok_or(maybe_err)? as usize;
        if index < self.pixels.len() {
            Ok(index)
        } else {
            Err(maybe_err)
        }
    }

    pub fn draw_pixel(&mut self, x: u32, y: u32, c: Color) -> LolwutResult<()> {
        let index = self.index(x, y)?;
        self.pixels[index] = c as u8;
        Ok(())
    }

    pub fn get_pixel(&self, x: u32, y: u32) -> LolwutResult<Color> {
        let byte = self.pixels[self.index(x, y)?];
        match byte {
            0 => Ok(Color::Black),
            1 => Ok(Color::White),
            // This is bad data!
            _ => unreachable!(),
        }
    }

    pub fn render(&self) -> LolwutResult<String> {
        let mut out = String::with_capacity(self.pixels.len());
        for x in 0..self.width {
            for y in 0..self.height {
                let c = self.get_pixel(x, y)?;
                let t = match c {
                    Color::Black => '.',
                    Color::White => '@',
                };
                out.push(t);
            }
            out.push('\n');
        }
        Ok(out)
    }
}

/// Translate a group of 8 pixels (2x4 rectangle) into the corresponding
/// braille character. Each bit of `byte` should correspond to the pixels
/// arranged as follows:
/// ```norun
///     0 3
///     1 4
///     2 5
///     6 7
/// ```
/// Returns the corresponding braille character.
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
