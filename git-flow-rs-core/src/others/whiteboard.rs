use std::{
    collections::HashMap,
    sync::{Mutex, OnceLock},
};

pub static WHITEBOARD: OnceLock<Mutex<HashMap<String, String>>> = OnceLock::new();
