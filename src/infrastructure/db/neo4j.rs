use business::db::Db;
use entities::*;
use rusted_cypher::GraphClient;
use business::error::RepoError;
use std::result;

type Result<T> = result::Result<T, RepoError>;

impl Db for GraphClient {

    fn get_entry(&self, id: &str) -> Result<Entry> {
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

    fn get_user(&self, username: &str) -> Result<User> {
        let result = self.exec(cypher_stmt!(
        "MATCH u:User
         WHERE u.username = {username}",
        { "username" => username })?)?;
        let r = result.rows().next().ok_or(RepoError::NotFound)?;
        let u = r.get::<User>("u")?;
        Ok(u)
    }

    fn all_entries(&self) -> Result<Vec<Entry>> {
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

    fn create_entry(&mut self, e: &Entry) -> Result<()> {
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

    fn create_tag(&mut self, t: &Tag) -> Result<()> {
        self.exec(cypher_stmt!("MERGE (t:Tag {id:{id}})",
        { "id" => &t.id })?)?;
        Ok(())
    }

    fn create_user(&mut self, u: &User) -> Result<()> {
        self.exec(cypher_stmt!(
        "MERGE (
           u:User {
             username:{username},
             password:{password},
             email:{email},
           }
        )",
        {
            "username" => &u.username,
            "password" => &u.password,
            "email"    => &u.email
        })?)?;
        Ok(())
    }

    fn create_triple(&mut self, t: &Triple) -> Result<()> {
        let predicate = match t.predicate {
            Relation::IsTaggedWith => "IS_TAGGED_WITH"
        };
        let (subject_type, subject_id) = match t.subject {
            ObjectId::Entry(ref id) => ("Entry",id),
            ObjectId::Tag(ref id) => ("Tag",id),
            ObjectId::User(ref id) => ("User",id)
        };
        let (object_type, object_id) = match t.object {
            ObjectId::Entry(ref id) => ("Entry",id),
            ObjectId::Tag(ref id) => ("Tag",id),
            ObjectId::User(ref id) => ("User",id)
        };
        let stmt = format!(
           "MATCH (s:{s_type})
            WHERE s.id = \"{s_id}\"
            WITH s
            MATCH (o:{o_type})
            WHERE o.id = \"{o_id}\"
            WITH s,o
            MERGE (s)-[:{predicate}]->(o)",
                s_type = subject_type,
                s_id = subject_id,
                o_type = object_type,
                o_id = object_id,
                predicate = predicate
            );
        self.exec(stmt)?;
        Ok(())
    }

    fn update_entry(&mut self, e: &Entry) -> Result<()> {
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
         )",
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

    fn all_categories(&self) -> Result<Vec<Category>> {
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

    fn all_triples(&self) -> Result<Vec<Triple>> {
        //TODO: extend for category
        let result = self.exec(
        "MATCH (e:Entry)-[IS_TAGGED_WITH]->(t:Tag)
         RETURN {
           subject   : { entry: e.id },
           predicate : \"is_tagged_with\",
           object    : { tag: t.id }
         } AS t")?;
        Ok(result
            .rows()
            .filter_map(|r| r.get::<Triple>("t").ok())
            .collect::<Vec<Triple>>())
    }

    fn all_tags(&self) -> Result<Vec<Tag>> {
        let result = self.exec(
        "MATCH (t:Tag) RETURN t")?;
        Ok(result
            .rows()
            .filter_map(|r| r.get::<Tag>("t").ok())
            .collect::<Vec<Tag>>())
    }

    fn delete_triple(&mut self, t: &Triple) -> Result<()> {
        let predicate = match t.predicate {
            Relation::IsTaggedWith => "IS_TAGGED_WITH"
        };
        let (subject_type, subject_id) = match t.subject {
            ObjectId::Entry(ref id) => ("Entry",id),
            ObjectId::Tag(ref id) => ("Tag",id),
            ObjectId::User(ref id) => ("User",id)
        };
        let (object_type, object_id) = match t.object {
            ObjectId::Entry(ref id) => ("Entry",id),
            ObjectId::Tag(ref id) => ("Tag",id),
            ObjectId::User(ref id) => ("User",id)
        };
        let stmt = format!(
           "MATCH (s:{s_type})-[p:{predicate}]->(o:{o_type})
            WHERE s.id = \"{s_id}\" AND o.id = \"{o_id}\"
            DELETE p",
                s_type = subject_type,
                s_id = subject_id,
                o_type = object_type,
                o_id = object_id,
                predicate = predicate
            );
        self.exec(stmt)?;
        Ok(())
    }
}
