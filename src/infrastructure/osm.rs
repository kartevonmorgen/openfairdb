use entities::*;
use business::db::Db;
use std::io::{Error, ErrorKind};
use std::io::prelude::*;
use std::fs::File;
use std::result;
use std::collections::HashMap;
use serde_json;
use super::web::sqlite::create_connection_pool;
use chrono::prelude::*;
use uuid::Uuid;
use infrastructure::error::AppError;

type Result<T> = result::Result<T, AppError>;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct OsmQueryResult {
    elements: Vec<OsmEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct OsmEntry {
    id: u64,
    lat: f64,
    lon: f64,
    tags: HashMap<String, String>,
}

pub fn import_from_osm_file(db_url: &str, file_name: &str) -> Result<()> {
    let mut file = File::open(file_name)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    let osm_entries = parse_query_result(&contents)?;
    debug!("parsed {} entries", osm_entries.len());
    let pool = create_connection_pool(db_url).unwrap();
    let db = &mut *pool.get().unwrap();
    let ofdb_entries = db.all_entries()?;
    let old_osm_entries: Vec<_> = ofdb_entries
        .into_iter()
        .filter(|e| e.osm_node.is_some())
        .collect();
    let new_osm_entries: Vec<_> = osm_entries
        .into_iter()
        .filter(|e| e.tags.get("name").is_some())
        .filter(|new| {
            !old_osm_entries
                .iter()
                .any(|old| old.osm_node == Some(new.id))
        })
        .collect();
    debug!("mapping new osm entries ...");
    let mapped_entries: Vec<_> = new_osm_entries
        .into_iter()
        .filter_map(|osm| match map_osm_to_ofdb_entry(&osm) {
            Ok(x) => Some(x),
            Err(err) => {
                warn!("Could not map osm entry: {}", err);
                None
            }
        })
        .collect();

    debug!("importing nodes ...");
    db.import_multiple_entries(mapped_entries.as_slice())?;
    info!("Imported {} new entries from OSM", mapped_entries.len());
    Ok(())
}

fn parse_query_result(data: &str) -> result::Result<Vec<OsmEntry>, serde_json::error::Error> {
    let r: OsmQueryResult = serde_json::from_str(data)?;
    Ok(r.elements)
}

fn map_osm_tags(osm_tags: &HashMap<String, String>) -> Vec<Tag> {
    let mut tags = vec![];
    let mut tag_map = HashMap::new();

    // TODO: use config file
    tag_map.insert("diet:vegan", "vegan");
    tag_map.insert("diet:vegetarian", "vegetarisch");
    tag_map.insert("diet:egg_free", "eifrei");
    tag_map.insert("diet:lactose_free", "laktosefrei");
    tag_map.insert("diet:soy_free", "soyafrei");
    tag_map.insert("diet:dairy_free", "milchfrei");
    tag_map.insert("diet:gluten_free", "glutenfrei");
    tag_map.insert("organic", "bio");

    for (k, v) in tag_map {
        if let Some(_val) = osm_tags.get(k) {
            tags.push(Tag { id: v.into() });
        }
    }
    tags
}

fn map_osm_to_ofdb_entry(osm: &OsmEntry) -> Result<Entry> {
    let title = osm.tags
        .get("name")
        .ok_or_else(|| Error::new(ErrorKind::Other, "Tag 'name' not found"))?
        .clone();

    let description = title.clone();

    let id = Uuid::new_v4().simple().to_string();

    let osm_node = Some(osm.id);

    let lat = osm.lat;
    let lng = osm.lon;

    let version = 0;
    let created = Utc::now().timestamp() as u64;
    let house_nr = osm.tags.get("addr:housenumber").cloned();
    let street = osm.tags.get("addr:street").cloned();
    let zip = osm.tags.get("addr:postcode").cloned();
    let city = osm.tags.get("addr:city").cloned();
    let country = osm.tags.get("addr:country").cloned();
    let email = None;
    let telephone = osm.tags.get("phone").cloned();
    let homepage = osm.tags.get("website").cloned();
    let categories = vec![];
    let license = Some("ODbL-1.0".into());

    let street = street.map(|s| {
        if let Some(nr) = house_nr {
            format!("{} {}", s, nr)
        } else {
            s
        }
    });

    let tags = map_osm_tags(&osm.tags).into_iter().map(|t| t.id).collect();

    Ok(Entry {
        id,
        osm_node,
        created,
        version,
        title,
        description,
        lat,
        lng,
        street,
        zip,
        city,
        country,
        email,
        telephone,
        homepage,
        categories,
        tags,
        license,
    })
}

#[test]
fn test_parse_query_result() {
    let result = r#"{
      "version": 0.6,
      "generator": "Overpass API 0.7.54.12 054bb0bb",
      "osm3s": {
        "timestamp_osm_base": "2017-11-22T22:20:03Z",
        "copyright": "The data included in this document is from www.openstreetmap.org. The data is made available under ODbL."
      },
      "elements": [

    {
      "type": "node",
      "id": 20962297,
      "lat": 47.0598329,
      "lon": 15.4701174,
      "tags": {
        "addr:city": "Graz",
        "addr:country": "AT",
        "addr:housenumber": "107a",
        "addr:postcode": "8042",
        "addr:street": "Plüddemanngasse",
        "diet:dairy_free": "yes",
        "diet:egg_free": "yes",
        "diet:gluten_free": "yes",
        "diet:lactose_free": "yes",
        "diet:soy_free": "yes",
        "diet:vegan": "yes",
        "diet:vegetarian": "yes",
        "name": "denn's Biomarkt",
        "opening_hours": "Mo-Fr 08:00-19:00; Sa 08:00-18:00",
        "organic": "only",
        "phone": "+43 316-422677",
        "shop": "supermarket",
        "start_date": "2016-04-21",
        "website": "http://www.denns-biomarkt.at/",
        "wheelchair": "limited"
      }
    }]
    }"#;
    let x = parse_query_result(result).unwrap();
    assert_eq!(x.len(), 1);
    assert_eq!(x[0].id, 20962297);
    assert_eq!(x[0].tags.get("addr:city").unwrap(), "Graz");
}

#[test]
fn test_from_osm_for_entry() {
    let mut tags = HashMap::new();

    tags.insert("addr:street".into(), "Plüddemanngasse".into());
    tags.insert("addr:housenumber".into(), "107a".into());
    tags.insert("addr:postcode".into(), "8042".into());
    tags.insert("addr:city".into(), "Graz".into());
    tags.insert("addr:country".into(), "AT".into());
    tags.insert("name".into(), "denn's Biomarkt".into());
    tags.insert("phone".into(), "+43 316-422677".into());
    tags.insert("website".into(), "http://www.denns-biomarkt.at/".into());

    tags.insert("diet:dairy_free".into(), "yes".into());
    tags.insert("diet:egg_free".into(), "yes".into());
    tags.insert("diet:gluten_free".into(), "yes".into());
    tags.insert("diet:lactose_free".into(), "yes".into());
    tags.insert("diet:soy_free".into(), "yes".into());
    tags.insert("diet:vegan".into(), "yes".into());
    tags.insert("diet:vegetarian".into(), "yes".into());
    tags.insert("organic".into(), "only".into());

    let osm = OsmEntry {
        id: 12123,
        lat: 48.0,
        lon: 10.0,
        tags,
    };

    let e = map_osm_to_ofdb_entry(&osm).unwrap();

    assert_eq!(e.lat, 48.0);
    assert_eq!(e.lng, 10.0);
    assert_eq!(e.version, 0);
    assert_eq!(e.osm_node, Some(12123));
    assert_eq!(e.title, "denn's Biomarkt");
    assert_eq!(e.description, "denn's Biomarkt");
    assert_eq!(e.city, Some("Graz".into()));
    assert_eq!(e.zip, Some("8042".into()));
    assert_eq!(e.country, Some("AT".into()));
    assert_eq!(e.street, Some("Plüddemanngasse 107a".into()));
    assert_eq!(e.homepage, Some("http://www.denns-biomarkt.at/".into()));
    assert_eq!(e.telephone, Some("+43 316-422677".into()));
    assert_eq!(e.license, Some("ODbL-1.0".into()));

    assert!(e.tags.iter().any(|id| id == "vegan"));
    assert!(e.tags.iter().any(|id| id == "vegetarisch"));
    assert!(e.tags.iter().any(|id| id == "bio"));
    assert!(e.tags.iter().any(|id| id == "eifrei"));
    assert!(e.tags.iter().any(|id| id == "laktosefrei"));
    assert!(e.tags.iter().any(|id| id == "soyafrei"));
    assert!(e.tags.iter().any(|id| id == "milchfrei"));
    assert!(e.tags.iter().any(|id| id == "glutenfrei"));
}
