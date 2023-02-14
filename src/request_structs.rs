use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct LabelUpdate {
    pub label_id: String,
}
