use argon2::password_hash::{SaltString, rand_core::OsRng};
use std::{fs, io, path::Path};

pub mod services;

pub struct Quest<'a> {
    pub name: &'a str,
    pub id: &'a str,
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
