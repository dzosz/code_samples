use crate::error_handler::CustomError;

use diesel::r2d2::ConnectionManager;
use diesel::sqlite;
use diesel::sqlite::SqliteConnection;
use lazy_static::lazy_static;
use r2d2;
use std::env;

type Pool = r2d2::Pool<ConnectionManager<SqliteConnection>>;
pub type DbConnection = r2d2::PooledConnection<ConnectionManager<SqliteConnection>>;

embed_migrations!();

// SqliteCOnnection::establish("test.db");
lazy_static! {
    static ref POOL: Pool = {
        let db_url = env::var("DATABASE_URL").expect("database url is not set");
        let manager = ConnectionManager::<SqliteConnection>::new(db_url);
        Pool::new(manager).expect("failed to create db pool")
    };
}

pub fn init() {
    lazy_static::initialize(&POOL);
    let conn = connection().expect("failed to get db connection");
    embedded_migrations::run(&conn).unwrap();
}

pub fn connection() -> Result<DbConnection, CustomError> {
    POOL.get()
        .map_err(|e| CustomError::new(500, format!("Failed getting db connection: {}", e)))
}
