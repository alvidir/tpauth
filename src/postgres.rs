use std::env;
use lazy_static;
use diesel::{
    r2d2::{Pool, ConnectionManager},
    pg::PgConnection
};

const ERR_NOT_URL: &str = "Postgres url must be set";
const ERR_CONNECT: &str = "Error connecting to";
const ENV_DATABASE_URL: &str = "DATABASE_URL";
const POOL_SIZE: u32 = 8_u32;

type PgPool = Pool<ConnectionManager<PgConnection>>;

pub struct Values {
   pub db_connection: PgPool,
}

lazy_static! {
    static ref STREAM: Values = {
       Values {
           db_connection: PgPool::builder()
               .max_size(POOL_SIZE)
               .build(ConnectionManager::new(env::var(ENV_DATABASE_URL).expect(ERR_NOT_URL)))
               .expect(ERR_CONNECT)
        }
    };
}

pub fn open_stream() -> &'static PgPool {
    &STREAM.db_connection
}

pub fn can_connect() -> bool {
    open_stream().get().is_ok()
}