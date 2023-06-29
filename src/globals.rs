use std::{collections::HashMap, sync::Mutex};

use once_cell::sync::Lazy;

use crate::encoder::{Instr, Rela, UserDefinedSection};

pub static USER_DEFINED_SYMBOLS: Lazy<Mutex<HashMap<String, Instr>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));
pub static RELA_TEXT_USERS: Lazy<Mutex<Vec<Rela>>> = Lazy::new(|| Mutex::new(Vec::new()));
pub static USER_DEFINED_SECTIONS: Lazy<Mutex<HashMap<String, UserDefinedSection>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));
