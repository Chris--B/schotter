
use lolwut;

use std::error;

fn main() -> Result<(), Box<error::Error>> {
    const W: u32 = 132;
    const H: u32 = 196;
    let mut buf = [0; (W*H) as usize];
    let mut canvas = lolwut::Canvas::create(W, H, &mut buf)?;

    // Something interesting
    canvas.draw_schotter(66, 8, 12)?;
    let text = canvas.render();
    print!("{}", text);
    println!("Georg Nees - schotter, plotter on paper, 1968. Redis ver.");

    Ok(())
}
