use std::io;
use std::process::Command;

pub mod gpt_complete;
pub mod panes;
pub mod walk;

fn main() {
    let root = walk::walk_directory("test-dir").unwrap();
    println!("{}", root);
    // let total_iterations = 5;
    // watch(total_iterations).unwrap();
}

#[allow(unused)]
fn watch(total_iterations: u8) -> Result<(), Box<dyn std::error::Error>> {
    let out_path = "pane.txt";
    let mut line_count = 0;
    let mut i = 0;

    loop {
        if i == total_iterations {
            break;
        }
        // Read input from the user
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        Command::new(input);

        line_count += 1;

        println!("Line count: {}", line_count);

        if line_count == 25 {
            let pane = panes::Pane::capture();
            println!("{}", pane.content);
            pane.write_to(out_path).unwrap();
            line_count = 0;
            i += 1;
        }
    }

    Ok(())
}
