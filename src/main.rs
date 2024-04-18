mod earley_parse;
mod cfg;
mod nfa;

use std::env;
use std::error::Error;
use std::fmt::Result;
use std::fs::File;
use std::io::{self, BufReader, BufRead};

// fn grep(regex: &str, filenmae: &str) -> std::io::Result<()> {
//     let file = File::open(filenmae)?;
//     let reader = BufReader::new(file);

//     for (index, line) in reader.lines().enumerate() {
//         let line = line?;
//         if regex.is_match(&line) {
//             println!("{}", line);
//         }
//     }

//     Ok(())
// }

fn main() {
    // let args: Vec<String> = env::args().collect();
    // if args.len() != 3 {
    //     eprintln!("Usage: {} <regular expression> <input_file>", args[0]);
    //     std::process::exit(1);
    // }

    // let regex = &args[1];
    // let input_file = &args[2];

    // match grep(regex, input_file) {
    //     Ok(()) => (),
    //     Err(e) => eprintln!("Error: {}", e),
    // }

}