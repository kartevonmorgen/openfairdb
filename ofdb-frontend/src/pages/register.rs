use leptos::*;
use leptos_router::*;

use ofdb_boundary::Credentials;

use crate::{
    api::{self, PublicApi},
    components::*,
    Page,
};

#[component]
pub fn Register(public_api: Signal<PublicApi>) -> impl IntoView {
    let (register_response, set_register_response) = create_signal(None::<()>);
    let (register_error, set_register_error) = create_signal(None::<String>);
    let (wait_for_response, set_wait_for_response) = create_signal(false);

    let register_action = create_action(move |credentials: &Credentials| {
        log::info!(
            "Registering new account for {email}",
            email = credentials.email
        );
        let credentials = credentials.to_owned();
        async move {
            set_wait_for_response.update(|w| *w = true);
            let result = public_api.get_untracked().register(&credentials).await;
            set_wait_for_response.update(|w| *w = false);
            match result {
                Ok(res) => {
                    set_register_response.update(|v| *v = Some(res));
                    set_register_error.update(|e| *e = None);
                }
                Err(err) => {
                    let msg = match err {
                        api::Error::Fetch(js_err) => {
                            format!("{js_err:?}")
                        }
                        api::Error::Api(err) => err.message,
                    };
                    log::warn!(
                        "Unable to register new account for {}: {msg}",
                        credentials.email
                    );
                    set_register_error.update(|e| *e = Some(msg));
                }
            }
        }
    });

    let disabled = Signal::derive(move || wait_for_response.get());

    view! {
      <section>
        <div class="container py-12 px-6 mx-auto">
          <div class="flex justify-center items-center flex-wrap h-full g-6 text-gray-800">
            <div class="xl:w-10/12">
              <div class="block bg-white shadow-lg rounded-lg">
                <div class="lg:flex lg:flex-wrap g-0">
                  <div class="lg:w-6/12 px-4 md:px-0">
                    <div class="md:p-12 md:mx-6">
                      <CredentialsForm
                          title = "Register"
                          description = "Please enter the desired credentials"
                          submit_credentials_label = "Register"
                          initial_credentials = Credentials::default()
                          submit_credentials_action = register_action
                          error = register_error.into()
                          disabled
                      />
                      <div class="flex items-center justify-between pb-6">
                        <p class="mb-0 mr-2 text-gray-600">"You already have an account?"</p>
                        <A
                          href=Page::Login.path()
                          class="inline-block px-6 py-2 border-2 border-kvm-raspberry-light text-kvm-raspberry font-medium text-xs leading-tight uppercase rounded hover:border-kvm-raspberry hover:bg-kvm-raspberry-light hover:bg-opacity-25 focus:outline-none focus:ring-0 transition duration-150 ease-in-out"
                        >
                          "Login"
                        </A>
                      </div>
                    </div>
                  </div>
                  <div class="lg:w-6/12 flex items-center lg:rounded-r-lg rounded-b-lg lg:rounded-bl-none bg-kvm-blue-light">
                    <div class="px-4 py-6 md:p-12 md:mx-6">{move || match register_response.get() {
                        Some(()) => view!{
                          <h4 class="text-xl font-semibold mb-6">"Successfully registered"</h4>
                          <p class="text-sm">
                            "Congratulations! You've successfully registered your OpenFairDB account."
                          </p>
                          <p class="text-sm">
                            "Now check your email inbox and confirm the validity of your email address."
                          </p>
                        }.into_view(),
                        None => view!{
                          <h4 class="text-xl font-semibold mb-6">"What you can do with OpenFairDB"</h4>
                          <p class="text-sm">
                            "OpenFairDB is an Open Source Database to map sustainable places around the world. With an account you can subscribe to a map section so that you are always informed about new or changed entries."
                          </p>
                        }.into_view()
                      }}
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </section>
    }
}
