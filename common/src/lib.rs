use argon2::password_hash::{Salt, SaltString};
use base64::{Engine as _, prelude::BASE64_STANDARD};
use rand::RngCore;
use std::{
    fs::{self, File, Permissions},
    io::{self, Write},
    os::unix::fs::PermissionsExt as _,
    path::Path,
};

mod credentials;
mod error;
pub mod nats;
mod quest;
pub mod services;
mod user;

pub use credentials::Credentials;
pub use error::Error;
pub use quest::{Quest, QuestId, QuestItem};
pub use user::{User, UserId, Username};

pub fn load_salt(path: impl AsRef<Path>) -> io::Result<SaltString> {
    Ok(SaltString::from_b64(fs::read_to_string(path)?.trim()).expect("failed to load salt"))
}

pub fn load_or_generate_salt(path: impl AsRef<Path>) -> io::Result<SaltString> {
    match fs::read_to_string(&path) {
        Ok(salt) => return Ok(SaltString::from_b64(&salt.trim()).expect("failed to load salt")),
        Err(e) if e.kind() == io::ErrorKind::NotFound => (),
        Err(e) => return Err(e),
    }

    let generated_salt =
        generate_base64_encoded_cryptographically_safe_random_material(Salt::RECOMMENDED_LENGTH);
    fs::write(path, &generated_salt)?;
    let generated_salt = SaltString::from_b64(&generated_salt).unwrap();
    return Ok(generated_salt);
}

pub fn load_secret_key(path: impl AsRef<Path>) -> io::Result<String> {
    fs::read_to_string(path).map(|s| s.trim().to_owned())
}

fn generate_base64_encoded_cryptographically_safe_random_material(bytes: usize) -> String {
    let mut bytes = vec![0; bytes];
    rand::rng().fill_bytes(&mut bytes);
    BASE64_STANDARD.encode(&bytes)
}

pub fn load_or_generate_secret_key(path: impl AsRef<Path>) -> io::Result<String> {
    match fs::read_to_string(&path).map(|s| s.trim().to_owned()) {
        Ok(secret_key) => return Ok(secret_key),
        Err(e) if e.kind() == io::ErrorKind::NotFound => (),
        Err(e) => return Err(e),
    }

    let generated_secret_key = generate_base64_encoded_cryptographically_safe_random_material(32);
    let mut file = File::create(&path)?;
    file.set_permissions(Permissions::from_mode(0o600))?;
    file.write_all(&generated_secret_key.as_bytes())?;

    Ok(generated_secret_key)
}
