use rand::distributions::Alphanumeric;
use rand::Rng;

use worker::{FormData, FormEntry};

pub fn generate_state_string() -> String {
    let rand_string: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(32)
        .map(char::from)
        .collect();

    rand_string
}

pub fn get_param_val(form_data: &FormData, name: &str) -> Option<String> {
    // This is fucking atrocious.
    if let Some(value) = form_data.get(name) {
        if let FormEntry::Field(val) = value {
            return Some(val);
        }
    };

    None
}
