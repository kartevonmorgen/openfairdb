use maud::{html, Markup, DOCTYPE};
use rocket::request::FlashMessage;

const MAIN_CSS_URL: &str = "/main.css";

pub fn page(
    title: &str,
    email: Option<&str>,
    flash: Option<FlashMessage>,
    h: Option<Markup>,
    content: Markup,
) -> Markup {
    html! {
        (DOCTYPE)
        head{
            meta charset="utf-8";
            meta name="viewport" content="width=device-width, initial-scale=1, shrink-to-fit=no";
            title {(title)}
            link rel="stylesheet" href=(MAIN_CSS_URL);
            @if let Some(h) = h {
               (h)
            }
        }
        body{
            (flash_msg(flash))
            (header(email))
            (content)
        }
    }
}

fn flash_msg(flash: Option<FlashMessage>) -> Markup {
    html! {
        @if let Some(msg) = flash {
            div class=(format!("flash {}", msg.name())) {
                (msg.msg())
            }
        }
    }
}

fn header(email: Option<&str>) -> Markup {
    html! {
    header {
        @if let Some(email) = email {
            div class="msg" { "Your are logged in as " span class="email" { (email) } }
            nav {
                a href="/" { "search" }
                a href="/dashboard" { "dashboard" }
                form class="logout" action="/logout" method ="POST" {
                    input type="submit" value="logout";
                }
            }
        }
        @ else {
            nav {
                a href="/login"  { "login" }
                a href="/register" { "register" }
            }
        }
    }
    }
}
