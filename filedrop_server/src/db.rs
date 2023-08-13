use std::env;

use sqlx::{Pool, Postgres};
use tokio::sync::OnceCell;

static DATABASE: OnceCell<Pool<Postgres>> = OnceCell::const_new();

pub(super) async fn init() {
    let url = env::var("DATABASE_URL").unwrap();

    let pool = Pool::connect(&url).await.unwrap();
    DATABASE.set(pool).unwrap();
}

pub fn get() -> &'static Pool<Postgres> {
    DATABASE.get().unwrap()
}
