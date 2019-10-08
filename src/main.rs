use lolwut::Canvas;

use std::error;
use std::env::args;

fn main() -> Result<(), Box<dyn error::Error>> {
    let console_cols    = args().nth(1).and_then(|s| s.parse().ok()).unwrap_or(66);
    let squares_per_row = args().nth(2).and_then(|s| s.parse().ok()).unwrap_or(8);
    let squares_per_col = args().nth(3).and_then(|s| s.parse().ok()).unwrap_or(12);

    let canvas = Canvas::create_and_render_schotter(console_cols, squares_per_row, squares_per_col)?;

    print!("{}", canvas.render());
    println!("Georg Nees - schotter, plotter on paper, 1968");

    Ok(())
}
