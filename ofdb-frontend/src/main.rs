use leptos::*;
use ofdb_app::App;

#[cfg(debug_assertions)]
const LOG_LEVEL: log::Level = log::Level::Debug;

#[cfg(not(debug_assertions))]
const LOG_LEVEL: log::Level = log::Level::Info;

fn main() {
    _ = console_log::init_with_level(LOG_LEVEL);
    console_error_panic_hook::set_once();
    log::info!("Start web application");
    mount_to_body(|| view! { <App /> });
}
