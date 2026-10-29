use std::io::Stdin;

fn main()  {
    let mut buffer: String = String::new();
    let stdin: Stdin = std::io::stdin();
    stdin.read_line(&mut buffer).expect("failed to read line");
    println!("{}", buffer.trim_end());
}
