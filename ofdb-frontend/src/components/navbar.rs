use crate::Page;
use leptos::*;
use leptos_router::*;

use ofdb_boundary::User;

#[component]
pub fn NavBar<F>(user: Signal<Option<User>>, on_logout: F) -> impl IntoView
where
    F: Fn() + 'static + Copy,
{
    let (menu_open, set_menu_open) = create_signal(false);

    view! {
      <nav class="relative container mx-auto p-6">
        <div class="flex items-center justify-between">

          // Logo
          <div class="pt-2 font-bold">
            <A href = Page::Home.path()>"OpenFairDB"</A>
          </div>

          // Menu items
          <div class="hidden space-x-6 md:flex">
            <UserMenu user on_logout />
          </div>

          // Hamburger Icon
          <button
            class = {move ||
              if menu_open.get() {
                "open block hamburger md:hidden focus:outline-none"
              } else {
                "block hamburger md:hidden focus:outline-none"
              }
            }
            on:click = move |_| set_menu_open.update(|s|*s = !*s)
          >
            <span class="hamburger-top"></span>
            <span class="hamburger-middle"></span>
            <span class="hamburger-bottom"></span>
          </button>
        </div>

        // Mobile Menu
        <div class="md:hidden">
          <menu
            class = {move ||
              if menu_open.get() {
                "absolute flex flex-col items-center self-end py-8 mt-10 space-y-6 font-bold bg-white sm:w-auto sm:self-center left-6 right-6 drop-shadow-md"
              } else {
                "hidden absolute flex-col items-center self-end py-8 mt-10 space-y-6 font-bold bg-white sm:w-auto sm:self-center left-6 right-6 drop-shadow-md"
              }
            }>
            <UserMenu user on_logout />
          </menu>
        </div>
      </nav>
    }
}

#[component]
fn UserMenu<F>(user: Signal<Option<User>>, on_logout: F) -> impl IntoView
where
    F: Fn() + 'static + Copy,
{
    let memorized_user = create_memo(move |_| user.get());

    move || match memorized_user.get() {
        Some(user) => view! {  <UserMenuItems user on_logout /> }.into_view(),
        None => view! {  <PublicMenuItems /> }.into_view(),
    }
}

#[component]
fn UserMenuItems<F>(user: User, on_logout: F) -> impl IntoView
where
    F: Fn() + 'static + Clone,
{
    view! {
      <MenuItem page = Page::Home label = "Search" />
      <MenuItem page = Page::Dashboard label = "Dashboard" />
      <a href="#" on:click= move |_| on_logout()>
      {
        format!("Logout ({})", user.email)
      }
      </a>
    }
}

#[component]
fn PublicMenuItems() -> impl IntoView {
    view! {
      <MenuItem page = Page::Home label = "Search" />
      <MenuItem page = Page::Dashboard label = "Dashboard" />
      <MenuItem page = Page::Login label = "Login" />
      <MenuItem page = Page::Register label = "Register" />
    }
}

// TODO: Highlight active item.
#[component]
fn MenuItem(page: Page, label: &'static str) -> impl IntoView {
    view! {
      <A href=page.path() class="hover:text-gray-600".to_string()>{ label }</A>
    }
}
