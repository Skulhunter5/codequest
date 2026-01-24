use argon2::password_hash::{SaltString, rand_core::OsRng};
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
pub mod event;
pub mod nats;
mod quest;
pub mod services;
pub mod statistics;
mod user;

pub use credentials::Credentials;
pub use error::Error;
pub use quest::{PartialQuestData, Quest, QuestData, QuestDataFields, QuestEntry, QuestId};
pub use user::{User, UserId, Username, UsernameRef};

pub fn load_salt(path: impl AsRef<Path>) -> io::Result<SaltString> {
    Ok(SaltString::from_b64(fs::read_to_string(path)?.trim()).expect("failed to load salt"))
}

pub fn load_or_generate_salt(path: impl AsRef<Path>) -> io::Result<SaltString> {
    match fs::read_to_string(&path) {
        Ok(salt) => return Ok(SaltString::from_b64(&salt.trim()).expect("failed to load salt")),
        Err(e) if e.kind() == io::ErrorKind::NotFound => (),
        Err(e) => return Err(e),
    }

    let generated_salt = SaltString::generate(&mut OsRng);
    fs::write(path, &generated_salt.as_str())?;
    return Ok(generated_salt);
}

pub fn load_secret_key(path: impl AsRef<Path>) -> io::Result<String> {
    fs::read_to_string(path).map(|s| s.trim().to_owned())
}

pub fn load_or_generate_secret_key(path: impl AsRef<Path>) -> io::Result<String> {
    match fs::read_to_string(&path).map(|s| s.trim().to_owned()) {
        Ok(secret_key) => return Ok(secret_key),
        Err(e) if e.kind() == io::ErrorKind::NotFound => (),
        Err(e) => return Err(e),
    }

    let mut bytes = vec![0; 32];
    rand::rng().fill_bytes(&mut bytes);
    let generated_secret_key = BASE64_STANDARD.encode(&bytes);

    let mut file = File::create(&path)?;
    file.set_permissions(Permissions::from_mode(0o600))?;
    file.write_all(&generated_secret_key.as_bytes())?;

    Ok(generated_secret_key)
}
