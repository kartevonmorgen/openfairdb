const PICNIC_CSS: &str = include_str!("./picnic.min.css");
const MAIN_CSS: &str = include_str!("./main.css");

use maud::{html, Markup};

fn page(content: Markup) -> Markup {
    html! {
        head {
            meta charset="utf-8" ;
            meta name="viewport" content="width=device-width";
            title { "OpenFairDB" }
            style { (PICNIC_CSS) }
            style { (MAIN_CSS) }
        }
        body {
            nav {
                a href="/admin" class="brand" {
                    span { "OpenFairDB" }
                }
            }
            main {
                section {
                    (content)
                }
            }
        }
    }
}

pub fn index(flash: Option<&str>) -> Markup {
    page(html!{
        p {
            "Welcome to the admin interface of OpenFairDB." br;
            "Please login to continue."
        }
        @if let Some(msg) = flash {
            p {
                (format!("Error: {}", msg))
            }
        }
        form id="login" action = "login" method="post" accept-charset="utf-8" {
            fieldset {
                label {
                    "Username"
                    input type="text" name="username";
                }
                label {
                    "Password"
                    input type="password" name="password";
                }
            }
            input type="submit" value="login";
        }
    })
}

pub fn user_dashboard(username: &str) -> Markup {
    dashboard(html!(p {
            "Hi "
            b {(username)}
            " your are logged in as user."
        }))
}

pub fn admin_dashboard(username: &str) -> Markup {
    dashboard(html!(p {
            "Hi "
            b {(username)}
            " your are logged in as administrator."
        }))
}

fn dashboard(content: Markup) -> Markup {
    page(html!{
        h1 { "OpenFairDB Dashboard" }
        (content)
        form action="/admin/logout" method="post" accept-charset="utf-8" {
            input type="submit" name="logout" id="logout" value="logout";
        }
    })
}
