use std::collections::{HashMap, HashSet};
use crate::earley_parse::{ASTNode};
use std::str::FromStr;
use crate::earley_parse::CFG;
use crate::cfg::cfg_for_regular_expression;
use crate::earley_parse::PrettyPrint;
use std::iter::Filter;


#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct State {
    id: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum Transition {
    Epsilon,
    Char(char),
}

#[derive(Debug)]
pub struct NFA {
    states: HashSet<State>,
    transitions: HashMap<State, Vec<(Transition, State)>>,
    start_state: State,
    accept_states: HashSet<State>,
    next_state_id: usize,
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

    fn new_with_n_states(n: usize) -> Self {
        let mut nfa = NFA::new();
        for _ in 1..(n+1) {
            nfa.add_state();
        }
        nfa
    }

    fn add_transition(&mut self, from: State, transition: Transition, to: State) {
        self.transitions.entry(from).or_insert_with(Vec::new).push((transition, to));
    }

    fn from_char(c: char) -> Self {
        let mut nfa = NFA::new();
        let accept_state = nfa.add_state();
        nfa.accept_states.insert(accept_state.clone());
        nfa.add_transition(nfa.start_state.clone(), Transition::Char(c), accept_state);
        nfa
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
            let mut new_transitions = Vec::new();
            for (transition, next_state) in transitions {
                new_transitions.push((transition.clone(), State { id: next_state.id + shift_num }));
            }
            new_nfa.transitions.insert(State { id: state.id + shift_num }, new_transitions);
        }

        new_nfa
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

    pub fn epsilon_close(mut nfa: NFA) -> Self {
        let mut old: HashSet<(State, State)> = HashSet::new();
        let mut cur: HashSet<(State, State)> = HashSet::new();
        // find all episolon transitions from all state
        for state in &nfa.states {
            if let Some(transition) = nfa.transitions.get_mut(state) {
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
            let transition_copy = nfa.transitions.get(next_state).unwrap_or(&Vec::new()).clone();
            for (t, state_c) in transition_copy {
                if t != Transition::Epsilon 
                {   
                    nfa.add_transition(state.clone(), t.clone(), state_c.clone());
                }
            }
            // if next_state is accept state, add state to accept states
            if nfa.accept_states.contains(next_state) {
                nfa.accept_states.insert(state.clone());
            }
        }

        // remove all repetitive transitions
        for state in &nfa.states {
            if let Some(transition) = nfa.transitions.get_mut(state) {
                let mut seen: HashSet<State> = HashSet::new();
                transition.retain(|(_, next_state)| seen.insert(next_state.clone()));
            }
        }

        nfa

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
                    NFA::from_regex(&children[0])
                }
                _ => // print out sym
                panic!("Invalid non-terminal {}", sym),
            }
            ASTNode::Terminal (terminal) => NFA::from_char(*terminal),
        }
    }

    fn debug_helper(&self) {
        println!("States: {:?}", self.states);
        println!("Transitions: {:?}", self.transitions);
        println!("Start state: {:?}", self.start_state);
        println!("Accept states: {:?}", self.accept_states);
    }

    pub fn check_str2(&self, input_str: &str) -> bool {
        let mut current_states: HashSet<State> = HashSet::new();
        current_states.insert(self.start_state.clone());
        for c in input_str.chars() {
            let mut next_states: HashSet<State> = HashSet::new();
            for state in current_states {
                if let Some(transitions) = self.transitions.get(&state) {
                    for (transition, next_state) in transitions {
                        match transition {
                            Transition::Char(c1) => {
                                if c == *c1 {
                                    next_states.insert(next_state.clone());
                                }
                            }
                            _ => (),
                        }
                    }
                }
            }
            current_states = next_states;
        }
        current_states.iter().any(|state| self.accept_states.contains(state))
    }

    
    pub fn check_str_princeton(&self, input_str: &str) -> Vec<String> {
        let mut cur_states: HashSet<State> = HashSet::new();
        let mut start_positions: HashMap<State, Vec<usize>> = HashMap::new();
        cur_states.insert(self.start_state.clone());
        start_positions.insert(self.start_state.clone(), vec![0]);

        // strings to return
        let mut matched_strs: Vec<String> = Vec::new();

        for (i, c) in input_str.char_indices() {
            let mut next_states: HashSet<State> = HashSet::new();
            next_states.insert(self.start_state.clone());
            let mut next_positions: HashMap<State, Vec<usize>> = HashMap::new();
            next_positions.insert(self.start_state.clone(), vec![i+1]);

            // for all possible current states
            for state in &cur_states {
                if let Some(transitions) = self.transitions.get(state) {
                    for (transition, next_state) in transitions {
                        match transition {
                            // if the character can lead to a next state by a valid transition
                            Transition::Char(c1) if *c1 == c => {
                                next_states.insert(next_state.clone());
                                // get the starting positions of the current state
                                // if the next state is not in the hashmap, add the starting position of the current state
                                if !next_positions.contains_key(next_state) {
                                    if let Some(start_position) = start_positions.get(state) {
                                        next_positions.insert(next_state.clone(), start_position.clone());
                                    } else {
                                        next_positions.insert(next_state.clone(), vec![i+1]);
                                    }
                                }
                                else {
                                    // if the next state is in the hashmap, add the starting positions of the current state
                                    // to the vector of starting positions of the next state
                                    if let Some(start_position) = start_positions.get(state) {
                                        if let Some(next_start_positions) = next_positions.get_mut(next_state) {
                                            for start_pos in start_position {
                                                next_start_positions.push(*start_pos);
                                            }
                                        }
                                    }
                                }
                            }
                            _ => (),
                        }
                    }
                }
            }
            cur_states = next_states;
            start_positions = next_positions;
            
            // check any matched
            for state in &cur_states {
                if self.accept_states.contains(&state) {
                    if let Some(start_positions) = start_positions.get(&state) {
                        for start_pos in start_positions {
                            matched_strs.push(input_str[*start_pos..(i+1)].to_string());
                        }
                    }
                }
            }
        }
        matched_strs
    }

  
    pub fn check_str(&self, input_str: &str) -> (bool, String) {
        let mut current_states: HashSet<State> = HashSet::new();
        current_states.insert(self.start_state.clone());
        let mut matched_str = String::new();

        for c in input_str.chars() {
            let mut next_states: HashSet<State> = HashSet::new();
            let mut found_accept = false;

            for state in &current_states {
                if let Some(transitions) = self.transitions.get(state) {
                    for (transition, next_state) in transitions {
                        match transition {
                            Transition::Char(c1) if *c1 == c => {
                                next_states.insert(next_state.clone());
                                if self.accept_states.contains(next_state) {
                                    found_accept = true;
                                    matched_str.push(c);
                                    return (true, matched_str);
                                }
                            }
                            _ => (),
                        }
                    }
                }
            }

            if !next_states.is_empty() {
                matched_str.push(c);
            }

            current_states = next_states;

            // If no next states are found, stop the loop
            if current_states.is_empty() {
                break;
            }
        }

        (false, matched_str) // Return false with the substring processed so far (may not be fully matched)
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
        };
        new_nfa
    }
}

pub fn nfa_from_reg(regex: &str) -> NFA {
    let cfg = cfg_for_regular_expression();
    let ast = cfg.parse(regex).unwrap().collapse();
    NFA::from_regex(&ast)
}

#[cfg(test)]
mod test {
    use crate::nfa;

    use super::*;
    #[test]
    fn test_shift_nfa() {
        println!("Test shift nfa");
        let mut nfa = NFA::new();
        nfa.add_state();
        nfa.add_state();
        nfa.add_transition(State {id: 0}, Transition::Char('a'), State {id: 1});
        let nfa = nfa.modify_state_id(5);
        // nfa.debug_helper();
        println!("\n");
    }

    #[test]
    fn test_from_char() {
        let nfa = NFA::from_char('a');
        println!("Test from char");
        // nfa.debug_helper();
        println!("\n");
    }


    #[test]
    fn test_single_char_nfa() {
        let nfa = NFA::from_char('a');
        println!("Test single char NFA");
        // nfa.debug_helper();
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
        let nfa = NFA::epsilon_close(nfa);
        nfa.debug_helper();
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

    #[test]
    fn test_check_str_basic() {
        println!("Test check string");
        let nfa = NFA::from_char('a');
        assert!(nfa.check_str("a").0);
        assert!(!nfa.check_str("b").0);
        assert!(nfa.check_str("ab").0);
    }

    // #[test]
    // fn test_check_return_str() {
    //     println!("Test check string return string");
    //     let nfa = nfa_from_reg("ab|c");
    //     let nfa = NFA::epsilon_close(nfa);
    //     let result = nfa.check_str_princeton("ab");
    //     println!("{:?}\n", result);
    //     let result = nfa.check_str_princeton("dab");
    //     println!("{:?}", result);

        
    //     // assert!(!nfa.check_str2("b"));
    //     // assert!(nfa.check_str2("c"));
    // }
    
    // #[test]
    // fn test_check_bool() {
    //     println!("Test check string return bool");
    //     let nfa = nfa_from_reg("ab|c");
    //     let nfa = NFA::epsilon_close(nfa);
    //     assert!(nfa.check_str_princeton("ab"));
    //     assert!(nfa.check_str_princeton("dab"));
    //     assert!(nfa.check_str_princeton("gjgjhfc"));
    //     assert!(nfa.check_str_princeton("gjgcjhf"));
    // }

    #[test]
    fn test_check_string_1() {
        println!("Test check string return string vec 1");
        let nfa = nfa_from_reg("ab|c");
        let nfa = NFA::epsilon_close(nfa);
        print!("{:?}", nfa.check_str_princeton("abab"));
        print!("{:?}", nfa.check_str_princeton("cab"));
        print!("{:?}", nfa.check_str_princeton("c"));
        print!("{:?}", nfa.check_str_princeton("cabcabcab"));
    }

    #[test]
    fn test_check_string_2() {
        println!("Test check string return string vec 2");
        let nfa = nfa_from_reg("ab");
        let nfa = NFA::epsilon_close(nfa);
        nfa.debug_helper();
        println!("");
        print!("{:?}", nfa.check_str_princeton("abab"));
    }


    #[test]
    fn test_epsilon_closure() {
        println!("Test epsilon closure");
        let mut nfa = NFA::new_with_n_states(4);
        let state0 = State { id: 0 };
        let state1 = State { id: 1 };
        let state2 = State { id: 2 };
        let state3 = State { id: 3 };
        let state4 = State { id: 4 };

        nfa.add_transition(state0.clone(), Transition::Char('0'), state0.clone());
        nfa.add_transition(state0.clone(), Transition::Epsilon, state1.clone());
        // 1, 1, 1
        nfa.add_transition(state1.clone(), Transition::Char('1'), state1.clone());
        // 1, epsilon, 2
        nfa.add_transition(state1.clone(), Transition::Epsilon, state2.clone());
        // 2, 0, 2
        nfa.add_transition(state2.clone(), Transition::Char('0'), state2.clone());
        // 2, epsilon, 3
        nfa.add_transition(state2.clone(), Transition::Epsilon, state3.clone());
        // 2, 0, 4
        nfa.add_transition(state2.clone(), Transition::Char('0'), state4.clone());
        nfa.accept_states.insert(state3.clone());
        nfa.accept_states.insert(state4.clone());
        let nfa = NFA::epsilon_close(nfa);
        nfa.debug_helper();
    }

    // #[test]
    // fn test_nfa_from_regex() {
    //     let regex = "a | b";
    //     let cfg = cfg_for_regular_expression();
    //     let ast = cfg.parse(regex).unwrap().collapse();

    //     println!("{:#?}", PrettyPrint(&cfg.parse(regex).unwrap()));

    //     println!("{:#?}", PrettyPrint(&ast));

    //     let nfa = NFA::from_regex(&ast);
    //     nfa.debug_helper();

    //     println!("After epsilon closure:");

    //     let nfa = NFA::epsilon_close(nfa);
    //     nfa.debug_helper();

    // }

    // #[test]
    // fn test_from_union_empty() {
    //     let nfas = Vec::new();
    //     let nfa = NFA::from_union(nfas);
    //     nfa.debug_helper();

    //     assert_eq!(nfa.states.len(), 1); // Only the accept state should be present
    //     assert_eq!(nfa.accept_states.len(), 1);
    //     assert!(nfa.transitions.is_empty());
    // }

    // #[test]
    // fn test_from_union_single_nfa() {
    //     let mut single_nfa = NFA::new();
    //     let start_state = single_nfa.add_state();
    //     single_nfa.start_state = start_state.clone();
    //     single_nfa.accept_states.insert(start_state.clone());

    //     let nfas = vec![single_nfa.clone()];
    //     let nfa = NFA::from_union(nfas);

    //     assert_eq!(nfa.states.len(), 2); // Start state + accept state
    //     assert_eq!(nfa.accept_states.len(), 1);
    //     assert!(nfa.transitions.contains_key(&start_state));
    //     let transitions = nfa.transitions.get(&start_state).unwrap();
    //     assert_eq!(transitions.len(), 1);
    //     assert_eq!(transitions[0].0, Transition::Epsilon);
    // }

    // #[test]
    // fn test_from_union_multiple_nfas() {
    //     let mut nfa1 = NFA::new();
    //     let start1 = nfa1.add_state();
    //     nfa1.start_state = start1.clone();
    //     nfa1.accept_states.insert(start1.clone());

    //     let mut nfa2 = NFA::new();
    //     let start2 = nfa2.add_state();
    //     nfa2.start_state = start2.clone();
    //     nfa2.accept_states.insert(start2.clone());

    //     let nfas = vec![nfa1, nfa2];
    //     let nfa = NFA::from_union(nfas);

    //     assert_eq!(nfa.states.len(), 3); // Two start states + one accept state
    //     assert_eq!(nfa.accept_states.len(), 1);
    //     assert!(nfa.transitions.contains_key(&start1));
    //     assert!(nfa.transitions.contains_key(&start2));
    //     assert_eq!(nfa.transitions.get(&start1).unwrap().len(), 1);
    //     assert_eq!(nfa.transitions.get(&start2).unwrap().len(), 1);
    // }
}