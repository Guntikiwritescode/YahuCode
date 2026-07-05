//! The ACTUAL environment: globals plus a stack of function-call frames.
//!
//! A function body sees its own frame plus the globals — not the caller's locals — so
//! recursion is well-behaved and there is no dynamic-scope leakage between calls.

use std::collections::HashMap;

use crate::model::Val;

/// A scoped value environment.
#[derive(Clone, Debug, Default)]
pub struct Env {
    globals: HashMap<String, Val>,
    frames: Vec<HashMap<String, Val>>,
}

impl Env {
    pub fn new() -> Self {
        Env::default()
    }

    /// Read a variable: the current call frame (if any) first, then globals.
    pub fn get(&self, name: &str) -> Option<&Val> {
        if let Some(frame) = self.frames.last() {
            if let Some(v) = frame.get(name) {
                return Some(v);
            }
        }
        self.globals.get(name)
    }

    /// Assign a variable. Updates it where it already lives (current frame, else a
    /// global); otherwise creates it in the current scope (the call frame if inside a
    /// call, else globals).
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

    /// Read a variable mutably: the current call frame (if any) first, then globals.
    /// Used by the collection ops to mutate a container in place (a `push`/`allocate`/
    /// `classify` writes the single backing store — no clone-and-reinsert churn).
    pub fn get_mut(&mut self, name: &str) -> Option<&mut Val> {
        if let Some(frame) = self.frames.last_mut() {
            if frame.contains_key(name) {
                return frame.get_mut(name);
            }
        }
        self.globals.get_mut(name)
    }

    /// Enter a new function-call frame.
    pub fn push_frame(&mut self) {
        self.frames.push(HashMap::new());
    }

    /// Leave the current function-call frame.
    pub fn pop_frame(&mut self) {
        self.frames.pop();
    }

    /// Bind a parameter/local directly in the current frame (shadowing any global).
    pub fn define_local(&mut self, name: &str, val: Val) {
        if let Some(frame) = self.frames.last_mut() {
            frame.insert(name.to_string(), val);
        } else {
            self.globals.insert(name.to_string(), val);
        }
    }
}
