use leptos::*;
use ofdb_app::App;

fn main() {
    _ = console_log::init_with_level(log::Level::Debug);
    console_error_panic_hook::set_once();
    log::info!("Start web application");
    mount_to_body(|| view! { <App /> });
}
