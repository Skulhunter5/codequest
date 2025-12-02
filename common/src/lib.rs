use argon2::password_hash::{SaltString, rand_core::OsRng};
use serde::{Deserialize, Serialize};
use std::{fs, io, path::Path};

pub mod services;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Quest {
    pub name: String,
    pub id: String,
}

impl Quest {
    pub fn new<S: AsRef<str>>(name: S, id: S) -> Self {
        Self {
            name: name.as_ref().to_owned(),
            id: id.as_ref().to_owned(),
        }
    }
}

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
