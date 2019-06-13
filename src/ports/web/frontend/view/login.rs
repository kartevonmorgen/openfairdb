use super::page::*;
use maud::{html, Markup};
use rocket::request::FlashMessage;

pub fn login(flash: Option<FlashMessage>) -> Markup {
    page(
        "Login",
        None,
        flash,
        None,
        html! {
          form class="login" action="login" method="POST" {
              fieldset{
                label {
                    "eMail:"
                    br;
                    input type="email" name="email" placeholder="eMail address";
                }
                    br;
                label{
                    "Password:"
                    br;
                    input type="password" name="password" placeholder="Password";
                }
                br;
                input type="submit" value="login";
                a href="/password_reset_request" { "reset password" }
              }
          }
        },
    )
}
