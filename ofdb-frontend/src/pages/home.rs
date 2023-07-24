use leptos::*;

use ofdb_boundary::MapBbox;

use crate::{api::PublicApi, components::*};

#[component]
pub fn Home(cx: Scope, api: PublicApi, bbox: ReadSignal<MapBbox>) -> impl IntoView {
    view! { cx,
      <section>
        <div class="container p-6 mx-auto">
          <PlaceSearch api bbox />
        </div>
      </section>
    }
}
