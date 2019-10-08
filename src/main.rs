use lolwut::Canvas;

use std::error;
use std::env::args;

fn print_help() {
    let program_name: String = args().nth(0).unwrap();
    eprintln!("Usage: {} 66 8 12", program_name);
    eprintln!("  66 columns of output in the console window");
    eprintln!("  8 squares per row (wide)");
    eprintln!("  12 squares per column (tall)");
}

fn main() -> Result<(), Box<dyn error::Error>> {
    if let Some(arg) = args().nth(1) {
        match arg.as_str() {
            "help" |
            "--help" |
            "-h" |
            "h" => {
                print_help();
                return Ok(());
            }
            _ => {}
        }
    }

    if args().len() > 4 {
        print_help();
        return Ok(());
    }

    let console_cols    = args().nth(1).and_then(|s| s.parse().ok()).unwrap_or(66);
    let squares_per_row = args().nth(2).and_then(|s| s.parse().ok()).unwrap_or(8);
    let squares_per_col = args().nth(3).and_then(|s| s.parse().ok()).unwrap_or(12);

    let canvas = Canvas::create_and_render_schotter(console_cols, squares_per_row, squares_per_col)?;

    print!("{}", canvas.render());
    println!("Georg Nees - schotter, plotter on paper, 1968");

    Ok(())
}
