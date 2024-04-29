use std::collections::{HashMap, HashSet};
use std::vec;
use crate::earley_parse::{ASTNode};
use crate::helper::{helper_print};
use std::str::FromStr;
use crate::earley_parse::CFG;
use crate::cfg::cfg_for_regular_expression;
use crate::earley_parse::PrettyPrint;
use std::iter::Filter;
use core::ops::RangeInclusive;
// use indexmap::IndexSet;


#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct State {
    id: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum Transition {
    Epsilon,
    Char(char)
}

#[derive(Debug)]
pub struct NFA {
    states: HashSet<State>,
    transitions: HashMap<State, HashSet<(Transition, State)>>,
    start_state: State,
    accept_states: HashSet<State>,
    next_state_id: usize
}

impl NFA {
    fn new() -> Self {
        let start_state = State { id: 0 };
        let mut nfa = NFA {
            states: HashSet::new(),
            transitions: HashMap::new(),
            start_state: start_state.clone(),
            accept_states: HashSet::new(),
            next_state_id: 1,
        };
        nfa.states.insert(start_state.clone());
        nfa
    }

    fn get_len(&self) -> usize {
        self.states.len()
    }

    fn add_state(&mut self) -> State {
        let state = State { id: self.next_state_id };
        self.next_state_id += 1;
        self.states.insert(state.clone());
        state
    }

    fn add_transition(&mut self, from: State, transition: Transition, to: State) {
        self.transitions.entry(from).or_insert_with(HashSet::new).insert((transition, to));
    }

    fn add_transition_ch_list(&mut self, char_vec: RangeInclusive<u8>, to: State) {
        for c in char_vec {
            let c = c as char;
            self.add_transition(self.start_state.clone(), Transition::Char(c), to.clone());
        }
    }

    fn modify_state_id(self, shift_num: usize) -> Self{
        let mut new_nfa = NFA {
            states: HashSet::new(),
            transitions: HashMap::new(),
            start_state: State { id: 0 },
            accept_states: HashSet::new(),
            next_state_id: self.next_state_id,
        };

        for state in self.states {
            new_nfa.states.insert(State { id: state.id + shift_num });
        }
        new_nfa.start_state = State { id: self.start_state.id + shift_num };

        for state in self.accept_states {
            new_nfa.accept_states.insert(State { id: state.id + shift_num });
        }

        for (state, transitions) in self.transitions {
            let mut new_transitions = HashSet::new();
            for (transition, next_state) in transitions {
                new_transitions.insert((transition.clone(), State { id: next_state.id + shift_num }));
            }
            new_nfa.transitions.insert(State { id: state.id + shift_num }, new_transitions);
        }

        new_nfa
}

    fn from_char(c: char) -> Self {
        let mut nfa = NFA::new();
        let accept_state = nfa.add_state();
        nfa.accept_states.insert(accept_state.clone());
        nfa.add_transition(nfa.start_state.clone(), Transition::Char(c), accept_state);
        nfa
    }

    fn from_char_class(c: char) -> Self {
        let mut nfa = NFA::new();
        let accept_state = nfa.add_state();
        nfa.accept_states.insert(accept_state.clone());

        match c {
            '.' => {
                let all_chars = 0x20u8..=0x7Eu8;
                nfa.add_transition_ch_list(all_chars, accept_state.clone());
                nfa.add_transition(nfa.start_state.clone(), Transition::Char(0x09 as char), accept_state.clone()); // add Tab
            }
            's' => {
                let upperclass_letters = 0x41u8..=0x5Au8;
                let lowerclass_letters = 0x61u8..=0x7Au8;
                nfa.add_transition_ch_list(upperclass_letters, accept_state.clone());
                nfa.add_transition_ch_list(lowerclass_letters, accept_state.clone());
            }
            'S' => {
                let all_except_letters1 = 0x20u8..=0x40u8;
                let all_except_letters2 = 0x5Bu8..=0x60u8;
                let all_except_letters3 = 0x7Bu8..=0x7Eu8;
                nfa.add_transition_ch_list(all_except_letters1, accept_state.clone());
                nfa.add_transition_ch_list(all_except_letters2, accept_state.clone());
                nfa.add_transition_ch_list(all_except_letters3, accept_state.clone());

                nfa.add_transition(nfa.start_state.clone(), Transition::Char(0x09 as char), accept_state.clone()); // add Tab
            }
            'd' => {
                let char_vec = 0x30u8..=0x39;
                nfa.add_transition_ch_list(char_vec, accept_state.clone());
            }
            'D' => {
                let char_vec = 0x20u8..=0x2Eu8;
                nfa.add_transition_ch_list(char_vec, accept_state.clone());
                let char_vec = 0x3Au8..=0x7Eu8;
                nfa.add_transition_ch_list(char_vec, accept_state.clone());

                nfa.add_transition(nfa.start_state.clone(), Transition::Char(0x09 as char), accept_state.clone()); // add Tab
            }
            'w' => {
                let tab = 0x09u8;
                let space = 0x20u8;
                nfa.add_transition(nfa.start_state.clone(), Transition::Char(tab as char), accept_state.clone()); // add Tab
                nfa.add_transition(nfa.start_state.clone(), Transition::Char(space as char), accept_state.clone()); // add Space
            }
            'W' => {
                let all_chars = 0x21u8..=0x7Eu8;
                nfa.add_transition_ch_list(all_chars, accept_state.clone());
            }
            _ => (),
        }
        nfa
    }
    
    fn from_concatenation(nfas: Vec<NFA>) -> Self {
        let mut nfa = NFA::new();
        let mut prev_accept_states: Vec<State> = Vec::new();
        prev_accept_states.push(nfa.start_state.clone());
        for mut n in nfas {
            let to_shift = nfa.get_len();
            n = n.modify_state_id(to_shift);
            nfa.states.extend(n.states);
            nfa.transitions.extend(n.transitions);

            // add transitions from previous accept states to the start state of the next NFA
            for prev_accept_state in prev_accept_states.clone() {
                nfa.add_transition(prev_accept_state, Transition::Epsilon, n.start_state.clone());
            }
            prev_accept_states = n.accept_states.iter().cloned().collect();
        }
        nfa.accept_states = prev_accept_states.into_iter().collect();
        nfa
    }


    fn from_union(nfas: Vec<NFA>) -> Self {
        let mut nfa = NFA::new();
        for mut n in nfas {
            let to_shift = nfa.get_len();
            n = n.modify_state_id(to_shift);
            nfa.states.extend(n.states);
            nfa.transitions.extend(n.transitions);
            nfa.accept_states.extend(n.accept_states);
            nfa.add_transition(nfa.start_state.clone(), Transition::Epsilon, n.start_state);
        }
        nfa
    }

    fn from_kleene_star(mut nfa: NFA) -> Self {
        for accept_state in nfa.accept_states.clone() {
            nfa.add_transition(accept_state.clone(), Transition::Epsilon, nfa.start_state.clone());
        }
        nfa.accept_states.insert(nfa.start_state.clone());
        nfa
    }

    fn from_plus(mut nfa: NFA) -> Self {
        for accept_state in nfa.accept_states.clone() {
            nfa.add_transition(accept_state.clone(), Transition::Epsilon, nfa.start_state.clone());
        }
        nfa
    }

    fn from_question_mark(mut nfa: NFA) -> Self {
        nfa.accept_states.insert(nfa.start_state.clone());
        nfa
    }

    pub fn epsilon_close(&mut self) {
        // println!("Epsilon close");
        let mut old: HashSet<(State, State)> = HashSet::new();
        let mut cur: HashSet<(State, State)> = HashSet::new();
        // find all episolon transitions from all state
        for state in &self.states {
            if let Some(transition) = self.transitions.get_mut(&state) {
                let transition_copy = transition.clone();
                for (t, next_state) in transition_copy {
                    if t == Transition::Epsilon {
                        cur.insert((state.clone(), next_state.clone()));
                    }
                }
                transition.retain(|(t, _)| *t != Transition::Epsilon);
            }
        }

        while old != cur {
            old = cur.clone();
            cur = HashSet::new();
            for (state, next_state) in old.iter() {
                cur.insert((state.clone(), next_state.clone()));
            }
            for (state, next_state) in old.iter() {
                // find all starting with next_state
                for (state_b, state_c) in old.iter() {
                    if state_b == next_state {
                        cur.insert((state.clone(), state_c.clone()));
                    }
                }
            }
        }

        for (state, next_state) in cur.iter() {
            let transition_copy = self.transitions.get(next_state).unwrap_or(&HashSet::new()).clone();
            for (t, state_c) in transition_copy {
                if t != Transition::Epsilon 
                {   
                    self.add_transition(state.clone(), t.clone(), state_c.clone());
                }
            }
            // if next_state is accept state, add state to accept states
            if self.accept_states.contains(next_state) {
                self.accept_states.insert(state.clone());
            }
        }

        // remove all repetitive transitions
        for state in &self.states {
            if let Some(transition) = self.transitions.get_mut(state) {
                let mut seen: HashSet<(Transition, State)> = HashSet::new();
                transition.retain(|(transition_char, next_state)| seen.insert((transition_char.clone(), next_state.clone())));
            }
        }


    }

    pub fn remove_unreachable_states(&mut self) {
        let mut reachable_states: HashSet<State> = HashSet::new();
        let mut stack: Vec<State> = Vec::new();
        stack.push(self.start_state.clone());
        reachable_states.insert(self.start_state.clone());
        while let Some(state) = stack.pop() {
            if let Some(transitions) = self.transitions.get(&state) {
                for (_, next_state) in transitions {
                    if !reachable_states.contains(next_state) {
                        reachable_states.insert(next_state.clone());
                        stack.push(next_state.clone());
                    }
                }
            }
        }
        let unreachable_states: Vec<State> = self.states.difference(&reachable_states).cloned().collect();
        for state in unreachable_states {
            self.states.remove(&state);
            self.transitions.remove(&state);
            self.accept_states.remove(&state);
        }

    }


    pub fn from_regex(node: &ASTNode) -> Self{
        match node {
            ASTNode::NonTerminal { sym, children } =>
            match *sym {
                "RE" => {
                    NFA::from_regex(&children[0])
                }
                "Concat" => {
                    let left = NFA::from_regex(&children[0]);
                    let right = NFA::from_regex(&children[1]);
                    NFA::from_concatenation(vec![left, right])
                }
                "Union" => {
                    assert!(children.len() == 3);
                    let left = NFA::from_regex(&children[0]);
                    let right = NFA::from_regex(&children[2]);
                    NFA::from_union(vec![left, right])
                }
                "Repeat" => {
                    let nfa = NFA::from_regex(&children[0]);
                    match children[1].unwrap_terminal() {
                        '*' => NFA::from_kleene_star(nfa),
                        '+' => NFA::from_plus(nfa),
                        '?' => NFA::from_question_mark(nfa),
                        _ => panic!("Invalid repeat operator"),
                    }
                }
                "Term" => {
                    let len_children = children.len();
                    if len_children == 1 {
                        NFA::from_regex(&children[0])
                    } else {
                        // skip '(' and ')'
                        NFA::from_regex(&children[1])
                    }
                }
                "Literal" => {
                    if children.len() == 1 {
                        NFA::from_regex(&children[0])
                    } else { // special characters or character classes
                        let character_class = vec!['s', 'S', 'd', 'D', 'w', 'W'];
                        let c = children[1].unwrap_terminal();
                        if character_class.contains(&c) {
                            NFA::from_char_class(c)
                        }
                        else{
                            NFA::from_char(c)
                        }
                    }
                }
                _ => // print out sym
                panic!("Invalid non-terminal {}", sym),
            }
            ASTNode::Terminal (terminal) => 
            match terminal {
                '.' => NFA::from_char_class('.'),
                _ => NFA::from_char(*terminal),
            }
        }
    }

    pub fn debug_helper(&self) {
        println!("States: {:?}", self.states);
        println!("Transitions: {:?}", self.transitions);
        println!("Start state: {:?}", self.start_state);
        println!("Accept states: {:?}", self.accept_states);
    }


    pub fn check_str_without_start(&self, input_str: &str) -> Vec<usize> {
        let mut cur_positions: HashMap<State, usize> = HashMap::new();
        let mut next_positions: HashMap<State, usize> = HashMap::new();

        let line_len = input_str.len();

        // strings to return
        let mut matched_strs: Vec<usize> = vec![0; line_len];
        // println!("initial matched_strs: {:?}", matched_strs);
       
        for (i, c) in input_str.char_indices() {
            next_positions.clear();
            
            if !cur_positions.contains_key(&self.start_state) {
                cur_positions.insert(self.start_state.clone(), i);
            }

            if self.accept_states.contains(&self.start_state) {
                matched_strs[i] = i + 1;
            }

            // println!("positions before iter {}: {:?}", i, cur_positions);
            // for all possible current states
            for (state, start_position) in cur_positions.iter() {
                if let Some(transitions) = self.transitions.get(state) {
                    for (transition, next_state) in transitions {
                        match transition {
                            // if the character can lead to a next state by a valid transition
                            Transition::Char(c1) if *c1 == c => {
                                
                                // get the starting positions of the current state
                                // if the next state is not in the hashmap, add the starting position of the current state
                                if !next_positions.contains_key(next_state) {
                                    next_positions.insert(next_state.clone(), start_position.clone());
                                }
                                // else check if the cur start_position can be smaller
                                else {
                                    if let Some(next_start_position) = next_positions.get_mut(next_state) {
                                        if *next_start_position > *start_position {
                                            *next_start_position = *start_position;
                                        }
                                    }
                                }
                                
                            }
                            _ => (),
                        }
                    }
                }
            }
            
            // switch the hashmap
            // temp_positions = cur_positions;
            std::mem::swap(&mut cur_positions, &mut next_positions);
            // println!("positions after iter {}: {:?}", i, cur_positions);

            // check any matched
            for accept_state in &self.accept_states {
                if let Some(start_pos) = cur_positions.get_mut(accept_state) {
                    matched_strs[*start_pos] = i + 1;
                    // println!("matched from index {} at char {}: {}", *start_pos, i, input_str[*start_pos..(i+1)].to_string());
                }
            }


    }
    matched_strs
}


    pub fn check_str_with_start(&self, starting_idxes: &Vec<usize>, input_str: &str, prefix_len: usize) -> Vec<usize> {
        assert!(!starting_idxes.is_empty());

        let max_match = starting_idxes.len();
        let mut matched_strs: Vec<usize> = vec![0; max_match];
        let mut end_idx = 0;

        for (i, start_idx) in starting_idxes.iter().enumerate() {
            // println!("start_idx: {} and end_idx {}", start_idx, end_idx);
            if *start_idx - prefix_len < end_idx {
                continue;
            }
            let mut cur_states = HashSet::new();
            cur_states.insert(&self.start_state);
            if self.accept_states.contains(&self.start_state) {
                end_idx = *start_idx;
                matched_strs[i] = end_idx;
            }

            // start iterate from the start_idx
            for (j, c) in input_str.char_indices().skip_while(|(index, _)| *index < *start_idx) {
                let mut next_states = HashSet::new();
                for state in cur_states.iter() {
                    if let Some(transitions) = self.transitions.get(state) {
                        for (transition, next_state) in transitions {
                            match transition {
                                Transition::Char(c1) if *c1 == c => {
                                    next_states.insert(next_state);
                                }
                                _ => (),
                            }
                        }
                    }
                }
                cur_states = next_states;
                if cur_states.is_empty() {
                    break;
                }
                if cur_states.iter().any(|state| self.accept_states.contains(state)) {
                    end_idx = j + 1;
                    matched_strs[i] = end_idx;
                }
            }
        }
        matched_strs
    }


    pub fn find_prefix_from_nfa(&mut self) -> String {
        let mut cur_state_vec = vec![self.start_state.clone()];
        let mut prefix = String::new();

        assert!(cur_state_vec.len() == 1);

        'outer: while ! cur_state_vec.is_empty() {
            let mut common_char: Option<char> = None;
            let mut next_state_vec: Vec<State> = Vec::new();
            for (i, cur_state) in cur_state_vec.iter().enumerate() {
                if self.accept_states.contains(cur_state) {
                    break 'outer;
                }
                if self.transitions.contains_key(&cur_state) {
                    let transitions = self.transitions.get(&cur_state).unwrap();
                    if i == 0 {
                        let mut iter = transitions.iter();
                        if let Some((transition, state)) = iter.next() {
                            if state == cur_state {
                                break 'outer;
                            }
                            if let Transition::Char(c) = transition {
                                common_char = Some(*c);
                                next_state_vec.push(state.clone());
                                for (key, value) in iter {
                                    if let Transition::Char(c) = key {
                                        if *c != common_char.unwrap() {
                                            break 'outer;
                                        }
                                        next_state_vec.push(value.clone());
                                    }
                                    else {
                                        break;
                                    }
                                }
                            }
                        }
                        else {
                            break;
                        }
                    }
                    else {
                        let mut iter = transitions.iter();
                        for (transition, state) in iter {
                            if state == cur_state {
                                break 'outer;
                            }
                            if let Transition::Char(c) = transition {
                                if *c != common_char.unwrap()  {
                                    break 'outer;
                                }
                                next_state_vec.push(state.clone());
                            }
                            else {
                                break;
                            }
                        }
                    }
                }
                else {
                    break 'outer;
                }
            }
            prefix.push(common_char.unwrap());
            cur_state_vec = next_state_vec;
            
    }

    // modify nfa to have prefix start states
    // let ori_start_state = self.start_state.clone();
    self.start_state = State { id: self.next_state_id };
    self.next_state_id += 1;
    self.states.insert(self.start_state.clone());
    for cur_state in cur_state_vec {
        self.add_transition(self.start_state.clone(), Transition::Epsilon, cur_state);
    }
    // epsilon_close
    self.epsilon_close();
    // remove unreachable states
    self.remove_unreachable_states();

    prefix

    }

    
    
}


impl Clone for NFA {
    fn clone(&self) -> Self {
        let mut new_nfa = NFA {
            states: self.states.clone(),
            transitions: self.transitions.clone(),
            start_state: self.start_state.clone(),
            accept_states: self.accept_states.clone(),
            next_state_id: self.next_state_id,
            // prefix_start_states: self.prefix_start_states.clone(),
        };
        new_nfa
    }
}

pub fn nfa_from_reg(regex: &str) -> NFA {
    let cfg = cfg_for_regular_expression();
    let ast = cfg.parse(regex).unwrap().collapse();
    let mut nfa = NFA::from_regex(&ast);
    NFA::epsilon_close(&mut nfa);
    NFA::remove_unreachable_states(&mut nfa);
    nfa
}



#[cfg(test)]
mod test {

    use super::*;
    #[test]
    fn test_shift_nfa() {
        println!("Test shift nfa");
        let mut nfa = NFA::new();
        nfa.add_state();
        nfa.add_state();
        nfa.add_transition(State {id: 0}, Transition::Char('a'), State {id: 1});
        let nfa = nfa.modify_state_id(5);
        nfa.debug_helper();
        println!("\n");
    }

    #[test]
    fn test_from_char() {
        let nfa = NFA::from_char('a');
        println!("Test from char");
        nfa.debug_helper();
        println!("\n");
    }


    #[test]
    fn test_single_char_nfa() {
        let nfa = NFA::from_char('a');
        println!("Test single char NFA");
        nfa.debug_helper();
        println!("\n");
    }

    #[test]
    fn test_union() {
        println!("Test union");
        let nfa1 = NFA::from_char('a');
        let nfa2 = NFA::from_char('b');
        let nfa = NFA::from_union(vec![nfa1, nfa2]);
        assert!(nfa.states.len() == 5);
        assert!(nfa.accept_states.len() == 2);
    }

    #[test]
    fn test_concatenation() {
        println!("Test concatenation");
        let nfa1 = NFA::from_char('a');
        nfa1.debug_helper();
        println!("NFA 1 \n");

        let nfa2 = NFA::from_char('b');
        nfa2.debug_helper();
        println!("NFA 2 \n");

        let nfa = NFA::from_concatenation(vec![nfa1, nfa2]);
        nfa.debug_helper();
    }

    #[test]
    fn test_kleene_star() {
        println!("Test kleene star");
        let nfa = NFA::from_char('a');
        let nfa = NFA::from_kleene_star(nfa);
        nfa.debug_helper();
    }

    #[test]
    fn test_union_and_concat(){
        println!("Test union and concatenation");
        let regex = "ac|b";
        let cfg = cfg_for_regular_expression();
        let ast = cfg.parse(regex).unwrap().collapse();
        let nfa = NFA::from_regex(&ast);
        nfa.debug_helper();
        println!("\n After Episolon closure\n");
    }

    #[test]
    fn test_parentheses() {
        println!("Test parentheses");
        let regex = "a(b|c)";
        let cfg = cfg_for_regular_expression();
        let ast = cfg.parse(regex).unwrap().collapse();
        let nfa = NFA::from_regex(&ast);
        nfa.debug_helper();
    }

    // #[test]
    // fn test_digit() {
    //     println!("Test Digit Class");
    //     let regex = "\\dab";
    //     let cfg = cfg_for_regular_expression();
    //     let ast = cfg.parse(regex).unwrap().collapse();
    //     println!("{:#?}", PrettyPrint(&ast));
    //     let nfa = nfa_from_reg("\\dab");
    //     nfa.debug_helper();
    //     print!("{:?}", nfa.check_str_princeton("abcabf"));
    // }


    // #[test]
    // fn test_digit_2() {
    //     println!("Test Digit Class");
    //     let regex = "\\d";
    //     let cfg = cfg_for_regular_expression();
    //     let ast = cfg.parse(regex).unwrap().collapse();
    //     println!("{:#?}", PrettyPrint(&ast));
    //     let nfa = nfa_from_reg(regex);
    //     nfa.debug_helper();
    //     let nfa = NFA::epsilon_close(nfa);
    //     nfa.debug_helper();
    //     print!("{:?}", nfa.check_str_princeton("d12345"));
    // }

    // #[test]
    // fn test_check_string_1() {
    //     println!("Test check string return string vec 1");
    //     let nfa = nfa_from_reg("ab|c");
    //     let nfa = NFA::epsilon_close(nfa);
    //     print!("{:?}", nfa.check_str_princeton("abab"));
    //     print!("{:?}", nfa.check_str_princeton("cab"));
    //     print!("{:?}", nfa.check_str_princeton("c"));
    //     print!("{:?}", nfa.check_str_princeton("cabcabcab"));
    // }

    // #[test]
    // fn test_check_string_2() {
    //     println!("Test check string return string vec 2");
    //     let nfa = nfa_from_reg("ab");
    //     let nfa = NFA::epsilon_close(nfa);
    //     nfa.debug_helper();
    //     println!("");
    //     print!("{:?}", nfa.check_str_princeton("abab"));
    // }

    // #[test]
    // fn test_check_string_kleen() {
    //     println!("Test check string return string vec kleen");
    //     let nfa = nfa_from_reg("c(ab)*");
    //     let nfa = NFA::epsilon_close(nfa);
    //     nfa.debug_helper();
    //     println!("");
    //     print!("{:?}", nfa.check_str_princeton("a"));
    //     print!("{:?}", nfa.check_str_princeton("bab"));
    //     print!("{:?}", nfa.check_str_princeton("cabab"));
    // }

    // #[test]
    // fn test_check_string_kleen_2() {
    //     println!("Test check string return string vec kleen");
    //     let nfa = nfa_from_reg("(b)*");
    //     let nfa = NFA::epsilon_close(nfa);
    //     nfa.debug_helper();
    //     println!("");
    //     print!("{:?}", nfa.check_str_princeton("a"));
    //     print!("{:?}", nfa.check_str_princeton("bab"));
    //     print!("{:?}", nfa.check_str_princeton("cabab"));
    // }

    // #[test]
    // fn test_check_string_question_mark() {
    //     println!("Test question mark");
    //     let nfa = nfa_from_reg("ka?");
    //     let nfa = NFA::epsilon_close(nfa);
    //     nfa.debug_helper();
    //     println!("");
    //     print!("{:?}", nfa.check_str_princeton("ka"));
    //     print!("{:?}", nfa.check_str_princeton("k"));
    // }

    // #[test]
    // fn test_check_string_3() {
    //     println!("Test check string return string vec 3");
    //     let nfa = nfa_from_reg("ab|c");
    //     let nfa = NFA::epsilon_close(nfa);
    //     println!("{:?}", nfa.check_str_princeton("ab"));
    //     println!("{:?}", nfa.check_str_princeton("cab"));
    //     println!("{:?}", nfa.check_str_princeton("jghfhjfckhuieabkc"));
    // }

    // #[test]
    // fn test_epsilon_closure() {
    //     println!("Test epsilon closure");
    //     let mut nfa = NFA::new_with_n_states(4);
    //     let state0 = State { id: 0 };
    //     let state1 = State { id: 1 };
    //     let state2 = State { id: 2 };
    //     let state3 = State { id: 3 };
    //     let state4 = State { id: 4 };

    //     nfa.add_transition(state0.clone(), Transition::Char('0'), state0.clone());
    //     nfa.add_transition(state0.clone(), Transition::Epsilon, state1.clone());
    //     // 1, 1, 1
    //     nfa.add_transition(state1.clone(), Transition::Char('1'), state1.clone());
    //     // 1, epsilon, 2
    //     nfa.add_transition(state1.clone(), Transition::Epsilon, state2.clone());
    //     // 2, 0, 2
    //     nfa.add_transition(state2.clone(), Transition::Char('0'), state2.clone());
    //     // 2, epsilon, 3
    //     nfa.add_transition(state2.clone(), Transition::Epsilon, state3.clone());
    //     // 2, 0, 4
    //     nfa.add_transition(state2.clone(), Transition::Char('0'), state4.clone());
    //     nfa.accept_states.insert(state3.clone());
    //     nfa.accept_states.insert(state4.clone());
    //     let nfa = NFA::epsilon_close(nfa);
    //     nfa.debug_helper();
    // }

    #[test]
    fn test_rename_states() {
        let mut nfa = nfa_from_reg("(fd)+|fl");
        nfa.debug_helper();
        nfa.rename_states();
        nfa.debug_helper();
    }

    #[test]
    fn test_prefix_nfa() {
        let nfa = nfa_from_reg("(fd)+|fl");
        nfa.debug_helper();
    }

    #[test]
    fn test_prefix_from_nfa() {
        let mut nfa = nfa_from_reg("(fd)+|fl");
        nfa.debug_helper();
        let prefix = nfa.find_prefix_from_nfa();
        println!("Prefix: {}", prefix);
    }

    #[test]
    fn test_prefix_from_nfa_2() {
        let mut nfa = nfa_from_reg("a(b|c)");
        nfa.debug_helper();
        let prefix = nfa.find_prefix_from_nfa();
        println!("Prefix: {}", prefix);
    }

    #[test]
    fn test_prefix_from_nfa_3() {
        let mut nfa = nfa_from_reg("(abc|abc)de");
        nfa.debug_helper();
        let prefix = nfa.find_prefix_from_nfa();
        println!("Prefix: {}", prefix);
        nfa.debug_helper()
    }

    #[test]
    fn test_prefix_from_nfa_4() {
        let mut nfa = nfa_from_reg("ab*");
        nfa.debug_helper();
        let prefix = nfa.find_prefix_from_nfa();
        println!("Prefix: {}", prefix);
        nfa.debug_helper()
    }

    #[test]
    fn test_prefix_from_nfa_5() {
        let mut nfa = nfa_from_reg("ab+");
        nfa.debug_helper();
        let prefix = nfa.find_prefix_from_nfa();
        println!("Prefix: {}", prefix);
    }

    #[test]
    fn test_prefix_from_nfa_6() {
        let mut nfa = nfa_from_reg("(fd)+|fl");
        nfa.debug_helper();
        let prefix = nfa.find_prefix_from_nfa();
        println!("Prefix: {}", prefix);
    }

    #[test]
    fn test_prefix_from_nfa_7() {
        let mut nfa = nfa_from_reg("ab*a");
        nfa.debug_helper();
        let prefix = nfa.find_prefix_from_nfa();
        println!("Prefix: {}", prefix);
    }


    #[test]
    fn test_prefix_from_nfa_8() {
        let mut nfa = nfa_from_reg(".*");
        nfa.debug_helper();
        let prefix = nfa.find_prefix_from_nfa();
        println!("Prefix: {}", prefix);
    }

    #[test]
    fn test_prefix_from_nfa_9() {
        let mut nfa = nfa_from_reg("a?a?a?aa");
        nfa.debug_helper();
        let prefix = nfa.find_prefix_from_nfa();
        println!("Prefix: {}", prefix);
        nfa.debug_helper();
    }

    #[test]
    fn test_check_str_without_start_1() {
        let nfa = nfa_from_reg("ab|c");
        nfa.debug_helper();
        let matched_strs = nfa.check_str_without_start("ab");
        println!("{:?}", matched_strs);
    }

    #[test]
    fn test_check_str_without_start_2() {
        let nfa = nfa_from_reg("k*");
        nfa.debug_helper();
        let str = "kabk";
        let matched_strs = nfa.check_str_without_start(&str);
        helper_print(1, &str, matched_strs);
    }

    #[test]
    fn test_check_str_without_start_3() {
        let nfa = nfa_from_reg("ab*");
        nfa.debug_helper();
        let str = "ababbabbbab";
        let matched_strs = nfa.check_str_without_start(&str);
        println!("{:?}", matched_strs);
        helper_print(1, &str, matched_strs);
    }
}
