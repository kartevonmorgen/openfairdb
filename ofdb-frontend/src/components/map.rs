use leaflet::LatLng;
use leptos::*;
use leptos_leaflet::{position, MapContainer, MapEvents, Marker, Popup, Position, TileLayer};

use ofdb_boundary::{MapBbox, MapPoint, PlaceSearchResult};

const TILE_LAYER_URL: &str = "https://tile.openstreetmap.org/{z}/{x}/{y}.png";
const MAP_ATTRIBUTION: &str =
    "&copy; <a href=\"https://www.openstreetmap.org/copyright\">OpenStreetMap</a> contributors";

#[component]
pub fn Map(
    center: Signal<MapPoint>,
    on_bbox_changed: Callback<MapBbox, ()>,
    places: Signal<Vec<PlaceSearchResult>>,
) -> impl IntoView {
    let events = MapEvents::new();
    let map = RwSignal::<Option<leaflet::Map>>::new(None);

    let update_bbox = move || {
        let Some(map) = map.get_untracked() else {
            log::warn!("No leaflet map found");
            return;
        };
        let bounds = map.get_bounds();
        let bbox = MapBbox {
            sw: MapPoint {
                lat: bounds.get_south_west().lat(),
                lng: bounds.get_south_west().lng(),
            },
            ne: MapPoint {
                lat: bounds.get_north_east().lat(),
                lng: bounds.get_north_east().lng(),
            },
        };
        on_bbox_changed.call(bbox);
    };

    events.clone().move_end(move |_| {
        update_bbox();
    });

    Effect::new(move |_| {
        log::debug!("Leaflet map changed");
        if map.get().is_some() {
            update_bbox();
        };
    });

    Effect::new(move |_| {
        let Some(map) = map.get_untracked() else {
            log::warn!("No leaflet map found");
            return;
        };
        let MapPoint { lat, lng } = center.get();
        let zoom = map.get_zoom();
        let p = LatLng::new(lat, lng);
        map.set_view(&p, zoom);
    });

    let MapPoint { lat, lng } = center.get_untracked();
    let center = Position::new(lat, lng);

    view! {
      <MapContainer
        class="h-full"
        center
        zoom=13.0
        zoom_control=false
        map=map.write_only()
        set_view=true
        events
      >
        <TileLayer url=TILE_LAYER_URL attribution=MAP_ATTRIBUTION />
        <For
          each=move || places.get()
          key=|place| place.id.clone()
          let:place
        >
          <Marker position=position!(place.lat, place.lng)>
            <Popup>
              <strong>{place.title}</strong>
            </Popup>
          </Marker>
        </For>
      </MapContainer>
    }
}
