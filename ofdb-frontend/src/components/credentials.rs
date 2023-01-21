use leptos::{ev, *};

#[component]
pub fn CredentialsForm(
    cx: Scope,
    title: &'static str,
    description: &'static str,
    action_label: &'static str,
    action: Action<(String, String), ()>,
    error: Signal<Option<String>>,
    disabled: Signal<bool>,
) -> impl IntoView {
    let (password, set_password) = create_signal(cx, String::new());
    let (email, set_email) = create_signal(cx, String::new());

    let dispatch_action = move || action.dispatch((email.get(), password.get()));

    let button_is_disabled = Signal::derive(cx, move || {
        disabled.get() || password.get().is_empty() || email.get().is_empty()
    });

    view! { cx,
      <form on:submit=|ev|ev.prevent_default()>
        //<p>{ title }</p>
        <div class="text-center">
          // TODO: Add logo image
          // <img
          //   class="mx-auto w-48"
          //   src="..."
          //   alt="logo"
          // />
          <h4 class="text-xl font-semibold mt-1 mb-12 pb-1">{ title }</h4>
        </div>
        <p class="mb-4 text-gray-600">{ description }</p>
        {move || error.get().map(|err| view!{ cx,
          <p class="mb-4 text-red-700">{ err }</p>
        })}
        <div class="mb-4">
          <input
            type = "email"
            required
            placeholder = "Email address"
            class="form-control block w-full px-3 py-1.5 text-base font-normal text-gray-700 bg-white bg-clip-padding border border-solid border-gray-300 rounded transition ease-in-out m-0 focus:text-gray-700 focus:bg-white focus:border-kvm-blue focus:outline-none"
            prop:disabled = move || disabled.get()
            on:keyup = move |ev: ev::KeyboardEvent| {
              let val = event_target_value(&ev);
              set_email.update(|v|*v = val);
            }
            // The `change` event fires when the browser fills the form automatically,
            on:change = move |ev| {
              let val = event_target_value(&ev);
              set_email.update(|v|*v = val);
            }
          />
        </div>
        <div class="mb-4">
          <input
            type = "password"
            required
            placeholder = "Password"
            class="form-control block w-full px-3 py-1.5 text-base font-normal text-gray-700 bg-white bg-clip-padding border border-solid border-gray-300 rounded transition ease-in-out m-0 focus:text-gray-700 focus:bg-white focus:border-kvm-blue focus:outline-none"
            prop:disabled = move || disabled.get()
            on:keyup = move |ev: ev::KeyboardEvent| {
              match &*ev.key() {
                  "Enter" => {
                     dispatch_action();
                  }
                  _=> {
                     let val = event_target_value(&ev);
                     set_password.update(|p|*p = val);
                  }
              }
            }
            // The `change` event fires when the browser fills the form automatically,
            on:change = move |ev| {
              let val = event_target_value(&ev);
              set_password.update(|p|*p = val);
            }
          />
        </div>
        <div class="text-center pt-1 mb-12 pb-1">
          <button
            prop:disabled = move || button_is_disabled.get()
            on:click = move |_| dispatch_action()
            class="inline-block px-6 py-2.5 font-medium text-xs leading-tight uppercase rounded shadow-md hover:bg-kvm-blue hover:text-white hover:shadow-lg focus:shadow-lg focus:outline-none focus:ring-0 active:shadow-lg transition duration-150 ease-in-out w-full mb-3 bg-kvm-blue-light"
          >
          { action_label }
          </button>
        </div>
      </form>
    }
}
