
use lolwut;

use std::mem;

fn main() -> Result<(), lolwut::CanvasError> {
    const W: i32 = 132;
    const H: i32 = 196;
    let mut buf: [u8; (W*H) as usize] = unsafe { mem::zeroed() };
    let mut canvas = lolwut::Canvas::create(W, H, &mut buf)?;

    // Something interesting
    canvas.draw_schotter(66, 8, 12)?;
    let text = canvas.render()?;
    print!("{}", text);
    println!("Georg Nees - schotter, plotter on paper, 1968. Redis ver.");

    Ok(())
}
