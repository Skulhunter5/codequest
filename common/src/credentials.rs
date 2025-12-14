pub struct Credentials {
    pub username: String,
    pub password: String,
}

impl Credentials {
    pub fn new<S: AsRef<str>>(username: S, password: S) -> Self {
        Self {
            username: username.as_ref().to_owned(),
            password: password.as_ref().to_owned(),
        }
    }
}
