use leptos::*;
use wasm_bindgen::{UnwrapThrowExt, JsCast, JsValue};
use web_sys::{Blob, HtmlElement, Url};

use ofdb_boundary::{MapBbox, MapPoint, PlaceSearchResult, SearchResponse};

use crate::{
    api::{PublicApi, UserApi},
    components::*,
};

#[component]
pub fn Home(
    public_api: PublicApi,
    user_api: Signal<Option<UserApi>>,
    bbox: RwSignal<MapBbox>,
) -> impl IntoView {
    // -- signals -- //

    let search_response = RwSignal::new(None::<SearchResponse>);
    let search_error = RwSignal::new(None::<String>);
    let search_term = RwSignal::new(String::new());
    let center = RwSignal::new(center_of_bbox(bbox.get_untracked()));

    let places: Signal<Vec<_>> = Signal::derive(move || {
        search_response.with(|r| {
            r.as_ref()
                .map(|res| res.visible.clone())
                .unwrap_or_default()
        })
    });

    // -- actions -- //

    let search_action = Action::new(move |_: &()| {
        let bbox = bbox.get_untracked();
        let text = search_term.get_untracked();
        async move {
            let result = public_api.search(&text, &bbox).await;
            match result {
                Ok(res) => {
                    search_response.update(|v| *v = Some(res));
                    search_error.update(|e| *e = None);
                }
                Err(err) => {
                    search_error.update(|e| *e = Some(format!("{err:?}")));
                }
            }
        }
    });

    // -- callbacks -- //

    let on_bbox_changed = Callback::new(move |new_bbox| {
        bbox.set(new_bbox);
        search_action.dispatch(());
    });

    let search = Callback::new(move |txt: String| {
        search_term.set(txt);
        search_action.dispatch(());
    });

    // -- view -- //

    view! {
      <div class="h-full w-full relative">
        <Map center = center.into() on_bbox_changed places = places.into() />
        <div class="absolute z-[500] top-0 left-0 m-4 bg-red w-96">
          <SearchInput search />
          { move || search_error.get().map(|err|
            view! {
              <div class="mt-2 text-red-600">
                { err }
              </div>
            })
          }

          <div class="rounded p-2 mt-2 text-sm text-gray-500 bg-white bg-opacity-70 text-right">
            { move || search_response.with(|res|res.as_ref().map(|r|view! { "Results: " { r.visible.len() } }))  }
          </div>

          <SearchResults search_response />
        </div>
        { move || {
            user_api.get().map(|api|{
              view! {
                <div class="absolute z-[500] top-0 right-0 m-4">
                  <button
                    class="rounded p-2 bg-kvm-pink"
                    on:click = move |_| {
                        let export_bbox = bbox.get();
                        spawn_local({let api = api.clone(); async move {
                           match api.export_csv(&export_bbox, None).await {
                              Ok(csv) => {
                                  trigger_download(&csv, "export.csv");
                              }
                              Err(err) => {
                                log::warn!("{err}");
                              }
                          }
                       }});
                    }
                  >"CSV export"
                  </button>
                </div>
              }
            })
        }}
      </div>
    }
}

#[component]
pub fn SearchInput(search: Callback<String, ()>) -> impl IntoView {
    view! {
      <input
        type="search"
        placeholder="Search..."
        class="p-2 bg-white border rounded-lg shadow-lg w-full"
        on:keyup = move |ev| {
          ev.stop_propagation();
          let target = event_target::<web_sys::HtmlInputElement>(&ev);
          match &*ev.key() {
            "Enter" => {
              let value = event_target_value(&ev);
              let value = value.trim();
              search.call(value.to_string());
            }
            "Escape" => {
              target.set_value("");
            }
            _=> { /* nothing to to */ }
          }
        }
      />
    }
}

const MAX_ITEMS: usize = 50;

#[component]
pub fn SearchResults(search_response: RwSignal<Option<SearchResponse>>) -> impl IntoView {
    view! {
        {move ||
            search_response.get().and_then(|res|{
              if res.visible.is_empty() {
                  return None;
              }
              let items = res.visible.iter().take(MAX_ITEMS).map(|place|view! {
                <li><PlaceSearchResultItem place /></li>
              }).collect::<Vec<_>>();
              Some(view!{
                <div class="mt-2 p-2 bg-white border rounded-lg shadow-lg max-h-48 overflow-y-auto">
                  <ul>{ items }</ul>
                </div>
              })
            })
        }
    }
}

const MAX_DESCRIPTION_LEN: usize = 40;

#[component]
fn PlaceSearchResultItem<'a>(place: &'a PlaceSearchResult) -> impl IntoView {
    view! {
      <div class="font-bold hover:text-gray-600 pointer-cursor">
        <a>
          { place.title.clone() }
        </a>
      </div>
      <div class="text-gray-600">{ shorten_description(&place.description, MAX_DESCRIPTION_LEN) }</div>
    }
}

fn shorten_description(description: &str, max_length: usize) -> String {
    if description.chars().count() > max_length {
        let shortened: String = description.chars().take(max_length).collect();
        format!("{}...", shortened)
    } else {
        description.to_string()
    }
}


fn trigger_download(csv_data: &str, filename: &str) {
    let window = web_sys::window().unwrap_throw();
    let document = window.document().unwrap_throw();

    let blob = Blob::new_with_str_sequence(&js_sys::Array::of1(&JsValue::from_str(csv_data)))
        .unwrap_throw();

    let url = Url::create_object_url_with_blob(&blob).unwrap_throw();

    let a = document.create_element("a").unwrap_throw();
    let a = a.dyn_into::<HtmlElement>().unwrap_throw();
    a.set_attribute("href", &url).unwrap_throw();
    a.set_attribute("download", filename).unwrap_throw();
    a.style().set_property("display", "none").unwrap_throw();

    document
        .body()
        .unwrap_throw()
        .append_child(&a)
        .unwrap_throw();
    a.click();
    document
        .body()
        .unwrap_throw()
        .remove_child(&a)
        .unwrap_throw();

    Url::revoke_object_url(&url).unwrap_throw();
}

fn center_of_bbox(bbox: MapBbox) -> MapPoint {
        let MapBbox { sw, ne } = bbox;
        let lat = (sw.lat + ne.lat) / 2.0;
        let lng = (sw.lng + ne.lng) / 2.0;
        MapPoint { lat, lng }
}
