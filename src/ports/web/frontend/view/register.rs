use super::page::*;
use maud::{html, Markup};
use rocket::request::FlashMessage;

pub fn register(flash: Option<FlashMessage>) -> Markup {
    page(
        "Register",
        None,
        flash,
        None,
        html! {
          form class="register" action="register" method="POST" {
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
                input type="submit" value="register";
              }
          }
        },
    )
}
