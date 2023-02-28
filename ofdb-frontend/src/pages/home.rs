use leptos::*;

use ofdb_boundary::MapBbox;

use crate::{api::PublicApi, components::*};

#[component]
pub fn Home<A>(cx: Scope, api: A, bbox: ReadSignal<MapBbox>) -> impl IntoView
where
    A: PublicApi + Clone + 'static,
{
    view! { cx,
      <section>
        <div class="container p-6 mx-auto">
          <PlaceSearch api bbox />
        </div>
      </section>
    }
}
