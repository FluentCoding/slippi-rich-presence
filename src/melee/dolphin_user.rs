use std::fs;

use directories::BaseDirs;
use serde_json::Value;

pub fn get_connect_code() -> Option<String> {
    if let Some(base_dirs) = BaseDirs::new() {
        let user_json_path = base_dirs.config_dir().join("Slippi Launcher/netplay/User/Slippi/user.json");
        if user_json_path.is_file() && user_json_path.exists() {
            return match fs::read_to_string(user_json_path) {
                Ok(data) => {
                    let v = serde_json::from_str::<Value>(data.as_str());
                    match v {
                        Ok(data) => data["connectCode"].as_str().and_then(|v| Some(v.to_string())),
                        _ => None
                    }
                },
                _ => None
            }
        }
    }

    None
}