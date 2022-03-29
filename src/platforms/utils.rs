use rand::distributions::Alphanumeric;
use rand::Rng;

pub fn generate_state_string() -> String {
    let rand_string: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(32)
        .map(char::from)
        .collect();

    rand_string
}
