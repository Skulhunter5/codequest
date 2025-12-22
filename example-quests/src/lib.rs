use std::str::FromStr;

use rand::{RngCore, SeedableRng};
use rand_chacha::ChaCha12Rng;
use sha2::{Digest, Sha256};
use uuid::Uuid;

pub struct QuestContext {
    input: String,
    answer: String,
}

impl QuestContext {
    pub fn new(input: String, answer: String) -> Self {
        Self { input, answer }
    }
}

pub fn quest_context_generator(f: fn(&mut dyn RngCore) -> QuestContext) {
    let mut args = std::env::args();
    let _executable_path = args.next().unwrap();

    let user_id = args.next().expect("error: missing user-id");
    let user_id = Uuid::from_str(&user_id).expect("error: invalid user-id");

    let rng_seed = &Sha256::digest(user_id.as_bytes())[..];
    let mut rng = ChaCha12Rng::from_seed(rng_seed.try_into().unwrap());

    let QuestContext { input, answer } = f(&mut rng);

    print!("{input}\0{answer}");
}
