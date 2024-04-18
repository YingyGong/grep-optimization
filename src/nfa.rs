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
        nfa.accept_states.insert(start_state.clone());
        nfa
    }

    fn add_state(&mut self) -> State {
        let state = State { id: self.next_state_id };
        self.next_state_id += 1;
        self.states.insert(state.clone());
        state
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

    fn from_concatenation(nfas: Vec<NFA>) -> Self {
        let mut nfa = NFA::new();
        let mut accept_states = Vec::new();
        for mut n in nfas {
            let accept_state = n.add_state();
            n.accept_states.insert(accept_state.clone());
            let accept_states_copy: Vec<_> = n.accept_states.iter().cloned().collect();
            for state in accept_states_copy {
                n.add_transition(state.clone(), Transition::Epsilon, accept_state.clone());
            }
            accept_states.push(accept_state);
            nfa.states.extend(n.states);
            nfa.transitions.extend(n.transitions);
        }
        nfa.accept_states = accept_states.into_iter().collect();
        nfa
    }

    fn from_union(nfas: Vec<NFA>) -> Self {
        let mut nfa = NFA::new();
        for n in nfas {
            // n.add_transition(n.start_state.clone(), Transition::Epsilon, accept_state.clone());
            nfa.states.extend(n.states);
            nfa.transitions.extend(n.transitions);
            nfa.accept_states.extend(n.accept_states);
            nfa.transitions.insert(n.start_state.clone(), vec![(Transition::Epsilon, n.start_state.clone())]);
        }
        nfa
    }

    fn from_kleene_star(mut nfa: NFA) -> Self {
        let accept_state = nfa.add_state();
        nfa.accept_states.insert(accept_state.clone());
        nfa.add_transition(accept_state.clone(), Transition::Epsilon, nfa.start_state.clone());

        let accept_states_copy: Vec<_> = nfa.accept_states.iter().cloned().collect();
        for state in accept_states_copy {
            nfa.add_transition(state.clone(), Transition::Epsilon, accept_state.clone());
        }
        nfa
    }

    fn from_plus(mut nfa: NFA) -> Self {
        let accept_state = nfa.add_state();
        nfa.accept_states.insert(accept_state.clone());

        let accept_states_copy: Vec<_> = nfa.accept_states.iter().cloned().collect();
        for state in accept_states_copy{
            nfa.add_transition(state.clone(), Transition::Epsilon, accept_state.clone());
        }
        nfa
    }

    fn from_question_mark(mut nfa: NFA) -> Self {
        let accept_state = nfa.add_state();
        nfa.accept_states.insert(accept_state.clone());
        nfa.add_transition(accept_state.clone(), Transition::Epsilon, nfa.start_state.clone());
        nfa
    }

   fn epsilon_close(mut nfa: NFA) -> Self {
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

    fn from_regex(node: &ASTNode) -> Self{
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
                    NFA::from_regex(&children[0])
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

    // fn check_str(&self, input_str: &str) -> bool {

    // }
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

mod test {
    use super::*;
    #[test]
    fn test_single_char_nfa() {
        let nfa = NFA::from_char('a');
        println!("Test single char NFA");
        nfa.debug_helper();
        println!("\n");
    }

    #[test]
    fn test_union() {
        let nfa1 = NFA::from_char('a');
        let nfa2 = NFA::from_char('b');
        let nfa = NFA::from_union(vec![nfa1, nfa2]);
        println!("Test union");
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