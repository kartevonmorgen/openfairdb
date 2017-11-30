mod schema {
    infer_schema!("dotenv:DATABASE_URL");
}
mod models;
mod util;
mod connection;
