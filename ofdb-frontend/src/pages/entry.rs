use leptos::*;
use leptos_router::*;

use crate::api::PublicApi;

#[component]
pub fn Entry(public_api: PublicApi) -> impl IntoView {
    // -- signals -- //

    let params = use_params_map();
    let entry = create_rw_signal(None::<ofdb_boundary::Entry>);

    // -- actions -- //

    let fetch_entry = create_action(move |id: &String| {
        let id = id.to_owned();
        async move {
            match public_api.entries(&[&id]).await {
                Ok(mut entries) => {
                    debug_assert_eq!(entries.len(), 1);
                    let e = entries.remove(0);
                    debug_assert_eq!(e.id, id);
                    entry.update(|x| *x = Some(e));
                }
                Err(err) => {
                    log::warn!("Unable to fetch entry: {err}");
                }
            }
        }
    });

    // -- effects -- //

    create_effect(move |_| {
        if let Some(id) = params.with(|p| p.get("id").cloned()) {
            fetch_entry.dispatch(id);
        }
    });

    move || {
        if let Some(entry) = entry.get() {
            view! { <EntryProfile entry /> }.into_view()
        } else {
            view!{
              <div class="mx-auto text-center max-w-7xl px-4 mt-12 pb-16 sm:px-6 sm:pb-24 lg:px-8">
                <h2 class="text-3xl font-bold tracking-tight text-gray-900 sm:text-4xl">"Entry not found"</h2>
              </div>
            }.into_view()
        }
    }
}

#[component]
fn EntryProfile(entry: ofdb_boundary::Entry) -> impl IntoView {
    let ofdb_boundary::Entry {
        title,
        description,
        image_url,
        street,
        city,
        zip,
        contact_name,
        telephone,
        email,
        homepage,
        tags,
        version,
        opening_hours,
        ..
    } = entry;

    view! {
      <div class="bg-white">
        <div aria-hidden="true" class="relative">
          <img src={ image_url } alt="" class="h-96 w-full object-cover object-center" />
          <div class="absolute inset-0 bg-gradient-to-t from-white"></div>
        </div>

        <div class="relative mx-auto -mt-12 max-w-7xl px-4 pb-16 sm:px-6 sm:pb-24 lg:px-8">
          <div class="mx-auto max-w-2xl text-center lg:max-w-4xl">
            <h2 class="text-3xl font-bold tracking-tight text-gray-900 sm:text-4xl">{ title }</h2>
            <p class="mt-4 text-gray-500">{ description }</p>
          </div>

          <dl class="mx-auto mt-16 grid max-w-2xl grid-cols-1 gap-x-6 gap-y-10 sm:grid-cols-2 sm:gap-y-16 lg:max-w-none lg:grid-cols-3 lg:gap-x-8">
            <div class="border-t border-gray-200 pt-4">
              <dt class="font-medium text-gray-900">"Address"</dt>
              <dd class="mt-2 text-sm text-gray-500">
                { street }
                <br />
                { zip } " " { city }
              </dd>
            </div>
            <div class="border-t border-gray-200 pt-4">
              <dt class="font-medium text-gray-900">"Contact"</dt>
              <dd class="mt-2 text-sm text-gray-500">
              { contact_name }
              <br />
              { telephone }
              <br />
              { email }
              </dd>
            </div>
            <div class="border-t border-gray-200 pt-4">
              <dt class="font-medium text-gray-900">"Web"</dt>
              <dd class="mt-2 text-sm text-gray-500">{ homepage }</dd>
            </div>
            <div class="border-t border-gray-200 pt-4">
              <dt class="font-medium text-gray-900">"Tags"</dt>
              <dd class="mt-2 text-sm text-gray-500">
              { tags.join(" ") }
              </dd>
            </div>
            <div class="border-t border-gray-200 pt-4">
              <dt class="font-medium text-gray-900">"Opening Hours"</dt>
              <dd class="mt-2 text-sm text-gray-500">{ opening_hours }</dd>
            </div>
            <div class="border-t border-gray-200 pt-4">
              <dt class="font-medium text-gray-900">"Version"</dt>
              <dd class="mt-2 text-sm text-gray-500">{ version }</dd>
            </div>
          </dl>
        </div>
      </div>
    }
}
