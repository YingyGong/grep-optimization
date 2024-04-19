use crate::earley_parse::CFG;
use crate::earley_parse::PrettyPrint;
pub use crate::earley_parse::nt;
pub use crate::earley_parse::tr;

pub fn cfg_for_regular_expression2() -> CFG {
    let mut cfg = CFG::new("RE");

    // base case
    cfg.add_rule("RE", vec![nt("Union")]);

    // union
    cfg.add_rule("Union", vec![nt("Union"), tr('|'), nt("Concat")]);
    cfg.add_rule("Union", vec![]);

    // concatenation 
    cfg.add_rule("Concat", vec![nt("Concat"), nt("Term")]);
    cfg.add_rule("Concat", vec![]);

    // Three Repeat rules
    // Kleene star
    cfg.add_rule("Term", vec![nt("Term"), tr('*')]);

    // plus
    cfg.add_rule("Term", vec![nt("Term"), tr('+')]);

    // question mark
    cfg.add_rule("Term", vec![nt("Term"), tr('?')]);
    
    // cfg.add_rule("Repeat", vec![nt("Term")]);


    // parentheses
    cfg.add_rule("Term", vec![tr('('), nt("Union"), tr(')')]);
    cfg.add_rule("Term", vec![nt("Literal")]);


    // Tab (0x09) and all characters between space (0x20) and tilde (0x7E), 
    // except { |, *, (, ), ., +, ?, \} are regular expressions (literals).
    for c in 0x20u8..=0x7E {
        let ch = c as char;
        if !"{|*()+?\\}".contains(ch) {
            cfg.add_rule("Literal", vec![tr(ch)])
        }
    }

    // escaped special characters
    for &c in &['|', '*', '(', ')', '+', '?', '\\'] {
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
    for c in 0x20u8..=0x7E {
        let ch = c as char;
        if !"{|*()+?\\}".contains(ch) {
            cfg.add_rule("Literal", vec![tr(ch)])
        }
    }

    // escaped special characters
    for &c in &['|', '*', '(', ')', '+', '?', '\\'] {
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

}
