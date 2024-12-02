use leptos::*;
use leptos_router::*;

use ofdb_boundary::Credentials;

use crate::{
    api::{self, PublicApi, UserApi},
    components::*,
    Page,
};

#[component]
pub fn Login<F>(public_api: PublicApi, on_success: F) -> impl IntoView
where
    F: Fn(UserApi) + 'static + Clone,
{
    let (login_error, set_login_error) = create_signal(None::<String>);
    let (wait_for_response, set_wait_for_response) = create_signal(false);

    let login_action = create_action(move |credentials: &Credentials| {
        log::info!("Logging in with {email}", email = credentials.email);
        let credentials = credentials.to_owned();
        let on_success = on_success.clone();
        async move {
            set_wait_for_response.update(|w| *w = true);
            let result = public_api.login(&credentials).await;
            set_wait_for_response.update(|w| *w = false);
            match result {
                Ok(res) => {
                    set_login_error.update(|e| *e = None);
                    on_success(res);
                }
                Err(err) => {
                    let msg = match err {
                        api::Error::Fetch(js_err) => {
                            format!("{js_err:?}")
                        }
                        api::Error::Api(err) => err.message,
                    };
                    log::error!("Unable to login with {}: {msg}", credentials.email);
                    set_login_error.update(|e| *e = Some(msg));
                }
            }
        }
    });

    let disabled = Signal::derive(move || wait_for_response.get());

    view! {
      <section>
        <div class="container py-12 px-6 mx-auto">
          <div class="flex justify-center items-center flex-wrap h-full g-6 text-gray-800">
            <div class="xl:w-6/12">
              <div class="block bg-white shadow-lg rounded-lg">
                <div class="lg:flex lg:flex-wrap g-0">
                  <div class="px-4 md:px-0 mx-auto">
                    <div class="md:p-12 md:mx-6">
                      <CredentialsForm
                          title = "Login"
                          description = "Please login to your account"
                          submit_credentials_label = "Log in"
                          initial_credentials = Credentials::default()
                          submit_credentials_action = login_action
                          error = login_error.into()
                          disabled
                      />
                      <div class="text-center pt-1 mb-6 pb-1">
                        <A
                          href=Page::ResetPassword.path()
                          class="text-gray-500".to_string()>
                          "Forgot password?"
                        </A>
                      </div>
                      <div class="flex items-center justify-between pb-6">
                        <p class="mb-0 mr-2 text-gray-600">"Don't have an account?"</p>
                        <A
                          href=Page::Register.path()
                          class="inline-block px-6 py-2 border-2 border-kvm-raspberry-light text-kvm-raspberry font-medium text-xs leading-tight uppercase rounded hover:border-kvm-raspberry hover:bg-kvm-raspberry-light hover:bg-opacity-25 focus:outline-none focus:ring-0 transition duration-150 ease-in-out"
                        >
                          "Register"
                        </A>
                      </div>
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
