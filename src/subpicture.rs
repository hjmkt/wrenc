use super::slice::*;
use std::sync::{Arc, Mutex};

pub struct Subpicture {
    _slices: Vec<Arc<Mutex<Slice>>>,
}

impl Subpicture {
    pub fn new(_slices: Vec<Arc<Mutex<Slice>>>) -> Subpicture {
        Subpicture { _slices }
    }
}
