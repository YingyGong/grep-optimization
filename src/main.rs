mod earley_parse;
mod cfg;
mod nfa;

use crate::nfa::NFA;
use crate::earley_parse::CFG;
use crate::cfg::cfg_for_regular_expression;
use std::env;
use std::error::Error;
use std::fmt::Result;
use std::fs::File;
use std::io::{self, BufReader, BufRead};

fn grep(regex: &str, filename: &str) 
-> std::io::Result<()> 
{   
    let cfg = cfg_for_regular_expression();
    let ast = cfg.parse(regex).unwrap().collapse();
    let nfa = NFA::from_regex(&ast);
    let nfa = NFA::epsilon_close(nfa);

    let file = File::open(filename)?;
    let reader = BufReader::new(file);

    for (index, line) in reader.lines().enumerate() {
        let line = line?;
        // start from any index in the line
        let output_strs = nfa.check_str_princeton(&line);
        for output_str in output_strs {
            println!("{}:{}", index + 1, output_str);
        }
    }

    Ok(())
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: {} <regular expression> <input_file>", args[0]);
        std::process::exit(1);
    }

    let regex = &args[1];
    let input_file = &args[2];


    match grep(regex, input_file) {
        Ok(()) => (),
        Err(e) => eprintln!("Error: {}", e),
    }

}