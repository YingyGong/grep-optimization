use crate::nfa::NFA;
use crate::nfa;
use crate::earley_parse::CFG;
use crate::cfg;
use crate::cfg::cfg_for_regular_expression;

pub fn check_str_prefix_extraction(regex: &str, line: &str) -> Vec<String> {
    let (prefix, rest) = cfg::prefix_and_remainder_extract_after_plus(regex);
    let mut start_positions = vec![]; // the ending position of the prefix in the line
    // find all the prefixes in the line
    line.match_indices(&prefix).for_each(|(start, _)| start_positions.push(start + prefix.len()));
    // create a new NFA from the rest
    let nfa = nfa::nfa_from_reg(&rest);
    // check the rest of the line
    let output_strs = nfa.check_str_with_start_index(line, start_positions);
    // add prefix to the output strings
    let mut output_strs_with_prefix = vec![];
    for output_str in output_strs {
        output_strs_with_prefix.push(format!("{}{}", prefix, output_str));
    }
    output_strs_with_prefix
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_check_str_prefix_extraction() {
        let regex = "Caltech|California";
        let line = "Caltech is in California";
        let (prefix, rest) = cfg::prefix_and_remainder_extract_after_plus(regex);
        println!("prefix: {}, rest: {}", prefix, rest);
        let output_strs = check_str_prefix_extraction(regex, line);
        for output_str in output_strs {
            println!("{}", output_str);
        }
    }

    #[test]
    fn test_start_positions() {
        let regex = "Caltech|California";
        let line = "Caltech is in California";
        let (prefix, rest) = cfg::prefix_and_remainder_extract_after_plus(regex);
        println!("prefix: {}, rest: {}", prefix, rest);
        let mut start_positions = vec![];
        line.match_indices(&prefix).for_each(|(start, _)| start_positions.push(start));
        for start_position in start_positions {
            println!("{}", start_position);
        }
    }
}