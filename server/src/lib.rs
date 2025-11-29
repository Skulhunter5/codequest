pub mod services;

pub struct Quest<'a> {
    pub name: &'a str,
    pub id: &'a str,
}
