use leptos::*;

use ofdb_boundary::*;
use ofdb_frontend_api::{PublicApi, UserApi};

#[component]
pub fn Dashboard(public_api: PublicApi, user_api: Signal<Option<UserApi>>) -> impl IntoView {
    // -- signals -- //

    let subscriptions = create_rw_signal(None::<Vec<BboxSubscription>>);

    // -- actions -- //

    let fetch_entries_count_action =
        create_action(move |_| async move { public_api.count_entries().await });

    let fetch_tags_count_action =
        create_action(move |_| async move { public_api.count_tags().await });

    let fetch_bbox_subscriptions = create_action(move |api: &UserApi| {
        let api = api.clone();
        async move {
            match api.bbox_subscriptions().await {
                Ok(subs) => {
                    subscriptions.update(|s| *s = Some(subs));
                }
                Err(err) => {
                    log::warn!("Unable to fetch subscriptions: {err}");
                }
            }
        }
    });

    let delete_bbox_subscriptions = create_action(move |api: &UserApi| {
        let api = api.clone();
        async move {
            match api.unsubscribe_all_bboxes().await {
                Ok(_) => {
                    log::info!("Deleted all subscriptions");
                    subscriptions.update(|s| *s = None);
                }
                Err(err) => {
                    log::warn!("Unable to delete subscriptions: {err}");
                }
            }
        }
    });

    fetch_entries_count_action.dispatch(());
    fetch_tags_count_action.dispatch(());

    // -- effects -- //

    create_effect(move |_| {
        if let Some(user_api) = user_api.get() {
            fetch_bbox_subscriptions.dispatch(user_api);
        }
    });

    // -- memos -- //

    let memorized_subscriptions = create_memo(move |_| subscriptions.get());

    // -- callbacks -- //

    let on_bbox_delete = move |id| match user_api.get() {
        Some(user_api) => {
            delete_bbox_subscriptions.dispatch(user_api);
        }
        None => {
            log::warn!("Cannot delete subscription ({id}): not logged in");
        }
    };

    view! {
      <section class="container mx-auto">
        <div class="mx-auto max-w-7xl py-6 sm:px-6 lg:px-8">
          <div class="mx-auto max-w-none">
            <div class="overflow-hidden bg-white sm:rounded-lg sm:shadow">
              <div class="border-b border-gray-200 bg-white px-4 py-5 sm:px-6">
                <h3 class="text-base font-semibold leading-6 text-gray-900">"Statistics"</h3>
              </div>
              <dl class="grid grid-cols-1 sm:grid-cols-2">

                <div class="m-5 overflow-hidden rounded-lg bg-white px-4 py-5 shadow sm:p-6">
                  <dt class="truncate text-sm font-medium text-gray-500">"Number of Entries"</dt>
                  <dd class="mt-1 text-3xl font-semibold tracking-tight text-gray-900">
                    <DisplayNumber
                      number = fetch_entries_count_action.value().into()
                    />
                  </dd>
                </div>

                <div class="m-5 overflow-hidden rounded-lg bg-white px-4 py-5 shadow sm:p-6">
                  <dt class="truncate text-sm font-medium text-gray-500">"Number of Tags"</dt>
                  <dd class="mt-1 text-3xl font-semibold tracking-tight text-gray-900">
                    <DisplayNumber
                      number = fetch_tags_count_action.value().into()
                    />
                  </dd>
                </div>

              </dl>
            </div>
          </div>
        </div>
        <Show
          when = move || user_api.get().is_some()
          fallback = || view! {  }
        >
          <div class="mx-auto max-w-7xl py-6 sm:px-6 lg:px-8">
            <div class="mx-auto max-w-none">
              <div class="overflow-hidden bg-white sm:rounded-lg sm:shadow">
                <div class="border-b border-gray-200 bg-white px-4 py-5 sm:px-6">
                  <h3 class="text-base font-semibold leading-6 text-gray-900">"Subscriptions"</h3>
                </div>
                { move || match memorized_subscriptions.get() {
                    Some(subs) => view! {
                      <ul role="list" class="divide-y divide-gray-100">
                        <For
                          each=move||subs.clone()
                          key=|sub|sub.id.clone()
                          view=move | subscription|view!{  <BboxSubscriptionListElement subscription on_delete = on_bbox_delete /> }
                        />
                      </ul>
                    }.into_view(),
                    None => view! {
                      <p class="text-gray-500 p-5">"There are currently no active subscriptions."</p>
                    }.into_view()
                  }
                }
              </div>
            </div>
          </div>
        </Show>
      </section>
    }
}

#[component]
fn BboxSubscriptionListElement<F>(subscription: BboxSubscription, on_delete: F) -> impl IntoView
where
    F: Fn(String) + 'static + Copy,
{
    let BboxSubscription {
        id,
        south_west_lat,
        south_west_lng,
        north_east_lat,
        north_east_lng,
    } = subscription;

    view! {
      <li class="flex items-center justify-between gap-x-6 p-5">
        <div class="min-w-0">
          <div class="flex items-start gap-x-3">
            <p class="text-sm font-semibold leading-6 text-gray-900">"Bounding box subscription"</p>
            <p class="rounded-md whitespace-nowrap mt-0.5 px-1.5 py-0.5 text-xs font-medium ring-1 ring-inset text-green-700 bg-green-50 ring-green-600/20">"active"</p>
          </div>
          <div class="mt-1 flex items-center gap-x-2 text-xs leading-5 text-gray-500">
            <p class="whitespace-nowrap">
              { format!("{south_west_lat:.1}") }
               " / "
              { format!("{south_west_lng:.1}") }
              " : "
              { format!("{north_east_lat:.1}") }
              " / "
              { format!("{north_east_lng:.1}") }
            </p>
            <svg viewBox="0 0 2 2" class="h-0.5 w-0.5 fill-current">
              <circle cx="1" cy="1" r="1" />
            </svg>
            <p class="truncate">{ id.clone() }</p>
          </div>
        </div>
        <div class="flex flex-none items-center gap-x-4">
          <a
            href="#"
            class="hidden rounded-md bg-white px-2.5 py-1.5 text-sm font-semibold text-gray-900 shadow-sm ring-1 ring-inset ring-gray-300 hover:bg-gray-50 sm:block"
            on:click =move|_| on_delete(id.clone())
          >
            "delete"
          </a>
        </div>
      </li>
    }
}

#[component]
fn DisplayNumber(number: Signal<Option<Result<usize, crate::api::Error>>>) -> impl IntoView {
    let memorized_number = create_memo(move |_| number.get());

    #[allow(unused_braces)]
    move || match memorized_number.get() {
        Some(Ok(nr)) => view! { { nr.to_string() } }.into_view(),
        Some(Err(_)) => {
            // TODO: use an appropriate icon
            view! { "- API error -" }.into_view()
        }
        None => {
            // TODO: use spinner icon
            view! { "-" }.into_view()
        }
    }
}
