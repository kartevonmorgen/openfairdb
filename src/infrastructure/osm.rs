use entities::*;
use std::io::{Error, ErrorKind, Result};
use std::io::prelude::*;
use std::fs::File;
use std::result;
use std::collections::HashMap;
use serde_json;
use super::web::sqlite::create_connection_pool;

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
    let res = parse_query_result(&contents)?;
    debug!("parsed {} entries", res.len());
    let pool = create_connection_pool(db_url).unwrap();
    for osm in res.iter() {
        //TODO: import
    }
    Ok(())
}

fn parse_query_result(data: &str) -> result::Result<Vec<OsmEntry>, serde_json::error::Error> {
    let r: OsmQueryResult = serde_json::from_str(data)?;
    Ok(r.elements)
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
