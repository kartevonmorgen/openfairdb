// Copyright (c) 2015 - 2016 Markus Kohlhase <mail@markus-kohlhase.de>

use adapters::json::{Entry, Category};
use rusted_cypher::GraphClient;
use uuid::Uuid;
use business::error::StoreError;

pub trait Store {
    type Id;
    type Connection;
    type Error;

    fn get(&Self::Connection, Self::Id) -> Result<Self, Self::Error> where Self: Sized;
    fn all(&Self::Connection) -> Result<Vec<Self>, Self::Error> where Self: Sized;
    fn save(&self, &Self::Connection) -> Result<Self, Self::Error> where Self: Sized;
}

impl Store for Entry {
    type Id = String;
    type Connection = GraphClient;
    type Error = StoreError;

    fn get(graph: &Self::Connection, id: Self::Id) -> Result<Entry, StoreError> {
        let result = try!(graph.cypher().exec(cypher_stmt!(
      "MATCH (e:Entry)<--(s:EntryState) WHERE e.id = {id}
       WITH max(s.version) as version
       MATCH (e:Entry)<--(s:EntryState)
       WHERE e.id = {id} AND s.version = version
       WITH e, s
       MATCH s<-[:BELONGS_TO]-(c:Category)
       WITH e, s, collect(DISTINCT c.id) as categories
       RETURN {
         id          : e.id,
         created     : s.created,
         version     : s.version,
         title       : s.title,
         description : s.description,
         lat         : s.lat,
         lng         : s.lng,
         street      : s.street,
         zip         : s.zip,
         city        : s.city,
         country     : s.country,
         email       : s.email,
         telephone   : s.telephone,
         homepage    : s.homepage,
         categories  : categories,
         license     : s.license
       } AS e
       ORDER BY e.created DESC", {"id" => &id})));
        result.rows()
            .next()
            .ok_or(StoreError::NotFound)
            .and_then(|r| r.get::<Entry>("e").map_err(StoreError::Graph))

    }

    fn all(graph: &Self::Connection) -> Result<Vec<Entry>, StoreError> {
        let result = try!(graph.cypher().exec(
     "MATCH (e:Entry)<--(x:EntryState)
       WITH distinct e, max(x.created) as max
       MATCH e<--(s:EntryState)
       WHERE s.created = max
       WITH e,s
       MATCH e<-[:BELONGS_TO]-s
       OPTIONAL MATCH (s)<-[:BELONGS_TO]-(c:Category)
       WITH e, s, collect(DISTINCT c.id) as categories
       RETURN {
         id          : e.id,
         created     : s.created,
         version     : s.version,
         title       : s.title,
         description : s.description,
         lat         : s.lat,
         lng         : s.lng,
         street      : s.street,
         zip         : s.zip,
         city        : s.city,
         country     : s.country,
         email       : s.email,
         telephone   : s.telephone,
         homepage    : s.homepage,
         categories  : categories,
         license     : s.license
       } AS e
       ORDER BY e.created DESC"));
        Ok(result.rows()
            .filter_map(|r| r.get::<Entry>("e").ok())
            .collect::<Vec<Entry>>())
    }

    fn save(&self, graph: &Self::Connection) -> Result<Entry, StoreError> {
        match self.id {
            None => create_entry(self, graph),
            Some(_) => update_entry(self, graph),
        }
    }
}

fn create_entry(e: &Entry, graph: &GraphClient) -> Result<Entry, StoreError> {

    let id = match e.id {
        None => Uuid::new_v4().to_simple_string(),
        Some(_) => return Err(StoreError::InvalidId),
    };

    if let Some(v) = e.version {
        if !(v == 0 || v == 1) {
            return Err(StoreError::InvalidVersion);
        }
    }

    let result = try!(graph.cypher().exec(cypher_stmt!(
    "MATCH (c:Category)
     WHERE c.id in {categories}
     WITH
        collect(DISTINCT c)    AS cats,
        collect(DISTINCT c.id) AS cat_ids,
        count(c)               AS cnt
     WHERE cnt > 0
     CREATE (e:Entry {id:{id}})
     MERGE e<-[:BELONGS_TO]-(s:EntryState {
       created : timestamp(),
       version : 1
     })
     SET s.title       = {title},
         s.description = {description},
         s.lat         = {lat},
         s.lng         = {lng},
         s.street      = {street},
         s.zip         = {zip},
         s.city        = {city},
         s.country     = {country},
         s.email       = {email},
         s.telephone   = {telephone},
         s.homepage    = {homepage},
         s.license     = {license}
     FOREACH (c IN cats |
       MERGE c-[:BELONGS_TO]->s
     )
     RETURN {
       id          : e.id,
       created     : s.created,
       version     : s.version,
       title       : s.title,
       description : s.description,
       lat         : s.lat,
       lng         : s.lng,
       street      : s.street,
       zip         : s.zip,
       city        : s.city,
       country     : s.country,
       email       : s.email,
       telephone   : s.telephone,
       homepage    : s.homepage,
       categories  : cat_ids,
       homepage    : s.license
     } AS new",
   {
     "id"          => &id,
     "title"       => &e.title,
     "description" => &e.description,
     "lat"         => &e.lat,
     "lng"         => &e.lng,
     "street"      => &e.street,
     "zip"         => &e.zip,
     "city"        => &e.city,
     "country"     => &e.country,
     "email"       => &e.email,
     "telephone"   => &e.telephone,
     "homepage"    => &e.homepage,
     "license"     => &e.license,
     "categories"  => &e.categories.clone().unwrap_or(vec!())
    })));
    result.rows()
        .next()
        .ok_or(StoreError::Save)
        .and_then(|r| r.get::<Entry>("new").map_err(StoreError::Graph))
}

fn update_entry(e: &Entry, graph: &GraphClient) -> Result<Entry, StoreError> {
    let id = try!(e.id.clone().ok_or(StoreError::InvalidId));
    let version = try!(e.version.ok_or(StoreError::InvalidVersion));
    let result = try!(graph.cypher().exec(cypher_stmt!(
    "MATCH (e:Entry)<--(s:EntryState) WHERE e.id = {id}
     WITH max(s.version) as v
     MATCH (e:Entry)<--(old:EntryState)
     WHERE e.id = {id} AND old.version = v AND old.version + 1 = {version}
     WITH e
     MATCH (c:Category)
     WHERE c.id in {categories}
     WITH e, collect(DISTINCT c) AS cats, count(c) AS cnt
     WHERE cnt > 0
     MERGE e<-[:BELONGS_TO]-(s:EntryState {
       created : timestamp(),
       version : {version}
     })
     SET s.title       = {title},
         s.description = {description},
         s.lat         = {lat},
         s.lng         = {lng},
         s.street      = {street},
         s.zip         = {zip},
         s.city        = {city},
         s.country     = {country},
         s.email       = {email},
         s.telephone   = {telephone},
         s.homepage    = {homepage},
         s.license     = {license}
     FOREACH (c IN cats |
       MERGE (c)-[:BELONGS_TO]->s
     )
     WITH e, s
     MATCH s<-[:BELONGS_TO]-(c:Category)
     WITH e, s, collect(DISTINCT c.id) as categories
     RETURN {
       id          : e.id,
       created     : s.created,
       version     : s.version,
       title       : s.title,
       description : s.description,
       lat         : s.lat,
       lng         : s.lng,
       street      : s.street,
       zip         : s.zip,
       city        : s.city,
       country     : s.country,
       email       : s.email,
       telephone   : s.telephone,
       homepage    : s.homepage,
       license     : s.license,
       categories  : categories
     } AS e",
   {
     "id"          => &id,
     "version"     => &version,
     "title"       => &e.title,
     "description" => &e.description,
     "lat"         => &e.lat,
     "lng"         => &e.lng,
     "street"      => &e.street,
     "zip"         => &e.zip,
     "city"        => &e.city,
     "country"     => &e.country,
     "email"       => &e.email,
     "telephone"   => &e.telephone,
     "homepage"    => &e.homepage,
     "license"     => &e.license,
     "categories"  => &e.categories.clone().unwrap_or(vec!())
    })));
    result.rows()
        .next()
        .ok_or(StoreError::Save)
        .and_then(|r| r.get::<Entry>("e").map_err(StoreError::Graph))
}

impl Store for Category {
    type Id = String;
    type Connection = GraphClient;
    type Error = StoreError;

    fn get(graph: &Self::Connection, id: Self::Id) -> Result<Category, StoreError> {
        let result = try!(graph.cypher().exec(cypher_stmt!(
     "MATCH (e:Category)<--(s:CategoryState) WHERE c.id = {id}
      WITH max(s.created) as created
      MATCH (x:Category)<--(s:CategoryState)
      WHERE c.id = {id} AND s.created = created
      WITH c, s
      RETURN {
        id      : c.id,
        version : s.version,
        created : s.created,
        name    : s.name
      } AS c", {"id" => &id})));
        result.rows()
            .next()
            .ok_or(StoreError::NotFound)
            .and_then(|r| r.get::<Category>("c").map_err(StoreError::Graph))
    }

    fn all(graph: &Self::Connection) -> Result<Vec<Category>, StoreError> {
        let result = try!(graph.cypher().exec(
      "MATCH (c:Category)<--(s:CategoryState)
       RETURN {
         id      : c.id,
         version : s.version,
         created : s.created,
         name    : s.name
       } AS c"));
        Ok(result.rows()
            .filter_map(|r| r.get::<Category>("c").ok())
            .collect::<Vec<Category>>())
    }

    fn save(&self, graph: &Self::Connection) -> Result<Category, StoreError> {
        match self.id {
            None => create_category(self, graph),
            Some(_) => update_category(self, graph),
        }
    }
}

fn create_category(c: &Category, graph: &GraphClient) -> Result<Category, StoreError> {
    let id = match c.id {
        None => Uuid::new_v4().to_simple_string(),
        Some(_) => return Err(StoreError::InvalidId),
    };
    if let Some(v) = c.version {
        if !(v == 0 || v == 1) {
            return Err(StoreError::InvalidVersion);
        }
    }
    let result = try!(graph.cypher().exec(cypher_stmt!(
    "CREATE (c:Category {id:{id}})
     MERGE c<-[:BELONGS_TO]-(s:CategoryState {
       created : timestamp(),
       version : 1,
       name    : {name}
     })
     RETURN {
       id      : c.id,
       version : s.version,
       created : s.created,
       name    : s.name
     } AS c", {
       "id"   => &id,
       "name" => &c.name
    })));
    result.rows()
        .next()
        .ok_or(StoreError::Save)
        .and_then(|r| r.get::<Category>("c").map_err(StoreError::Graph))
}

fn update_category(c: &Category, graph: &GraphClient) -> Result<Category, StoreError> {
    let id = try!(c.id.clone().ok_or(StoreError::InvalidId));
    let version = try!(c.version.ok_or(StoreError::InvalidVersion));
    debug!("update category: {}", id);
    let result = try!(graph.cypher()
  .exec(cypher_stmt!(
   "MATCH (c:Category)<--(s:CategoryState) WHERE c.id = {id}
    WITH max(s.version) as v
    MATCH (c:Category)<--(old:CategoryState)
    WHERE c.id = {id} AND old.version = v AND old.version + 1 = {version}
    WITH c,e
    MERGE c<-[:BELONGS_TO]-(s:CategoryState {
      created : timestamp(),
      version : {version},
      name    : {name}
    })
    RETURN {
      id      : c.id,
      version : s.version,
      created : s.created,
      name    : s.name
    } AS c", {
      "id"      => &id,
      "version" => &version,
      "name"    => &c.name
    })));
    result.rows()
        .next()
        .ok_or(StoreError::Save)
        .and_then(|r| r.get::<Category>("c").map_err(StoreError::Graph))
}
