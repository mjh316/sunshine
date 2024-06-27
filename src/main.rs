use std::env;

fn read_file(location: &str) -> String {
    std::fs::read_to_string(location)
        .expect("Failed to read file")
        .trim()
        .to_string()
}

#[allow(dead_code)]
fn write_file(location: &str, data: &str) -> () {
    std::fs::write(location, data).expect("Failed to write file");
}

fn main() {
    let mut argv = env::args().collect::<Vec<String>>();
    let debug = argv.iter().any(|x| x == "--dbg");
    if debug {
        argv.retain(|x| x != "--dbg");
    }

    let location = argv.get(1);

    match location {
        Some(location) => {
            println!("Reading file: {}", location);
            let program = read_file(location);
            println!("{}", program);
        }
        None => {
            // No file provided, go to REPL?
            println!("Usage: {} <file>", argv[0]);
            return;
        }
    }
}
