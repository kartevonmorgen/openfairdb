use business::db::Repo;
use entities::*;
use rusted_cypher::GraphClient;
use business::error::RepoError;
use std::result;

type Result<T> = result::Result<T, RepoError>;

impl Repo<Entry> for GraphClient {
    type Id = String;

    fn get(&self, id: Self::Id) -> Result<Entry> {
        let result = self.exec(cypher_stmt!(
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
         ORDER BY e.created DESC", {"id" => &id})?)?;
        let r = result.rows().next().ok_or(RepoError::NotFound)?;
        let e = r.get::<Entry>("e")?;
        Ok(e)
    }

    fn all(&self) -> Result<Vec<Entry>> {
        let result = self.exec(
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
          ORDER BY e.created DESC")?;
        Ok(result.rows()
            .filter_map(|r| r.get::<Entry>("e").ok())
            .collect::<Vec<Entry>>())
    }

    fn create(&mut self, e: &Entry) -> Result<()> {
        self.exec(cypher_stmt!(
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
         )",
        {
            "id"          => &e.id,
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
            "categories"  => &e.categories
        })?)?;
        Ok(())
    }

    fn update(&mut self, e: &Entry) -> Result<()> {
        self.exec(cypher_stmt!(
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
         WITH e, s, collect(DISTINCT c.id) as categories",
        {
            "id"          => &e.id,
            "version"     => &e.version,
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
            "categories"  => &e.categories
        })?)?;
        Ok(())
    }
}

impl Repo<Category> for GraphClient {
    type Id = String;

    fn get(&self, id: Self::Id) -> Result<Category> {
        let result = self.exec(cypher_stmt!(
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
         } AS c", {"id" => &id})?)?;
        let r = result.rows().next().ok_or(RepoError::NotFound)?;
        let c = r.get::<Category>("c")?;
        Ok(c)
            
    }

    fn all(&self) -> Result<Vec<Category>> {
        let result = self.exec(
        "MATCH (c:Category)<--(s:CategoryState)
         RETURN {
           id      : c.id,
           version : s.version,
           created : s.created,
           name    : s.name
         } AS c")?;
        Ok(result
            .rows()
            .filter_map(|r| r.get::<Category>("c").ok())
            .collect::<Vec<Category>>())
    }

    fn create(&mut self, c: &Category) -> Result<()> {
        self.exec(cypher_stmt!(
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
           "id"   => &c.id,
           "name" => &c.name
        })?)?;
        Ok(())
    }

    fn update(&mut self, c: &Category) -> Result<()> {
        debug!("update category: {}", c.id);
        self.exec(cypher_stmt!(
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
           "id"      => &c.id,
           "version" => &c.version,
           "name"    => &c.name
         })?)?;
        Ok(())
    }
}
