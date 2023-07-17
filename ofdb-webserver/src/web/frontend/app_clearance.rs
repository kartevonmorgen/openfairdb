use std::{borrow::Cow, ffi::OsStr, path::PathBuf};

use rocket::{self, get, http::ContentType, response::content::RawHtml};
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "../ofdb-app-clearance/dist/"]
struct ClearanceAsset;

#[get("/clearance")]
pub fn get_index() -> Option<RawHtml<Cow<'static, [u8]>>> {
    ClearanceAsset::get("index.html").map(|html| RawHtml(html.data))
}

#[get("/clearance/login")]
pub fn get_login() -> Option<RawHtml<Cow<'static, [u8]>>> {
    get_index()
}

#[get("/clearance/logout")]
pub fn get_logout() -> Option<RawHtml<Cow<'static, [u8]>>> {
    get_index()
}

#[get("/clearance/<file..>")]
pub fn get_file(file: PathBuf) -> Option<(ContentType, Cow<'static, [u8]>)> {
    let filename = file.display().to_string();
    let asset = ClearanceAsset::get(&filename)?;
    let content_type = file
        .extension()
        .and_then(OsStr::to_str)
        .and_then(ContentType::from_extension)
        .unwrap_or(ContentType::Bytes);
    Some((content_type, asset.data))
}

#[cfg(test)]
mod tests {
    use crate::web::frontend::tests::setup;
    use rocket::http::Status;

    #[test]
    fn index_html() {
        let (client, _, _) = setup();
        let res = client.get("/clearance").dispatch();
        assert_eq!(res.status(), Status::Ok);
    }

    #[test]
    fn web_fonts() {
        let (client, _, _) = setup();
        let res = client
            .get("/clearance/webfonts/fa-solid-900.woff2")
            .dispatch();
        assert_eq!(res.status(), Status::Ok);
    }
}
