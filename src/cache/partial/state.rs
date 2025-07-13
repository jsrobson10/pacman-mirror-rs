use std::sync::{Condvar, Mutex, MutexGuard};


pub struct State {
    pub len: usize,
    pub done: bool,
}

pub struct StateHolder {
    pub mtx: Mutex<State>,
    pub cvar: Condvar,
}

impl State {
    fn empty() -> State {
        State { len: 0, done: false }
    }
}

impl StateHolder {
    pub fn new() -> StateHolder {
        StateHolder { mtx: State::empty().into(), cvar: Condvar::new() }
    }
    pub fn wait_and_lock(&self, len: usize) -> MutexGuard<State> {
        let mut state = self.mtx.lock().unwrap();
        while state.len < len && !state.done {
            state = self.cvar.wait(state).unwrap();
        }
        state
    }
}

