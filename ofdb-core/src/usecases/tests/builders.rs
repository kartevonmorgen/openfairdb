pub use self::new_place_builder::*;
pub use ofdb_entities::builders::Builder;

pub mod new_place_builder {

    use super::*;
    use crate::usecases::NewPlace;
    use ofdb_entities::geo::*;

    #[derive(Debug)]
    pub struct NewPlaceBuild {
        new_place: NewPlace,
    }

    impl NewPlaceBuild {
        pub fn title(mut self, title: &str) -> Self {
            self.new_place.title = title.into();
            self
        }
        pub fn description(mut self, desc: &str) -> Self {
            self.new_place.description = desc.into();
            self
        }
        pub fn pos(mut self, pos: MapPoint) -> Self {
            self.new_place.lat = pos.lat().to_deg();
            self.new_place.lng = pos.lng().to_deg();
            self
        }
        pub fn tags(mut self, tags: Vec<impl Into<String>>) -> Self {
            self.new_place.tags = tags.into_iter().map(|x| x.into()).collect();
            self
        }
        pub fn license(mut self, license: &str) -> Self {
            self.new_place.license = license.into();
            self
        }

        // TODO: add other fields

        pub fn finish(self) -> NewPlace {
            self.new_place
        }
    }

    impl Builder for NewPlace {
        type Build = NewPlaceBuild;
        fn build() -> Self::Build {
            Self::Build {
                new_place: NewPlace {
                    title: "".into(),
                    description: "".into(),
                    lat: 0.0,
                    lng: 0.0,
                    street: None,
                    zip: None,
                    city: None,
                    country: None,
                    state: None,
                    contact_name: None,
                    email: None,
                    telephone: None,
                    homepage: None,
                    opening_hours: None,
                    founded_on: None,
                    categories: vec![],
                    tags: vec![],
                    license: "".into(),
                    image_url: None,
                    image_link_url: None,
                    custom_links: vec![],
                },
            }
        }
    }
}
