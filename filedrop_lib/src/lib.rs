use uuid::Uuid;

#[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
pub struct EventData {
    pub filename: String,
    pub file_id: Uuid,
    pub groupname: String,
    pub group_id: Uuid,
    pub sender: String,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Group {
    pub name: String,
    pub id: Uuid,
}
