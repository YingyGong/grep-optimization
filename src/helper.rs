use std::vec;

const ALPHABET_SIZE: usize = 96;

// helper functions for boyer moore algorithm
fn alphabet_index(c: char) -> usize {
    match c {
        '\t' => 0,
        ' '..='~' => (c as usize) - 0x20 + 1,
        _ => panic!("Character out of valid range"),
    }
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
        z[i as usize] = z[1] - i + 1;
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
                z[i as usize] = a + match_length(s, a as usize, r as usize + 1) as i32;
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

pub fn bad_char_table(s: &str) -> Vec<Vec<i32>> { 
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

pub fn good_suffix_table(s: &str) -> Vec<i32> {
    let mut l = vec![-1; s.len()];
    let mut n = preprocess(reverse_string(&s).as_str());
    let n = reverse_vec(&n);
    for j in 0..(s.len() -1) {
        let i = s.len() - n[j] as usize;
        if i != s.len() {
            l[i] = j as i32;
        }
    }
    l
}

pub fn full_shift_table(s: &str) -> Vec<i32> {
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

pub fn find_prefix_boyer_moore(p: &str, t: &str, r: &Vec<Vec<i32>>, l: &Vec<i32>, f: &Vec<i32>) -> Vec<usize> {
    assert!(!p.is_empty());
    if t.is_empty() || t.len() < p.len() {
        return Vec::new();
    }


    let mut matches: Vec<usize> = Vec::new();
    // let r = bad_char_table(p);
    // let l = good_suffix_table(p);
    // let f = full_shift_table(p);

    let mut k = p.len() -1;
    let mut previous_k = -1 as isize;

    while k < t.len() {
        let mut i: isize = (p.len() -1) as isize;
        let mut h: isize = k as isize;

        while i >= 0 && h > previous_k && p.as_bytes()[i as usize] == t.as_bytes()[h as usize]{
            i -= 1;
            h -= 1;
        }

        if i == -1 || h == previous_k {
            matches.push(k + 1 - p.len());
            // k += if p.len() > 1 { p.len() - f[1] as usize } else { 1 }; // delete f[1]
            k += 
            if p.len() > 1 {
                p.len() 
                // - f[1] as usize
            } else {
                1
            };
        } else {
            let char_shift = i - r[alphabet_index(t.chars().nth(h as usize).unwrap())][i as usize] as isize;
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

    // add len of prefix to the matches
    for i in 0..matches.len() {
        matches[i] += p.len();
    }

    matches
}


pub fn helper_print(line_idx: usize, line: &str, output_strs: Vec<usize>){
    let mut end_idx: isize = -1;
    for (str_start, str_end) in output_strs.iter().enumerate() {
        if *str_end == 0 {
            continue;
        }
        if str_start as isize >= end_idx {
            end_idx = *str_end as isize;
            println!("{}:{}", line_idx, line.get(str_start..*str_end).unwrap());
        }
    }
}

pub fn helper_print_with_start(line_idx: usize, start_positions: Vec<usize>, line: &str, output_strs: Vec<usize>, prefix_len: usize, reverse: bool) {
    for (i, end_idx) in output_strs.iter().enumerate(){
        if *end_idx == 0 {
            continue;
        }
        let start_idx = start_positions[i] - prefix_len;
        if reverse {
            let str_to_print = line.get(start_idx..*end_idx).unwrap();
            println!("{}:{}", line_idx, str_to_print.chars().rev().collect::<String>());
        } else {
            println!("{}:{}", line_idx, line.get(start_idx..*end_idx).unwrap());
        }
    }
}

