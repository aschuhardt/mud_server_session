//configuration.rs

use rustc_serialize::json;
use file_io::FileIO;

static DEFAULT_CONFIG_PATH: &'static str = "config/config.json";

#[derive(RustcDecodable, RustcEncodable, Clone)]
pub struct Configuration {
    //add more fields to this as needed
    pub data_location: String,
    pub network_port: u16,
    pub debug_mode: bool,
    pub max_request_cache_count: usize,
    pub request_validation_token: String,
}

impl Configuration {
    pub fn load() -> Configuration {
        Configuration::load_path(DEFAULT_CONFIG_PATH)
    }

    pub fn load_path(path: &'static str) -> Configuration {
        let conf_json = FileIO::read_string(path);
        match json::decode(&conf_json) {
            Ok(c) => c,
            Err(why) => panic!("Unable to read supplied config file at {}: {}", path, why),
        }
    }

    pub fn save_config(conf: &Configuration) {
        Configuration::save_config_path(conf, DEFAULT_CONFIG_PATH);
    }

    pub fn save_config_path(conf: &Configuration, path: &str) {
        let conf_json = json::encode(conf).unwrap();
        FileIO::write_string(path, conf_json.as_str());
    }
}
