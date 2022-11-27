use rand::distributions::Alphanumeric;
pub use rand::prelude::SliceRandom;
pub use rand::{thread_rng, Rng};
use random_string::generate;

pub fn random_alphanumeric_string(n: usize) -> String {
    let mut rng = thread_rng();
    let chars: String = (0..n).map(|_| rng.sample(Alphanumeric) as char).collect();
    chars
}

pub fn rand_string(n: usize) -> String {
    generate(
        n,
        "1234567890abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTYUVWXYZ",
    )
}

pub fn rand_xattr(n: usize) -> String {
    generate(
        n,
        "1234567890abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTYUVWXYZ.",
    )
}
// +=-_\\|]}{[;:\"\'><,.?!@#$%^&*()~`
pub fn random_len_string() -> String {
    let mut rng = thread_rng();
    rand_string(rng.gen_range(1..10))
}

pub fn rand_size() -> i64 {
    thread_rng().gen_range(0..(1024 * 2 + 2)) as i64 - 1
}
