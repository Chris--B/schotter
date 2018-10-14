
use lolwut;

use std::mem;

fn main() -> lolwut::LolwutResult<()> {
    const W: u32 = 80;
    const H: u32 = 40;
    let mut buf: [u8; (W*H) as usize] = unsafe { mem::zeroed() };
    let mut canvas = lolwut::Canvas::create(W, H, &mut buf)?;

    canvas.draw_pixel(1, 10, 1)?;
    canvas.draw_line(20, 20, 30, 30, 1)?;
    let text = canvas.render()?;
    println!("{}", text);

    Ok(())
}
