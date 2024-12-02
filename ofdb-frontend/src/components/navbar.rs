use leptos::*;
use leptos_router::*;

use ofdb_boundary::User;

use crate::Page;

#[component]
pub fn NavBar<F, T>(user: Signal<Option<User>>, on_logout: F, on_copy_token: T) -> impl IntoView
where
    F: Fn() + 'static + Copy,
    T: Fn() + 'static + Copy,
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
            <UserMenu user on_logout on_copy_token />
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
            <UserMenu user on_logout on_copy_token />
          </menu>
        </div>
      </nav>
    }
}

#[component]
fn UserMenu<F, T>(user: Signal<Option<User>>, on_logout: F, on_copy_token: T) -> impl IntoView
where
    F: Fn() + 'static + Copy,
    T: Fn() + 'static + Copy,
{
    let memorized_user = create_memo(move |_| user.get());

    move || match memorized_user.get() {
        Some(user) => view! {  <UserMenuItems user on_logout on_copy_token /> }.into_view(),
        None => view! {  <PublicMenuItems /> }.into_view(),
    }
}

#[allow(clippy::needless_pass_by_value)] // TODO
#[component]
fn UserMenuItems<F, T>(user: User, on_logout: F, on_copy_token: T) -> impl IntoView
where
    F: Fn() + 'static + Clone,
    T: Fn() + 'static + Clone,
{
    let profile_is_open = RwSignal::new(false);
    let logout = Callback::new(move |()| {
        on_logout();
    });
    let copy_token = Callback::new(move |()| {
        on_copy_token();
    });
    let email = user.email;

    view! {
      <div class="grid grid-rows-1 grid-flow-col gap-4">
        <MenuItem page = Page::Home label = "Search" />
        <MenuItem page = Page::Events label = "Events" />
        <MenuItem page = Page::Dashboard label = "Dashboard" />
        <div>
          <div class="relative">
            <button
              on:click=move|_| { profile_is_open.update(|d|*d = !*d); }
              type="button"
              class="relative flex rounded-full bg-white text-sm focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:ring-offset-2">
              <span class="absolute -inset-1.5"></span>
              <span class="sr-only">"Open user menu"</span>
              <svg
                xmlns="http://www.w3.org/2000/svg"
                fill="none"
                viewBox="0 0 24 24"
                stroke-width="1.5"
                stroke="currentColor"
                class="size-6"
              >
                <path
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  d="M17.982 18.725A7.488 7.488 0 0 0 12 15.75a7.488 7.488 0 0 0-5.982 2.975m11.963 0a9 9 0 1 0-11.963 0m11.963 0A8.966 8.966 0 0 1 12 21a8.966 8.966 0 0 1-5.982-2.275M15 9.75a3 3 0 1 1-6 0 3 3 0 0 1 6 0Z"
                />
              </svg>
            </button>
          </div>
          <Show when=move||profile_is_open.get()>
            <div
              class="absolute right-0 z-10 mt-2 w-48 origin-top-right rounded-md bg-white py-1 shadow-lg ring-1 ring-black/5 focus:outline-none"
              role="menu"
              aria-orientation="vertical"
              tabindex="-1"
            >
              <a
                href="#"
                class="block px-4 py-2 text-sm text-gray-700"
                role="menuitem"
                tabindex="-1"
                on:click = move |_| {
                  copy_token.call(());
                  profile_is_open.set(false);
                }
              >
                "Copy API token to clipboard"
              </a>
              <a
                href="#"
                class="block px-4 py-2 text-sm text-gray-700"
                role="menuitem"
                tabindex="-1"
                on:click= move |_| {
                  logout.call(());
                  profile_is_open.set(false);
                }
              >
                "Sign out "
                <span class="text-gray-400">
                  "(" { email.clone() } ")"
                </span>
              </a>
            </div>
          </Show>
        </div>
      </div>
    }
}

#[component]
fn PublicMenuItems() -> impl IntoView {
    view! {
      <MenuItem page = Page::Home label = "Search" />
      <MenuItem page = Page::Events label = "Events" />
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
