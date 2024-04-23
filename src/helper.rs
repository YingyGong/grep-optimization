use std::cmp::max;
use std::vec;
use std::collections::{HashMap, HashSet};

use crate::nfa::NFA;
use crate::nfa;
use crate::earley_parse::CFG;
use crate::cfg;
use crate::cfg::cfg_for_regular_expression;

const ALPHABET_SIZE: usize = 256;

// helper functions for boyer moore algorithm
fn alphabet_index(c: char) -> usize {
    let ans = c as usize;
    assert!(ans < ALPHABET_SIZE && ans >= 0);
    ans
}

// Return the length of the match of the substrings of S beginning at idx1 and idx2.
fn match_length(s: &str, idx1: usize, idx2: usize) -> usize {
    let mut idx1 = idx1;
    let mut idx2 = idx2;
    if idx1 == idx2 {
        return s.len() - idx1
    }
    let mut count = 0;
    while idx1 < s.len() && idx2 < s.len() && s.chars().nth(idx1).unwrap() == s.chars().nth(idx2).unwrap() {
        count += 1;
        idx1 += 1;
        idx2 += 1;
    }
    count
}

fn preprocess(s: &str) -> Vec<i32> {
    if s.len() == 0 {
        return vec![]
    }
    if s.len() == 1 {
        return vec![1]
    }
    let mut z = vec![0; s.len()];
    z[0] = s.len() as i32;
    z[1] = match_length(s, 0, 1) as i32;
    for i in 2..(1 + z[1]) {
        z[i as usize] = z[i as usize] - i + 1;
    }
    let mut l = 0;
    let mut r = 0;
    for i in 2 + z[1]..s.len() as i32 {
        if i <= r {
            let k: usize = (i - l) as usize;
            let b = z[k];
            let a = r - i + 1;
            if b < a {
                z[i as usize] = b;
            }
            else {
                z[i as usize] = a + match_length(s, a as usize, r as usize) as i32;
                l = i;
                r = i + z[i as usize] - 1;
            }
        }
        else {
            z[i as usize] = match_length(s, 0, i as usize) as i32;
            if z[i as usize] > 0 {
                l = i;
                r = i + z[i as usize] - 1;
            }
        }
    }
    z
}

fn bad_char_table(s: &str) -> Vec<Vec<i32>> { 
    if s.len() == 0 {
        return vec![vec![]; ALPHABET_SIZE]
    }
    let mut R = vec![vec![-1]; ALPHABET_SIZE];
    let mut alpha = vec![-1; ALPHABET_SIZE];

    for (i, c) in s.chars().enumerate() {
        alpha[alphabet_index(c)] = i as i32;
        for (j, &a) in alpha.iter().enumerate() {
            R[j].push(a);
        }
    }
    R
}

fn reverse_string(s: &str) -> String {
    s.chars().rev().collect()
}

fn reverse_vec(s: &Vec<i32>) -> Vec<i32> {
    s.iter().rev().map(|&x| x).collect()
}

fn good_suffix_table(s: &str) -> Vec<i32> {
    let mut L = vec![-1; s.len()];
    let mut N = preprocess(reverse_string(&s).as_str());
    let N = reverse_vec(&N);
    for j in 0..(s.len() -1) {
        let i = s.len() - N[j] as usize;
        if i != s.len() {
            L[i] = j as i32;
        }
    }
    L
}

fn full_shift_table(s: &str) -> Vec<i32> {
    let n = s.len();
    let mut f = vec![0; n];
    let z = preprocess(s);
    let mut longest = 0;

    for (i, &zv) in z.iter().rev().enumerate() {
        if zv == (i + 1) as i32 {
            longest = zv.max(longest);
        }
        f[n - i - 1] = longest;
    }
    f
}

pub fn find_prefix_boyer_moore(p: &str, t: &str) -> Vec<usize> {
    if p.is_empty() || t.is_empty() || t.len() < p.len() {
        return Vec::new();
    }

    let mut matches: Vec<usize> = Vec::new();
    let r = bad_char_table(p);
    let l = good_suffix_table(p);
    let f = full_shift_table(p);

    let mut k = p.len() -1;
    let mut previous_k = -1isize;

    while k < t.len() {
        let mut i: isize = (p.len() -1) as isize;
        let mut h: isize = k as isize;

        while i >= 0 && (h as isize) > previous_k && p.as_bytes()[i as usize] == t.as_bytes()[h as usize]{
            i -= 1;
            h -= 1;
        }

        if i == -1 || (h as isize) == previous_k {
            matches.push(k + 1 - p.len());
            k += if p.len() > 1 { p.len() - f[1] as usize } else { 1 };
        } else {
            let char_shift = i as isize - r[alphabet_index(t.chars().nth(h as usize).unwrap())][i as usize] as isize;
            let suffix_shift = if i + 1 == p.len() as isize {
                1
            } else if l[(i + 1) as usize] == -1 {
                p.len() - f[(i + 1) as usize] as usize
            } else {
                p.len() - 1 - l[(i + 1) as usize] as usize
            };
            let shift = char_shift.max(suffix_shift as isize);
            previous_k = if shift >= (i + 1) as isize { k as isize } else { previous_k };
            k += shift as usize;
        }
    }

    matches
}


pub fn check_str_prefix_extraction(regex: &str, line: &str) -> HashMap<usize, String> {
    // let (prefix, rest) = cfg::prefix_and_remainder_extract_after_plus(regex);
    let (prefix, rest) = cfg::prefix_and_remainder_extract(&cfg_for_regular_expression().parse(regex).unwrap().collapse());

    // find all the prefixes in the line
    let mut start_positions = find_prefix_boyer_moore(&prefix, line);
    if prefix.is_empty() {
        // start_positions are all index
        for i in 0..line.len() {
            start_positions.push(i);
        }
    }
    // add length of prefix to the start_positions
    for i in 0..start_positions.len() {
        start_positions[i] += prefix.len();
    } 


    // let mut start_positions = vec![]; // the ending position of the prefix in the line

    // // find all the prefixes in the line
    // line.match_indices(&prefix).for_each(|(start, _)| start_positions.push(start + prefix.len()));

    let mut output_strs_with_prefix: HashMap<usize, String>= HashMap::new();

    if start_positions.len() == 0 {
        return output_strs_with_prefix;
    }

    if rest != "" {
        // create a new NFA from the rest
        let nfa = nfa::nfa_from_reg(&rest);

        // check the rest of the line
        let output_strs = nfa.check_str_with_start_index(line, start_positions);

        // println!("after that function {:?}", output_strs);

        // add prefix to the output strings
        for output_str in output_strs {
            output_strs_with_prefix.insert(output_str.0, format!("{}{}", prefix, output_str.1));
        }

    }
    else {
        for i in 0..start_positions.len() {
            output_strs_with_prefix.insert(i, prefix.to_string());
        }
    }

    // delete empty string in output_strs_with_prefix
    // output_strs_with_prefix.retain(|x| x != "");
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
            println!("{}", output_str.1);
        }
    }

    #[test]
    fn test_start_positions_cali() {
        let regex = "Caltech|California";
        let line = "Caltech is in California";
        let (prefix, rest) = cfg::prefix_and_remainder_extract_after_plus(regex);
        println!("prefix: {}, rest: {}", prefix, rest);

        let mut start_positions = vec![]; // the ending position of the prefix in the line

        // find all the prefixes in the line
        line.match_indices(&prefix).for_each(|(start, _)| start_positions.push(start + prefix.len()));
        // print the len of the start_positions
        println!("len of start_positions: {}", start_positions.len());
        println!("start_positions: {:?}", start_positions);

        // create a new NFA from the rest
        let nfa = nfa::nfa_from_reg(&rest);
        nfa.debug_helper();

        // check the rest of the line
        let output_strs = nfa.check_str_with_start_index(line, start_positions);

        // add prefix to the output strings
        let mut output_strs_with_prefix = vec![];
        for output_str in output_strs {
            output_strs_with_prefix.push(format!("{}{}", prefix, output_str.1));
        }
        for output_str in output_strs_with_prefix {
            println!("{}", output_str);
        }
    }

    #[test]
    fn test_start_positions_simple() {
        let regex = "ab|ac";
        let line = "ab in ac";
        let (prefix, rest) = cfg::prefix_and_remainder_extract_after_plus(regex);
        println!("prefix: {}, rest: {}", prefix, rest);

        let mut start_positions = vec![]; // the ending position of the prefix in the line

        // find all the prefixes in the line
        line.match_indices(&prefix).for_each(|(start, _)| start_positions.push(start + prefix.len()));
        // print the len of the start_positions
        println!("len of start_positions: {}", start_positions.len());
        println!("start_positions: {:?}", start_positions);

        // create a new NFA from the rest
        let nfa = nfa::nfa_from_reg(&rest);
        nfa.debug_helper();

        // check the rest of the line
        let output_strs = nfa.check_str_with_start_index(line, start_positions);

        // add prefix to the output strings
        let mut output_strs_with_prefix = vec![];
        // add prefix to the output strings (from the key small to large)
        for (_, output_str) in output_strs.iter() {
            output_strs_with_prefix.push(format!("{}{}", prefix, output_str));
        }
    }

    #[test]
    fn test_start_position_repeat() {
        let regex = "(ab)*";
        let line = "ababab";
        let (prefix, rest) = cfg::prefix_and_remainder_extract_after_plus(regex);
        let mut start_positions = vec![]; // the ending position of the prefix in the line

        // find all the prefixes in the line
        line.match_indices(&prefix).for_each(|(start, _)| start_positions.push(start + prefix.len()));

        // print all the start positions
        println!("start_positions: {:?}", start_positions);
    }

    #[test]
    fn test_helper_foo() {
        let regex = "foo(d|l)";
        let line = "food fool";
        let (prefix, rest) = cfg::prefix_and_remainder_extract_after_plus(regex);
        let mut start_positions = vec![]; // the ending position of the prefix in the line

        // find all the prefixes in the line
        line.match_indices(&prefix).for_each(|(start, _)| start_positions.push(start + prefix.len()));

        // print all the start positions
        println!("start_positions: {:?}", start_positions);
        
        let output_strs = check_str_prefix_extraction(regex, line);
        for output_str in output_strs {
            println!("{}", output_str.1);
        }
    }

    #[test]
    fn test_helper_kleen_star() {
        let regex = "(ab)*";
        let line = "cabab";
        let (prefix, rest) = cfg::prefix_and_remainder_extract_after_plus(regex);
        println!("prefix: {}, rest: {}", prefix, rest);
        let mut start_positions = vec![]; // the ending position of the prefix in the line

        // find all the prefixes in the line
        line.match_indices(&prefix).for_each(|(start, _)| start_positions.push(start + prefix.len()));

        // print all the start positions
        println!("start_positions: {:?}", start_positions);
        
        let output_strs = check_str_prefix_extraction(&regex, line);
        for output_str in output_strs {
            println!("{}", output_str.1);
        }
    }

    #[test]
    fn test_helper_kleen_star_2() {
        let regex = "ab+";
        let line = "ababbabbb";
        let (prefix, rest) = cfg::prefix_and_remainder_extract_after_plus(regex);
        println!("prefix: {}, rest: {}", prefix, rest);
        let mut start_positions = vec![]; // the ending position of the prefix in the line

        // find all the prefixes in the line
        line.match_indices(&prefix).for_each(|(start, _)| start_positions.push(start + prefix.len()));

        // print all the start positions
        println!("start_positions: {:?}", start_positions);
        
        let output_strs = check_str_prefix_extraction(&regex, line);
        for output_str in output_strs {
            println!("{}", output_str.1);
        }
    }


    #[test]
    fn test_helper_kleen_star_3() {
        let regex = "b*";
        let line = "abbababbb";
        let node = cfg_for_regular_expression().parse(regex).unwrap().collapse();
        let (prefix, rest) = cfg::prefix_and_remainder_extract(&node);
        println!("prefix: {}, rest: {}", prefix, rest);
        let mut start_positions = vec![]; // the ending position of the prefix in the line

        // find all the prefixes in the line
        line.match_indices(&prefix).for_each(|(start, _)| start_positions.push(start + prefix.len()));

        // print all the start positions
        println!("start_positions: {:?}", start_positions);
        
        let output_strs = check_str_prefix_extraction(&regex, line);
        // for output_str in output_strs {
        //     println!("{}", output_str.1);
        // }

        let mut keys: Vec<usize> = output_strs.keys().cloned().collect();
            keys.sort();
        let mut sum_set: HashSet<usize> = HashSet::new();
        for key in keys {
            let str = output_strs.get(&key).unwrap();
            let value = key + str.len();
            if sum_set.contains(&value) {
                continue;
            }
            else {
                sum_set.insert(value);
            }
            println!("{}:{}", key, output_strs.get(&key).unwrap());
        }
    }

    #[test]
    fn test_boyer_moore() {
        let p = "abab";
        let t = "ababababab";
        let matches = find_prefix_boyer_moore(p, t);
        println!("matches: {:?}", matches);
    }

    #[test]
    fn test_basic_match() {
        let p = "test";
        let t = "this is a test string";
        let matches = find_prefix_boyer_moore(p, t);
        assert_eq!(matches, [10]);
    }

    #[test]
    fn test_no_match() {
        let p = "hello";
        let t = "world, this test fails";
        let matches = find_prefix_boyer_moore(p, t);
        assert_eq!(matches, []);
    }

    #[test]
    fn test_overlapping_matches() {
        let p = "ana";
        let t = "banana";
        let matches = find_prefix_boyer_moore(p, t);
        assert_eq!(matches, [1, 3]);
    }

    #[test]
    fn test_pattern_at_start() {
        let p = "start";
        let t = "start here";
        let matches = find_prefix_boyer_moore(p, t);
        assert_eq!(matches, [0]);
    }

    #[test]
    fn test_pattern_at_end() {
        let p = "end";
        let t = "at the end";
        let matches = find_prefix_boyer_moore(p, t);
        assert_eq!(matches, [7]);
    }

    #[test]
    fn test_full_text_match() {
        let p = "full";
        let t = "full";
        let matches = find_prefix_boyer_moore(p, t);
        assert_eq!(matches, [0]);
    }

    #[test]
    fn test_empty_pattern() {
        let p = "";
        let t = "non-empty";
        let matches = find_prefix_boyer_moore(p, t);
        assert_eq!(matches, []);
    }

    #[test]
    fn test_empty_text() {
        let p = "non-empty";
        let t = "";
        let matches = find_prefix_boyer_moore(p, t);
        assert_eq!(matches, []);
    }

    #[test]
    fn test_special_characters() {
        let p = "@!";
        let t = "How about this?! Yes, @!";
        let matches = find_prefix_boyer_moore(p, t);
        assert_eq!(matches, [22]);
    }

    #[test]
    fn test_case_insensitivity() {
        let p = "case";
        let t = "This is a Case for testing";
        let matches = find_prefix_boyer_moore(&p.to_lowercase(), &t.to_lowercase());
        assert_eq!(matches, [10]);
    }

    #[test]
    fn test_order() {
        let regex = "c(ab)*";
        let line = "cabab";
        let node = cfg_for_regular_expression().parse(regex).unwrap().collapse();
        let (prefix, rest) = cfg::prefix_and_remainder_extract(&node);
        println!("prefix: {}, rest: {}", prefix, rest);
        let mut start_positions = vec![]; // the ending position of the prefix in the line

        // find all the prefixes in the line
        line.match_indices(&prefix).for_each(|(start, _)| start_positions.push(start + prefix.len()));

        // print all the start positions
        println!("start_positions: {:?}", start_positions);
        
        let output_strs = check_str_prefix_extraction(&regex, line);
        for output_str in output_strs {
            println!("{}", output_str.1);
        }
    }

    #[test]
    fn test_union_all() {
        let regex = "a|b|c";
        let line = "abc";
        let node = cfg_for_regular_expression().parse(regex).unwrap().collapse();
        let (prefix, rest) = cfg::prefix_and_remainder_extract(&node);
        println!("prefix: {}, rest: {}", prefix, rest);
        let mut start_positions = vec![]; // the ending position of the prefix in the line

        // find all the prefixes in the line
        line.match_indices(&prefix).for_each(|(start, _)| start_positions.push(start + prefix.len()));

        // print all the start positions
        println!("start_positions: {:?}", start_positions);
        
        let output_strs = check_str_prefix_extraction(&regex, line);
        // for output_str in output_strs {
        //     println!("{}", output_str.1);
        // }

        let mut keys: Vec<usize> = output_strs.keys().cloned().collect();
            keys.sort();
        let mut sum_set: HashSet<usize> = HashSet::new();
        for key in keys {
            let str = output_strs.get(&key).unwrap();
            let value = key + str.len();
            if sum_set.contains(&value) {
                continue;
            }
            else {
                sum_set.insert(value);
            }
            println!("{}:{}", key, output_strs.get(&key).unwrap());
        }
    }

    #[test]
    fn test_escape_char() {
        let regex = r"\\";
        let line = "abcs\\";
        let node = cfg_for_regular_expression().parse(regex).unwrap().collapse();
        let (prefix, rest) = cfg::prefix_and_remainder_extract(&node);
        println!("prefix: {}, rest: {}", prefix, rest);
        let mut start_positions = vec![]; // the ending position of the prefix in the line

        // find all the prefixes in the line
        line.match_indices(&prefix).for_each(|(start, _)| start_positions.push(start + prefix.len()));

        // print all the start positions
        println!("start_positions: {:?}", start_positions);
        
        let output_strs = check_str_prefix_extraction(&regex, line);
        // for output_str in output_strs {
        //     println!("{}", output_str.1);
        // }

        let mut keys: Vec<usize> = output_strs.keys().cloned().collect();
            keys.sort();
        let mut sum_set: HashSet<usize> = HashSet::new();
        for key in keys {
            let str = output_strs.get(&key).unwrap();
            let value = key + str.len();
            if sum_set.contains(&value) {
                continue;
            }
            else {
                sum_set.insert(value);
            }
            println!("{}:{}", key, output_strs.get(&key).unwrap());
        }
    }

    #[test]
    fn test_question_mark() {
        let regex = "ka?";
        let line = "k";
        let node = cfg_for_regular_expression().parse(regex).unwrap().collapse();
        let (prefix, rest) = cfg::prefix_and_remainder_extract(&node);
        println!("prefix: {}, rest: {}", prefix, rest);
        let mut start_positions = vec![]; // the ending position of the prefix in the line

        // find all the prefixes in the line
        line.match_indices(&prefix).for_each(|(start, _)| start_positions.push(start + prefix.len()));

        // print all the start positions
        println!("start_positions: {:?}", start_positions);
        
        let output_strs = check_str_prefix_extraction(&regex, line);
        // for output_str in output_strs {
        //     println!("{}", output_str.1);
        // }

        let mut keys: Vec<usize> = output_strs.keys().cloned().collect();
            keys.sort();
        let mut sum_set: HashSet<usize> = HashSet::new();
        for key in keys {
            let str = output_strs.get(&key).unwrap();
            let value = key + str.len();
            if sum_set.contains(&value) {
                continue;
            }
            else {
                sum_set.insert(value);
            }
            println!("{}:{}", key, output_strs.get(&key).unwrap());
        }
    }

    #[test]
    fn test_vec() {
        let mut vec = Vec::new();
        vec.push(1);
        vec.push(2);
        vec.push(3);

        for item in vec {
            println!("{}", item);
        }
        let regex = "\\";
        let new_str = format!(r"{}", regex);
    }

    
}