use std::{fs, path::PathBuf};

use uuid::Uuid;

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub struct LocalData {
    pub user_id: Uuid,
    pub username: String,
}

impl Default for LocalData {
    fn default() -> Self {
        Self {
            user_id: Uuid::new_v4(),
            username: "New user!".into(),
        }
    }
}

pub fn get_localdata() -> LocalData {
    let mut path = dirs::home_dir().expect("Unsupported os");
    path.push(".filedropid");

    if let Some(data) = load_from_disk(&path) {
        data
    } else {
        write_to_disk(&path)
    }
}

fn load_from_disk(path: &PathBuf) -> Option<LocalData> {
    let raw_data = fs::read_to_string(path).ok()?;
    serde_json::from_str(&raw_data).ok()
}

fn write_to_disk(path: &PathBuf) -> LocalData {
    let data = LocalData::default();
    fs::write(path, serde_json::to_string(&data).unwrap()).unwrap();
    data
}
