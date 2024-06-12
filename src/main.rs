mod earley_parse;
mod cfg;
// pub mod nfa;
mod helper;
mod nfa_optimized;

use crate::helper::{bad_char_table, good_suffix_table, full_shift_table, find_prefix_boyer_moore, helper_print, helper_print_with_start};
use std::env;
use std::fs::File;
use std::io::{BufReader, BufRead};

fn grep(regex: &str, filename: &str) 
-> std::io::Result<()> 
{   
    let file = File::open(filename)?;
    let reader = BufReader::new(file);


    let mut nfa = nfa_optimized::nfa_from_reg(&regex);

    let prefix = nfa.find_prefix_from_nfa();
    let prefix_len = prefix.len();

    if !prefix.is_empty() {
        let r = bad_char_table(prefix.as_str());
        let l = good_suffix_table(prefix.as_str());
        let f = full_shift_table(prefix.as_str());

        for (index, line) in reader.lines().enumerate() {
            let line = line?;
    
            let start_positions: Vec<usize> = find_prefix_boyer_moore(&prefix, &line, &r, &l, &f);
    
            if start_positions.is_empty() {
                continue;
            }
            
            let matched_tuples = nfa.check_str_with_start( &start_positions, &line, prefix_len);
            
            if matched_tuples.is_empty() {
                continue;
            }
            helper_print_with_start(index + 1, start_positions, &line, matched_tuples, prefix_len, false);

        }
    }
    else {
        let suffix = nfa.find_suffix_from_nfa();
        let suffix_len = suffix.len();
        if !suffix.is_empty() {
            let r = bad_char_table(suffix.as_str());
            let l = good_suffix_table(suffix.as_str());
            let f = full_shift_table(suffix.as_str());

            for (index, line) in reader.lines().enumerate() {
                let line = line?;

                // reverse line
                let line: String = line.chars().rev().collect();

                let start_positions: Vec<usize> = find_prefix_boyer_moore(&suffix, &line, &r, &l, &f);

                if start_positions.is_empty() {
                    continue;
                }

                let matched_tuples = nfa.check_str_with_start( &start_positions, &line, suffix_len);
            
                if matched_tuples.is_empty() {
                    continue;
                }

                helper_print_with_start(index + 1, start_positions, &line, matched_tuples, suffix_len, true);
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

    match grep(regex, input_file) {
        Ok(()) => (),
        Err(e) => eprintln!("Error: {}", e),
    }

}


#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_grep() {
        let mut regex_vec: Vec<String>= Vec::new();
        let filename = "./test/file3.txt";

        // Including different types of regex patterns
        regex_vec.push("foo(d|l)".to_string());  // Testing grouping and alternatives
        regex_vec.push("abcdef".to_string());    // Testing exact matches
        regex_vec.push("c(ab)*".to_string());    // Testing repetition of groups
        regex_vec.push("ab*".to_string());       // Testing repetition of single character
        regex_vec.push("^begin".to_string());    // Testing start of line
        regex_vec.push("end$".to_string());      // Testing end of line
        regex_vec.push(".+".to_string());        // Testing match of one or more characters
        regex_vec.push("\\d+".to_string());      // Testing digit matching
        regex_vec.push("no\\s+match".to_string()); // Testing whitespace matching
        regex_vec.push(".*fail.*".to_string());  // Testing 'any character' and greediness

        for regex in &regex_vec {
            println!("Testing regex: {}", regex);
            match grep(regex, filename) {
                Ok(()) => println!("Success for regex: {}", regex),
                Err(e) => eprintln!("Error for regex '{}': {}", regex, e),
            }
        }
    }

    #[test]
    fn test_suffix() {
        let filename = "./test/file1";
        let regex = "b?aaa";

        match grep(regex, filename) {
            Ok(()) => println!("Success for regex: {}", regex),
            Err(e) => eprintln!("Error for regex '{}': {}", regex, e),
        }
    }


}
