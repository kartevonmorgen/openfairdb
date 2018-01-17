use entities::*;
use uuid::Uuid;

pub trait EntryBuilder {
    fn build() -> EntryBuild;
}

pub struct EntryBuild {
    entry: Entry,
}

impl EntryBuild {
    pub fn id(mut self, id: &str) -> Self {
        self.entry.id = id.into();
        self
    }
    pub fn version(mut self, v: u64) -> Self {
        self.entry.version = v;
        self
    }
    pub fn title(mut self, title: &str) -> Self {
        self.entry.title = title.into();
        self
    }
    pub fn description(mut self, desc: &str) -> Self {
        self.entry.description = desc.into();
        self
    }
    pub fn lat(mut self, lat: f64) -> Self {
        self.entry.lat = lat;
        self
    }
    pub fn lng(mut self, lng: f64) -> Self {
        self.entry.lng = lng;
        self
    }
    pub fn categories(mut self, cats: Vec<&str>) -> Self {
        self.entry.categories = cats.into_iter().map(|x| x.into()).collect();
        self
    }
    pub fn tags(mut self, tags: Vec<&str>) -> Self {
        self.entry.tags = tags.into_iter().map(|x| x.into()).collect();
        self
    }
    pub fn finish(self) -> Entry {
        self.entry
    }
}

impl EntryBuilder for Entry {
    fn build() -> EntryBuild {
        EntryBuild {
            entry: Entry::default(),
        }
    }
}

impl Default for Entry {
    fn default() -> Entry {
        #[cfg_attr(rustfmt, rustfmt_skip)]
        Entry{
            id          : Uuid::new_v4().simple().to_string(),
            osm_node    : None,
            created     : 0,
            version     : 0,
            title       : "".into(),
            description : "".into(),
            lat         : 0.0,
            lng         : 0.0,
            street      : None,
            zip         : None,
            city        : None,
            country     : None,
            email       : None,
            telephone   : None,
            homepage    : None,
            categories  : vec![],
            tags        : vec![],
            license     : None,
        }
    }
}
