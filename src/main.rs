use std::fs;
use std::io;
use std::process::Command;

pub mod panes;

fn main() {
    let total_iterations = 5;
    watch(total_iterations).unwrap();
}

fn watch(total_iterations: u8) -> Result<(), Box<dyn std::error::Error>> {
    let out_path = "pane.txt";
    let mut line_count = 0;
    let mut i = 0;

    loop {
        if i == total_iterations {
            break;
        }
        if line_count == 25 {
            let pane = panes::capture_current_pane()
                .unwrap_or_else(|err| panic!("Problem with pane: {:?}", err));
            println!("{pane}");
            fs::write(out_path, pane).unwrap();
            Command::new("clear");
            line_count = 0;
            i += 1;
        } else {
            // Read input from the user
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;

            line_count += 1;

            println!("Line count: {}", line_count);
        }
    }

    Ok(())
}
