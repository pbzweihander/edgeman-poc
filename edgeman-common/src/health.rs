use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Health {
    pub id: String,
    /// name -> timestamp map
    pub pods: HashMap<String, DateTime<Utc>>,
}

impl Health {
    pub fn new(id: String) -> Self {
        Self {
            id,
            pods: HashMap::new(),
        }
    }
}
