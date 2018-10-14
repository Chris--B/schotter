
use lolwut;
use lolwut::Dims;

use std::mem;

fn main() -> lolwut::LolwutResult<()> {
    let mut buf: [u8; 40*80] = unsafe { mem::zeroed() };
    let mut canvas = lolwut::Canvas::create(Dims { width: 40, height: 80 }, &mut buf)?;

    canvas.draw_pixel(1, 10, lolwut::Color::White)?;
    let text = canvas.render()?;
    println!("{}", text);

    Ok(())
}
