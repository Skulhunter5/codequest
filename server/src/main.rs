use std::{
    fs::{self, DirBuilder},
    path::Path,
};

use base64::Engine as _;
use rand::RngCore;
use rocket::{fs::NamedFile, routes};
use rocket_dyn_templates::Template;
use serde::Serialize;

pub const RUN_DIR: &'static str = "./run";
pub const SALT_FILE: &'static str = "./run/salt";

fn load_or_generate_salt() -> String {
    if let Ok(salt) = fs::read_to_string(&SALT_FILE) {
        return salt;
    }

    let mut rng = rand::rng();
    let mut salt = [0u8; 16];
    rng.fill_bytes(&mut salt);
    let salt = base64::engine::general_purpose::STANDARD.encode(&salt);
    fs::write(&SALT_FILE, &salt).expect("failed to write salt to file");
    return salt;
}

#[rocket::get("/")]
async fn index() -> Template {
    #[derive(Serialize)]
    struct IndexPageContext<'a> {
        username: &'a str,
        links: Vec<LinkContext<'a>>,
    }

    #[derive(Serialize)]
    struct LinkContext<'a> {
        name: &'a str,
        url: &'a str,
    }

    Template::render(
        "index",
        IndexPageContext {
            username: "Someone",
            links: vec![
                LinkContext {
                    name: "GitHub",
                    url: "https://www.github.com",
                },
                LinkContext {
                    name: "Google",
                    url: "https://www.google.com",
                },
            ],
        },
    )
}

#[rocket::get("/about")]
async fn about() -> Option<NamedFile> {
    let path = Path::new("static").join("about.html");
    NamedFile::open(path).await.ok()
}

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    DirBuilder::new()
        .recursive(true)
        .create(&RUN_DIR)
        .expect("failed to create run dir");
    let _salt = load_or_generate_salt();

    // let rocket_config = rocket::Config::figment().merge(("template_dir", "static/"));
    // rocket::custom(&rocket_config)

    rocket::build()
        .mount("/", routes![index, about])
        .attach(Template::fairing())
        .launch()
        .await?;

    Ok(())
}
