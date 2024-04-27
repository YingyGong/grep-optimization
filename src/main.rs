mod earley_parse;
mod cfg;
mod nfa;
mod helper;

// use regex::Regex;
use crate::nfa::NFA;
use crate::earley_parse::CFG;
use crate::cfg::cfg_for_regular_expression;
use crate::helper::{bad_char_table, good_suffix_table, full_shift_table, find_prefix_boyer_moore, helper_print, helper_print_with_start};
use std::collections::HashSet;
use std::env;
use std::error::Error;
use std::fmt::Result;
use std::fs::File;
use std::hash::Hash;
use std::io::{self, BufReader, BufRead};

fn grep(regex: &str, filename: &str, only_matching: bool, line_number: bool) 
-> std::io::Result<()> 
{   
    let file = File::open(filename)?;
    let reader = BufReader::new(file);

    // let (prefix, rest) = cfg::prefix_and_remainder_extract(&cfg_for_regular_expression().parse(regex).unwrap().collapse());
    // println!("prefix: {} and rest {}", prefix, rest);

    let mut nfa = nfa::nfa_from_reg(&regex);
    // nfa.debug_helper();

    let prefix = nfa.find_prefix_from_nfa();
    // println!("prefix: {}", prefix);
    // nfa.debug_helper();

    if !prefix.is_empty() {
        let r = bad_char_table(prefix.as_str());
        let l = good_suffix_table(prefix.as_str());
        let f = full_shift_table(prefix.as_str());
        // println!("r: {:?}, l: {:?}, f: {:?}", r, l, f);

        for (index, line) in reader.lines().enumerate() {
            let line = line?;
    
            let start_positions: Vec<usize> = find_prefix_boyer_moore(&prefix, &line, &r, &l, &f);
    
            // println!("prefix {} with start_positions: {:?}", prefix, start_positions);
    
            if start_positions.is_empty() {
                continue;
            }
            // nfa.debug_helper();
            let prefix_len = prefix.len();
            
            let matched_tuples = nfa.check_str_with_start( &start_positions, &line, prefix_len);
            

            if matched_tuples.is_empty() {
                continue;
            }
            helper_print_with_start(index + 1, start_positions, &line, matched_tuples, prefix_len);

            
        }
    }
    else {
        for (index, line) in reader.lines().enumerate() {
            let line = line?;
            let matched_tuples = nfa.check_str_without_start(&line);
            if matched_tuples.is_empty() {
                continue;
            }
            helper_print(index + 1, &line, matched_tuples);
        }
    }
    

    Ok(())
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: {} <regular expression> <input_file>", args[0]);
        std::process::exit(1);
    }

    let regex = &args[1];
    let input_file: &String = &args[2];
    let show_line_numbers = args.iter().any(|arg| arg == "line-number");
    let show_only_matching = args.iter().any(|arg| arg == "only-matching");


    match grep(regex, input_file, true, true) {
        Ok(()) => (),
        Err(e) => eprintln!("Error: {}", e),
    }

}

// // use regex::Regex as a banchmark test
// fn main() {
//     let args: Vec<String> = env::args().collect();
//     if args.len() < 3 {
//         eprintln!("Usage: {} <regular expression> <input_file>", args[0]);
//         std::process::exit(1);
//     }
//     let regex = &args[1];
//     let input_file: &String = &args[2];

//     let re = Regex::new(regex).unwrap();
//     let file = File::open(input_file).unwrap();
//     let reader = BufReader::new(file);

//     for (index, line) in reader.lines().enumerate() {
//         let line = line.unwrap();
//         let output_strs: Vec<String> = re.find_iter(&line).map(|m| m.as_str().to_string()).collect();
//         if output_strs.len() > 0 {
//             for output_str in output_strs {
//                 println!("{}:{}", index + 1, output_str);
//             }
//         }
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_grep() {
        let mut regex_vec: Vec<String>= Vec::new();
        let filename = "./test/file3.txt";
        let only_matching = true;
        let line_number = true;

        // Including different types of regex patterns
        regex_vec.push("foo(d|l)".to_string());  // Testing grouping and alternatives
        regex_vec.push("abcdef".to_string());    // Testing exact matches
        regex_vec.push("c(ab)*".to_string());    // Testing repetition of groups
        regex_vec.push("ab*".to_string());       // Testing repetition of single character
        regex_vec.push("^begin".to_string());    // Testing start of line
        regex_vec.push("end$".to_string());      // Testing end of line
        // regex_vec.push(".+".to_string());        // Testing match of one or more characters
        // regex_vec.push("\\d+".to_string());      // Testing digit matching
        // regex_vec.push("no\\s+match".to_string()); // Testing whitespace matching
        // regex_vec.push(".*fail.*".to_string());  // Testing 'any character' and greediness

        for regex in &regex_vec {
            println!("Testing regex: {}", regex);
            match grep(regex, filename, only_matching, line_number) {
                Ok(()) => println!("Success for regex: {}", regex),
                Err(e) => eprintln!("Error for regex '{}': {}", regex, e),
            }
        }
    }


}
