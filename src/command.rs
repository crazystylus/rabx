use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize, Debug)]
pub enum Command {
    Set { key: String, value: String },
    Rm { key: String },
}
