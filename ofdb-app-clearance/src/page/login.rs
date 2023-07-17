use gloo_storage::{SessionStorage, Storage};
use leptos::*;
use leptos_router::Form;

use crate::Page;

#[component]
pub fn Login(cx: Scope, invalid_token: Signal<bool>) -> impl IntoView {
    let show_token = create_rw_signal(cx, false);
    let token = create_rw_signal(cx, String::new());

    let token_field_type = move || if show_token.get() { "text" } else { "password" };

    view! { cx,
        <div class="container">
            <div class="section">
                <h2 class="title">"Login"</h2>
                {move || {
                    invalid_token
                        .get()
                        .then(|| {
                            view! { cx,
                                <div style="color:red;padding-bottom:20px">
                                    "Your API token is invalid. Please try again"
                                </div>
                            }
                        })
                }}
                <div style="pdding-bottom:20px">"Enter your organization's API token: "</div>
                <Form
                    method="GET"
                    action=Page::Home.path()
                    on:submit=move |ev| {
                        ev.prevent_default();
                        show_token.update(|v| *v = false);
                        if let Err(err) = SessionStorage::set(crate::TOKEN_KEY, token.get()) {
                            log::debug!("{err}");
                        }
                        let form: web_sys::HtmlFormElement = event_target(&ev);
                        if let Err(err) = form.submit() {
                            log::error!("{err:?}");
                        }
                        log::info!("foo");
                    }
                >
                    <input
                        class="input"
                        name="username"
                        type="text"
                        value=crate::TITLE
                        style="display:none"
                    />
                    <div class="field">
                        <label class="label">"Token"</label>
                        <input
                            class="input"
                            name="password"
                            type=token_field_type
                            style="width:50%"
                            on:change=move |ev| {
                                let value = event_target_value(&ev);
                                token.update(|v| *v = value);
                            }
                        />
                    </div>
                    <div class="field">
                        <div class="control">
                            <label class="checkbox">
                                <input
                                    class="checkbox"
                                    type="checkbox"
                                    on:click=move |_| {
                                        show_token.update(|v| *v = !*v);
                                    }
                                    checked=move || show_token.get()
                                />
                                " show token"
                            </label>
                        </div>
                    </div>
                    <input class="button is-primary" type="submit" value="Login"/>
                </Form>
            </div>
        </div>
    }
}
