use std::fs::File;
use std::path::PathBuf;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Credentials {
    pub api_key: String,
    pub api_secret: String,
    pub access_token: String,
    pub access_secret: String,
}

#[derive(Deserialize)]
pub struct Configuration {
    pub track_keys: Vec<String>,
    pub http_port: Option<String>
}

pub fn setup() -> (Credentials, Configuration) {
    let mut setup_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    println!("{:?}",  setup_path);
    setup_path.push("config.json");

    let configuration_file = File::open(setup_path.clone()).expect("Configuration File");
    let configuration =
        Configuration::deserialize(&mut json::Deserializer::from_reader(configuration_file))
            .expect("Valid Configuration");
    setup_path.pop();
    setup_path.push("credentials.json");
    let credential_file = File::open(setup_path).expect("Crendential File");
    let credentials =
        Credentials::deserialize(&mut json::Deserializer::from_reader(credential_file))
            .expect("Valid Credentials");
    (credentials, configuration)
}
