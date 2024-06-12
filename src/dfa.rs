use std::collections::{HashMap, HashSet};
use crate::nfa_optimized::{NFA, State};


#[derive(Hash, Eq, PartialEq, Clone, Debug)]
struct DState {
    nfa_states: HashSet<State>, // each DFA state is a set of NFA states
    transitions: HashMap<char, Box<DState>>, 
}

struct DFACache {
    cache: HashMap<HashSet<State>, Box<DState>>,
}

impl DFACache {
    fn new() -> Self {
        DFACache {
            cache: HashMap::new(),
        }
    }

    fn get_or_create_dstate(&mut self, nfa_states: HashSet<State>) -> Box<DState> {
        if let Some(dstate) = self.cache.get(&nfa_states) {
            return dstate.clone();
        }

        let new_dstate = Box::new(DState {
            nfa_states: nfa_states.clone(),
            transitions: HashMap::new(),
        });
        self.cache.insert(nfa_states, new_dstate.clone());
        new_dstate
    }
}

