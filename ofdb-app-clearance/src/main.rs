pub fn main() {
    console_log::init_with_level(log::Level::Info).expect("Browser's console");
    console_error_panic_hook::set_once();
    ofdb_app_clearance::run();
}
