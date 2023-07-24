use leptos::*;
use ofdb_boundary::{MapBbox, PlaceSearchResult, SearchResponse};

use crate::{api::PublicApi, pages::Page};

#[component]
pub fn PlaceSearch(cx: Scope, api: PublicApi, bbox: ReadSignal<MapBbox>) -> impl IntoView {
    let (search_response, set_search_response) = create_signal(cx, None::<SearchResponse>);
    let (search_error, set_search_error) = create_signal(cx, None::<String>);

    let search_action = create_action(cx, move |text: &String| {
        //let bbox = DEFAULT_BBOX;
        let bbox = bbox.get();
        let text = text.clone();
        async move {
            let result = api.search(&text, &bbox).await;
            match result {
                Ok(res) => {
                    set_search_response.update(|v| *v = Some(res));
                    set_search_error.update(|e| *e = None);
                }
                Err(err) => {
                    set_search_error.update(|e| *e = Some(format!("{err:?}")));
                }
            }
        }
    });

    view! { cx,
      <div class="flex items-center justify-center">
        { search_error.get().map(|err| view!{ cx, <p>{ err }</p> }) }
        <input
          type="search"
          class="w-full max-w-md py-3 px-4 bg-gray-50 text-gray-700 outline-none mb-4 rounded"
          placeholder="search"
          on:keyup = move |ev| {
            ev.stop_propagation();
            let target = event_target::<web_sys::HtmlInputElement>(&ev);
            match &*ev.key() {
              "Enter" => {
                let value = event_target_value(&ev);
                let value = value.trim();
                search_action.dispatch(value.to_string());
              }
              "Escape" => {
                target.set_value("");
              }
              _=> { /* nothing to to */ }
            }
          }
        />
      </div>
      { move || search_response.get().is_some().then(||{
        let count = search_response.get().map(|r|r.visible.len()).unwrap_or_default();
        view!{ cx,
          <div class="flex justify-start mb-4">
            <p class="py-2 text-gray-500 border-b border-gray-300">
              "Found "
              <span class="font-bold">{ count }</span>
              " results"
            </p>
          </div>
        }})
      }
      { move || search_response.get().is_some().then(|| view!{ cx,
          <ul>
            <For
              each = move || search_response.get().map(|res|res.visible).unwrap_or_default()
              key = |place| place.id.clone() // TODO: can we avoid this clone?
              view = move |cx, place| {
                view! { cx, <li class="mb-3"><PlaceSearchResultItem place /></li> }
              }
            />
          </ul>
        })
      }
    }
}

#[component]
fn PlaceSearchResultItem(cx: Scope, place: PlaceSearchResult) -> impl IntoView {
    view! { cx,
      <div class="font-bold text-lg hover:text-gray-600">
        <a href=format!("{}/{}", Page::Entries.path(), place.id)>
          { place.title }
        </a>
      </div>
      <div class="text-gray-600">{ place.description }</div>
      <ul>
        <For
          each = move || place.tags.clone() // TODO: can we avoid this clone?
          key = |tag| tag.clone() // TODO: can we avoid this clone?
          view = move |cx, tag| {
             view!{ cx, <span class="text-xs bg-gray-100 text-gray-500 rounded mr-1 p-1">{ tag }</span> }
          }
        />
      </ul>
    }
}
