use crate::utils::config;
use diesel::prelude::*;
use dotenv::dotenv;

pub fn establish_connection() -> PgConnection {
    dotenv().ok();

    let database_url = config().DATABASE_URL;
    PgConnection::establish(&database_url).expect(&format!("Failed to connect to {}", database_url))
}
