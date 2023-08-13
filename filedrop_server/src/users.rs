use std::str::FromStr;

use axum::{extract::Path, Json};
use filedrop_lib::Group;
use uuid::Uuid;

use crate::db;

pub async fn get_user(Path(user_id): Path<String>) -> Json<Vec<Group>> {
    println!("{user_id}");
    let groups = sqlx::query!(
        "
        select name, id from group_members 
            join groups on group_id = groups.id 
            where user_id = $1",
        &user_id
    )
    .fetch_all(db::get())
    .await
    .unwrap();

    let groups = groups
        .into_iter()
        .filter_map(|g| {
            Some(Group {
                name: g.name?,
                id: Uuid::from_str(&g.id).ok()?,
            })
        })
        .collect();
    Json(groups)
}
