use std::collections::HashMap;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct HashStore {
    pub(crate) hashes: HashMap<String, String>,
}

#[derive(Clone)]
pub struct Node {
    pub(crate) target: String,
    pub(crate) deps: Vec<String>,
    pub(crate) command: Vec<String>,
}

pub(crate) type Graph = HashMap<String, Node>;

