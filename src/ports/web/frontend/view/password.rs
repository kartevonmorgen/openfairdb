use super::page::*;
use maud::{html, Markup};
use rocket::request::FlashMessage;

pub fn reset_password_request(flash: Option<FlashMessage>, action: &str) -> Markup {
    page(
        "Request password reset",
        None,
        flash,
        None,
        html! {
          h2 { "Password reset" }
          p { "Please enter your email address to reset your password." }
          form class="reset-pw-req" action=(action) method="POST" {
              fieldset{
                label {
                    "eMail:"
                    br;
                    input type="text" name="email" placeholder="eMail address";
                }
                br;
                input type="submit" value="reset";
              }
          }
        },
    )
}

pub fn reset_password_request_ack(flash: Option<FlashMessage>) -> Markup {
    page(
        "Request password reset",
        None,
        flash,
        None,
        html! {
          h2 { "Password reset" }
          p {
            "Your request to reset your password was successfully send."
            br;
            "Please look into your email inbox to continue."
          }
        },
    )
}

pub fn reset_password(flash: Option<FlashMessage>, action: &str, token: &str) -> Markup {
    page(
        "New password",
        None,
        flash,
        None,
        html! {
          form class="reset-pw" action=(action) method="POST" {
              fieldset{
                label {
                    "eMail:"
                    br;
                    input type="text" name="email" placeholder="eMail address";
                }
                br;
                label{
                    "New password:"
                    br;
                    input type="password" name="new_password" placeholder="new password";
                }
                br;
                label{
                    "New password (repeated):"
                    br;
                    input type="password" name="new_password_repeated" placeholder="repeat new password";
                }
                br;
                input type="hidden" name="token" value=(token);
                input type="submit" value="save";
              }
          }
        },
    )
}

pub fn reset_password_ack(flash: Option<FlashMessage>) -> Markup {
    page(
        "New password",
        None,
        flash,
        None,
        html! {
          h2 { "New password" }
          p { "Your password changed successfully." }
          p { a href = "/login" { "Login with your new password."} }
        },
    )
}
