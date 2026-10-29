use std::io::{self, Write};

fn main()  {
    let mut buffer: String = String::new();
    loop {
        print!("> ");
        io::stdout().flush().unwrap();

        let bytes = io::stdin().read_line(&mut buffer).expect("Failed to read line");

        if bytes == 0 {
            println!();
            break;
        }

        let input = buffer.trim();
        if input.is_empty() {
            continue;
        }

        println!("input: {input}")
    }
}
