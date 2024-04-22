use crate::earley_parse::ASTNode;
use crate::earley_parse::CFG;
use crate::earley_parse::PrettyPrint;
pub use crate::earley_parse::nt;
pub use crate::earley_parse::tr;

// left associative cfg
pub fn cfg_for_regular_expression() -> CFG {
    let mut cfg = CFG::new("RE");

    // base case
    cfg.add_rule("RE", vec![nt("Union")]);

    // union
    cfg.add_rule("Union", vec![nt("Union"), tr('|'), nt("Concat")]);
    cfg.add_rule("Union", vec![nt("Concat")]);

    // concatenation 
    cfg.add_rule("Concat", vec![nt("Concat"), nt("Repeat")]);
    cfg.add_rule("Concat", vec![nt("Repeat")]);

    // Three Repeat rules
    // Kleene star
    cfg.add_rule("Repeat", vec![nt("Term"), tr('*')]);

    // pluss
    cfg.add_rule("Repeat", vec![nt("Term"), tr('+')]);

    // question mark
    cfg.add_rule("Repeat", vec![nt("Term"), tr('?')]);
    
    cfg.add_rule("Repeat", vec![nt("Term")]);


    // parentheses
    cfg.add_rule("Term", vec![tr('('), nt("Union"), tr(')')]);
    cfg.add_rule("Term", vec![nt("Literal")]);


    // Tab (0x09) and all characters between space (0x20) and tilde (0x7E), 
    // except { |, *, (, ), ., +, ?, \} are regular expressions (literals).
    for c in 0x20u8..=0x80 {
        let ch = c as char;
        if !"{|*()+?\\.}".contains(ch) {
            cfg.add_rule("Literal", vec![tr(ch)])
        }
    }
    cfg.add_rule("Literal", vec![tr(0x09u8 as char)]); //tab

    // escaped special characters
    for &c in &['|', '*', '(', ')', '+', '?', '\\', '{', '}', '.'] {
        cfg.add_rule("Literal", vec![tr('\\'), tr(c)]);
    }

    // dot (any character)
    cfg.add_rule("Literal", vec![tr('.')]);

    // character classes
    cfg.add_rule("Literal", vec![tr('\\'), tr('s')]); // whitespace
    cfg.add_rule("Literal", vec![tr('\\'), tr('S')]); // non-whitespace
    cfg.add_rule("Literal", vec![tr('\\'), tr('d')]); // digit
    cfg.add_rule("Literal", vec![tr('\\'), tr('D')]); // non-digit
    cfg.add_rule("Literal", vec![tr('\\'), tr('w')]); // word character (alphanumeric + underscore)
    cfg.add_rule("Literal", vec![tr('\\'), tr('W')]); // non-word character

    cfg
}

pub fn prefix_extract(node: &ASTNode) -> String {
    match node {
        ASTNode::NonTerminal { sym, children } =>
        match *sym {
            "RE" => {
                prefix_extract(&children[0])
            },
            "Union" => {
                assert!(children.len() == 3);
                let str1 = &prefix_extract(&children[0]);
                let str2 = &prefix_extract(&children[2]);
                two_str_common_prefix(str1, str2).0
            },
            "Concat" => {
                let mut result = String::new();
                match children[0] {
                    ASTNode::NonTerminal { sym, .. } if sym == "Repeat" => {
                        prefix_extract(&children[0])
                    },
                    _ => {
                        result.push_str(&prefix_extract(&children[0]));
                        result.push_str(&prefix_extract(&children[1]));
                        result
                    }
                }
            },
            // no action for non-fixed symbols
            "Repeat" => {
                match children[1].unwrap_terminal() {
                    '*' => String::new(),
                    '+' => prefix_extract(&children[0]),
                    '?' => String::new(),
                    _ => panic!("Invalid repeat operator"),
                }
            },
            "Term" => {
                let len_children = children.len();
                if len_children == 1 {
                    prefix_extract(&children[0])
                } else {
                    // skip '(' and ')'
                    prefix_extract(&children[1])
                }
            },
            "Literal" => {
                if children.len() == 1 {
                    prefix_extract(&children[0])
                } else { // special characters or character classes
                    let character_class = vec!['s', 'S', 'd', 'D', 'w', 'W'];
                    let c = children[1].unwrap_terminal();
                    if character_class.contains(&c) {
                        String::new()
                    }
                    else{
                        prefix_extract(&children[1])
                    }
                }
            }
            _ => String::new(),
        },
        ASTNode::Terminal (terminal) => 
            match terminal {
                '.' => String::new(),
                _ => terminal.to_string(),
            }
        }
}

pub fn prefix_and_remainder_extract(node: &ASTNode) -> (String, String) {
    match node {
        ASTNode::NonTerminal { sym, children } => match *sym {
            "RE" => prefix_and_remainder_extract(&children[0]),
            "Union" => {
                assert!(children.len() == 3);
                let (prefix1, remainder1) = prefix_and_remainder_extract(&children[0]);
                let (prefix2, remainder2) = prefix_and_remainder_extract(&children[2]);
                let (common_prefix, remainder1, remainder2) = two_str_common_prefix(&prefix1, &prefix2);
                let remainder = format!("{}|{}", remainder1, remainder2);
                (common_prefix, remainder)
            },
            "Concat" => {
                let mut result = String::new();
                match children[0] {
                    ASTNode::NonTerminal { sym, .. } if sym == "Repeat" => {
                        let (prefix1, remainder1) = prefix_and_remainder_extract(&children[0]);
                        let (_, remainder2) = prefix_and_remainder_extract(&children[1]);
                        (prefix1, format!("{}{}", remainder1, remainder2))
                    },
                    _ => {
                        let (prefix1, remainder1) = prefix_and_remainder_extract(&children[0]);
                        let (prefix2, remainder2) = prefix_and_remainder_extract(&children[1]);
                        let combined_prefix = format!("{}{}", prefix1, prefix2);
                        let remainder = format!("{}{}", remainder1, remainder2);
                        (combined_prefix, remainder)
                    }
                }
            },
            "Repeat" => {
                match children[1].unwrap_terminal() {
                    '*' | '?' => (String::new(), format!("({}){}", prefix_and_remainder_extract(&children[0]).0, children[1].unwrap_terminal())),
                    '+' => {
                        prefix_and_remainder_extract(&children[0])
                    },
                    _ => panic!("Invalid repeat operator"),
                }
            },
            "Term" => {
                if children.len() == 1 {
                    prefix_and_remainder_extract(&children[0])
                } else {
                    // skip '(' and ')'
                    let (prefix, remainder) = prefix_and_remainder_extract(&children[1]);
                    if prefix.is_empty() {
                        (prefix, format!("({})", remainder))
                    } else {
                        (prefix, remainder)
                    }
                }
            },
            "Literal" => {
                if children.len() == 1 {
                    prefix_and_remainder_extract(&children[0])
                } else {
                    let c = children[1].unwrap_terminal();
                    if vec!['s', 'S', 'd', 'D', 'w', 'W'].contains(&c) {
                        (String::new(),format!("{}{}", children[0].unwrap_terminal(), children[1].unwrap_terminal()))
                    } else {
                        (c.to_string(), String::new())
                    }
                }
            }
            _ => (String::new(), String::new()),
        },
        ASTNode::Terminal(terminal) => {
            match *terminal {
                '.' | '*' | '+' | '?' => (String::new(), terminal.to_string()),
                _ => (terminal.to_string(), String::new()),
            }
        }
    }
}

pub fn check_plus_sign(s: &str) -> (String, String) {
    // split at the first '+'
    let mut iter = s.chars();
    let mut before_plus = String::new();
    let mut remainder = String::new();
    let mut last_char: Option<char> = None;
    loop {
        match iter.next() {
            Some('+') => {
                if last_char == Some('\\') {
                    before_plus.push('+');
                } else {
                    before_plus.push('+');
                    break;
                }
            }
            Some(c) => {
                before_plus.push(c);
                last_char = Some(c);
            }
            None => {
                break;
            }
        }
    }
    for c in iter {
        remainder.push(c);
    }
    (before_plus, remainder)
}

pub fn prefix_and_remainder_extract_after_plus(s: &str) -> (String, String) {
    let (mut before_plus, after_plus) = check_plus_sign(s);
    if before_plus.is_empty() {
        let (prefix, remainder) = prefix_and_remainder_extract(&cfg_for_regular_expression().parse(s).unwrap().collapse());
        return (prefix, remainder);
    }
    // println!("{:#?}",PrettyPrint(&cfg_for_regular_expression().parse(&before_plus).unwrap().collapse()));
    let (prefix, remainder) = prefix_and_remainder_extract(&cfg_for_regular_expression().parse(&before_plus).unwrap().collapse());
    let remainder = format!("{}{}", remainder, after_plus);
    (prefix, remainder)
}


// return the common prefix, together with the remainder from s1 and s2
fn two_str_common_prefix(s1: &str, s2: &str) -> (String, String, String) {
    let mut iter1 = s1.chars();
    let mut iter2 = s2.chars();
    let mut common_prefix = String::new();
    let mut remainder1 = String::new();
    let mut remainder2 = String::new();
    loop {
        match (iter1.next(), iter2.next()) {
            (Some(c1), Some(c2)) 
            if c1 == c2 => {
                common_prefix.push(c1);}
            (Some(c1), Some(c2)) => {
                remainder1.push(c1);
                remainder2.push(c2);
            }
            // if c1 is None, then c2 left
            (None, Some(c2)) => {
                remainder2.push(c2);
            }
            // if c2 is None, then c1 left
            (Some(c1), None) => {
                remainder1.push(c1);
            }
            (None, None) => {
                break;
            }
        }
    }
    (common_prefix, remainder1, remainder2)
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_cfg_for_regular_expression() {
        let cfg = cfg_for_regular_expression();
        let result = cfg.parse("ab*|c+");
        assert!(result.is_some());
        // println!("{:#?}", PrettyPrint(&result.unwrap().collapse()));
    }

    #[test]
    fn test_parentheses() {
        let cfg = cfg_for_regular_expression();
        let result = cfg.parse("a(b|c)");
        assert!(result.is_some());
        println!("{:#?}", PrettyPrint(&result.unwrap().collapse()));
    }

    #[test]
    fn test_character_classes() {
        let cfg = cfg_for_regular_expression();
        let result = cfg.parse(r"\s\d\D\w\W");
        assert!(result.is_some());
        println!("{:#?}", PrettyPrint(&result.unwrap().collapse()));
    }

    #[test]
    fn test_special_characters() {
        let cfg = cfg_for_regular_expression();
        let result = cfg.parse(r"\*");
        assert!(result.is_some());
        println!("{:#?}", PrettyPrint(&result.unwrap().collapse()));
    }

    #[test]
    fn test_prefix_extract_1() {
        let cfg = cfg_for_regular_expression();
        let result = cfg.parse(r"ab|ac");
        assert!(result.is_some());
        let tree = result.unwrap().collapse();
        println!("{:#?}", PrettyPrint(&tree));
        let (prefix, rest) = prefix_and_remainder_extract(&tree);
        println!("{} and {}", prefix, rest);
    }

    #[test]
    fn test_prefix_extract_2() {
        let cfg = cfg_for_regular_expression();
        let result = cfg.parse(r"foo\*(d|l)");
        assert!(result.is_some());
        let tree = result.unwrap().collapse();
        println!("{:#?}", PrettyPrint(&tree));
        let (prefix, rest) = prefix_and_remainder_extract(&tree);
        println!("{} and {}", prefix, rest);
    }

    #[test]
    fn test_prefix_extract_3() {
        let cfg = cfg_for_regular_expression();
        let result = cfg.parse(r"(na)+bc");
        assert!(result.is_some());
        let tree = result.unwrap().collapse();
        println!("{:#?}", PrettyPrint(&tree));
        let (prefix, rest) = prefix_and_remainder_extract_after_plus("(na)+bc");
        println!("{} and {}", prefix, rest);
    }

    #[test]
    fn test_prefix_and_remainder_extract_after_plus() {
        let mut r = String::new();
        // for c in 0x20u8..=0x80 {
        //     let ch = c as char;
        //     if !"{|*()+?\\.}".contains(ch) {
        //         r.push(c as char);
        //     }
        // }
        for &c in &['|', '*', '(', ')', '+', '?', '\\', '{', '}', '.'] {
            r.push('\\');
            r.push(c );
        }
        let (prefix, remainder) = prefix_and_remainder_extract_after_plus(&r);
        println!("{} and {}", prefix, remainder);
    }

    #[test]
    fn test_kleene_star_prefix() {
        let cfg = cfg_for_regular_expression();
        let result = cfg.parse(r"(ab)*");
        assert!(result.is_some());
        let tree = result.unwrap().collapse();
        println!("{:#?}", PrettyPrint(&tree));
        let (prefix, rest) = prefix_and_remainder_extract(&tree);
        println!("{} and {}", prefix, rest);
    }
}
