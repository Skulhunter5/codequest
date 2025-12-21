use argon2::password_hash::{SaltString, rand_core::OsRng};
use std::{fs, io, path::Path};

mod credentials;
mod error;
mod quest;
pub mod services;
mod user;

pub use credentials::Credentials;
pub use error::Error;
pub use quest::{Quest, QuestId, QuestItem};
pub use user::{User, UserId, Username};

pub fn load_or_generate_salt<P: AsRef<Path>>(path: P) -> SaltString {
    if let Ok(salt) = fs::read_to_string(&path) {
        return SaltString::from_b64(&salt).expect("failed to create salt");
    }

    let salt = SaltString::generate(&mut OsRng);
    fs::write(path, salt.as_str()).expect("failed to write salt to file");
    return salt;
}

pub fn load_secret_key<P: AsRef<Path>>(path: P) -> io::Result<String> {
    fs::read_to_string(path)
}
