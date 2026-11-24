//! Globals plus a stack of call frames; a function sees its own frame and the globals.

use std::collections::HashMap;

use crate::model::Val;

#[derive(Clone, Debug, Default)]
pub struct Env {
    globals: HashMap<String, Val>,
    frames: Vec<HashMap<String, Val>>,
}

impl Env {
    pub fn new() -> Self {
        Env::default()
    }

    pub fn get(&self, name: &str) -> Option<&Val> {
        if let Some(frame) = self.frames.last() {
            if let Some(v) = frame.get(name) {
                return Some(v);
            }
        }
        self.globals.get(name)
    }

    pub fn set(&mut self, name: &str, val: Val) {
        if let Some(frame) = self.frames.last_mut() {
            if frame.contains_key(name) {
                frame.insert(name.to_string(), val);
                return;
            }
            if self.globals.contains_key(name) {
                self.globals.insert(name.to_string(), val);
                return;
            }
            frame.insert(name.to_string(), val);
            return;
        }
        self.globals.insert(name.to_string(), val);
    }

    /// Mutable lookup, so collection ops update the container in place.
    pub fn get_mut(&mut self, name: &str) -> Option<&mut Val> {
        if let Some(frame) = self.frames.last_mut() {
            if frame.contains_key(name) {
                return frame.get_mut(name);
            }
        }
        self.globals.get_mut(name)
    }

    pub fn push_frame(&mut self) {
        self.frames.push(HashMap::new());
    }

    pub fn pop_frame(&mut self) {
        self.frames.pop();
    }

    pub fn define_local(&mut self, name: &str, val: Val) {
        if let Some(frame) = self.frames.last_mut() {
            frame.insert(name.to_string(), val);
        } else {
            self.globals.insert(name.to_string(), val);
        }
    }
}
