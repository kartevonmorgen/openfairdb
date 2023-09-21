use leptos::*;
use leptos_router::*;
use time::{format_description::FormatItem, macros::format_description, Duration, OffsetDateTime};

use ofdb_frontend_api::{EventQuery, PublicApi, UserApi};

use crate::Page;

const DATE_TIME_FORMAT: &[FormatItem] = format_description!("[year]-[month]-[day] [hour]:[minute]");

#[component]
pub fn Events(public_api: PublicApi) -> impl IntoView {
    let mut query = EventQuery::default();
    let start_min = OffsetDateTime::now_utc() - Duration::hours(12);
    query.start_min = Some(start_min.unix_timestamp());

    // -- actions -- //
    let fetch_events = create_action(move |_| {
        let query = query.clone();
        async move { public_api.events(&query).await }
    });

    fetch_events.dispatch(());

    view! {
     <section>
        <div class="container p-6 mx-auto">
        { move ||
          match fetch_events.value().get() {
            Some(Ok(events)) => {
              if events.is_empty() {
                view!{
                  <p>
                    "No events could be found."
                  </p>
                }.into_view()
              } else {
                view!{
                  <ul role="list" class="divide-y divide-gray-100">
                    <For
                      each = move || events.clone()
                      key = |event| event.id.clone() // TODO: can we avoid this clone?
                      view = move |event| view!{ <EventListItem event /> }
                    />
                  </ul>
                }.into_view()
              }
            }
            None => view!{ <p>"The events are loaded ..."</p> }.into_view(),
            Some(Err(_)) => view!{ <p>"An error occurred while loading the events."</p> }.into_view()
          }
      }
      </div>
     </section>
    }
}

#[component]
fn EventListItem(event: ofdb_boundary::Event) -> impl IntoView {
    let start = OffsetDateTime::try_from(event.start).expect("valid date time");
    view! {
      <li class="relative flex justify-between gap-x-6 py-5">
         <div class="flex min-w-0 gap-x-4">
           {
              event.image_url.map(|url| view!{
                <img class="h-12 w-12 flex-none rounded-full bg-gray-50" src={ url } alt="" />
              })
           }
           <div class="min-w-0 flex-auto">
             <p class="text-sm font-semibold leading-6 text-gray-900">
               <a href=format!("{}/{}", Page::Events.path(), event.id)>
                 <span class="absolute inset-x-0 -top-px bottom-0"></span>
                 { event.title }
               </a>
             </p>
             <p class="mt-1 flex text-xs leading-5 text-gray-500">
               <time>{ start.format(DATE_TIME_FORMAT) }</time>
             </p>
           </div>
         </div>
         <div class="flex shrink-0 items-center gap-x-4">
           <div class="hidden sm:flex sm:flex-col sm:items-end">
             <p class="text-sm leading-6 text-gray-900">{ event.city }</p>
             <p class="mt-1 text-xs leading-5 text-gray-500">{ event.organizer }</p>
           </div>
           <svg class="h-5 w-5 flex-none text-gray-400" viewBox="0 0 20 20" fill="currentColor" aria-hidden="true">
             <path fill-rule="evenodd" d="M7.21 14.77a.75.75 0 01.02-1.06L11.168 10 7.23 6.29a.75.75 0 111.04-1.08l4.5 4.25a.75.75 0 010 1.08l-4.5 4.25a.75.75 0 01-1.06-.02z" clip-rule="evenodd" />
           </svg>
         </div>
      </li>
    }
}

#[component]
pub fn Event(public_api: PublicApi, user_api: Signal<Option<UserApi>>) -> impl IntoView {
    // -- signals -- //

    let params = use_params_map();

    // -- actions -- //

    let fetch_event = create_action(move |id: &String| {
        let id = id.to_owned();
        async move { public_api.event(&id).await }
    });

    // -- effects -- //

    create_effect(move |_| {
        if let Some(id) = params.with(|p| p.get("id").cloned()) {
            fetch_event.dispatch(id);
        }
    });

    move || match fetch_event.value().get() {
        Some(Ok(event)) => view! { <EventProfile event user_api /> }.into_view(),
        None => view! { <p>"The event is loaded ..."</p> }.into_view(),
        Some(Err(_)) => view! { <p>"An error occurred while loading the event."</p> }.into_view(),
    }
}

#[component]
fn EventProfile(event: ofdb_boundary::Event, user_api: Signal<Option<UserApi>>) -> impl IntoView {
    let ofdb_boundary::Event {
        id,
        title,
        start,
        description,
        image_url,
        street,
        city,
        zip,
        organizer,
        telephone,
        email,
        homepage,
        tags,
        ..
    } = event;

    let archive_event = create_action(move |_| {
        let id = id.clone();
        let navigate = use_navigate();
        async move {
            let Some(user_api) = user_api.get() else {
                unreachable!();
            };
            match user_api.archive_events(&[&id]).await {
                Ok(_) => {
                    log::info!("Successfully archived event {id}");
                    navigate(Page::Events.path(), Default::default());
                }
                Err(err) => {
                    log::error!("Unable to archive event {id}: {err}");
                }
            }
        }
    });

    let start = OffsetDateTime::try_from(start).expect("valid date time");

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
              <dt class="font-medium text-gray-900">"Date & Time"</dt>
              <dd class="mt-2 text-sm text-gray-500">
                <time>{ start.format(DATE_TIME_FORMAT) }</time>
              </dd>
            </div>
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
              { organizer }
              <br />
              { telephone }
              <br />
              { email }
              </dd>
            </div>
            <div class="border-t border-gray-200 pt-4">
              <dt class="font-medium text-gray-900">"Tags"</dt>
              <dd class="mt-2 text-sm text-gray-500">
              { tags.join(" ") }
              </dd>
            </div>
            <div class="border-t border-gray-200 pt-4">
              <dt class="font-medium text-gray-900">"Web"</dt>
              <dd class="mt-2 text-sm text-gray-500">{ homepage }</dd>
            </div>
            { move || user_api.get().map(|_|
                view! {
                  <div class="border-t border-gray-200 pt-4">
                    <dt class="font-medium text-gray-900">"Actions"</dt>
                    <dd class="mt-2 text-sm text-gray-500">
                      <button
                        type="button"
                        class="rounded bg-indigo-600 px-2 py-1 text-xs font-semibold text-white shadow-sm hover:bg-indigo-500 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-indigo-600"
                        on:click=move|_|archive_event.dispatch(()) >
                        "Archive event"
                      </button>
                    </dd>
                  </div>
                }
              )
            }
          </dl>
        </div>
      </div>
    }
}
