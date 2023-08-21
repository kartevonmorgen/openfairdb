use leptos::*;
use leptos_router::*;

use crate::page::Page;

#[component]
pub fn Navbar(logged_in: Signal<bool>) -> impl IntoView {
    let menu_is_active = create_rw_signal(false);

    view! {
        <nav class="navbar" roler="navigation" aria-label="main navigation">
            <div class="navbar-brand">
                <A class="navbar-item" href=Page::Home.path()>
                    {crate::TITLE}
                </A>
                <BurgerMenu menu_is_active/>
            </div>
            ,
            <div class=move || {
                if menu_is_active.get() { "navbar-menu is-active" } else { "navbar-menu" }
            }>
                <div class="navbar-start"></div>
                <div class="navbar-end">
                    <div class="navbar-item">
                        <div class="buttons">
                            {move || {
                                if logged_in.get() {
                                    view! {
                                        <A class="button is-light" href=Page::Logout.path()>
                                            "Log out"
                                        </A>
                                    }
                                        .into_view()
                                } else {
                                    view! {
                                        <A href=Page::Login.path() class="button is-light">
                                            "Log in"
                                        </A>
                                    }
                                        .into_view()
                                }
                            }}
                        </div>
                    </div>
                </div>
            </div>
        </nav>
    }
}

#[component]
pub fn BurgerMenu(menu_is_active: RwSignal<bool>) -> impl IntoView {
    view! {
        <a
            on:click=move |_| {
                menu_is_active.update(|v| *v = !*v);
            }
            class=move || {
                if menu_is_active.get() { "navbar-burger is-active" } else { "navbar-burger" }
            }
            role="button"
            aria-label="menu"
            aria-expanded="false"
        >
            <span aria-hidden="true"></span>
            <span aria-hidden="true"></span>
            <span aria-hidden="true"></span>
        </a>
    }
}
