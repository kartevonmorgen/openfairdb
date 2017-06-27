use business::db::Db;
use entities::*;
use rusted_cypher::GraphClient;
use business::error::RepoError;
use std::result;

type Result<T> = result::Result<T, RepoError>;

fn neo4j_edge_name(t: &Triple) -> &str {
    match t.predicate {
       Relation::IsTaggedWith    => "IS_TAGGED_WITH",
       Relation::IsRatedWith     => "IS_RATED_WITH",
       Relation::IsCommentedWith => "IS_COMMENTED_WITH",
       Relation::CreatedBy       => "CREATED_BY"
    }.into()
}

fn object_id_to_neo4j_label(id: &ObjectId) -> (&str,&str) {
    match *id {
        ObjectId::Entry(ref id) => ("Entry",id),
        ObjectId::Tag(ref id) => ("Tag",id),
        ObjectId::User(ref id) => ("User",id),
        ObjectId::Comment(ref id) => ("Comment",id),
        ObjectId::Rating(ref id) => ("Rating",id)
    }
}

fn neo4j_label_to_object_id(label: &str, id: String) -> Option<ObjectId> {
    match label {
        "Entry"     => Some(ObjectId::Entry(id)),
        "Tag"       => Some(ObjectId::Tag(id)),
        "User"      => Some(ObjectId::User(id)),
        "Comment"   => Some(ObjectId::Comment(id)),
        "Rating"    => Some(ObjectId::Rating(id)),
        _           => None,
    }
}

fn neo4j_relation_to_relation(rel: &str) -> Option<Relation> {
    match rel {
        "IS_TAGGED_WITH"    => Some(Relation::IsTaggedWith),
        "IS_RATED_WITH"     => Some(Relation::IsRatedWith),
        "IS_COMMENTED_WITH" => Some(Relation::IsCommentedWith),
        "CREATED_BY"        => Some(Relation::CreatedBy),
        _                   => None
    }
}


#[derive(Deserialize,Debug)]
struct Neo4jObj {
    labels: Vec<String>,
    id: String
}

#[derive(Deserialize,Debug)]
struct Neo4jTriple {
    subject: Neo4jObj,
    predicate: String,
    object: Neo4jObj
}

fn from_neo_triple(t: Neo4jTriple) -> Option<Triple> {
    if t.subject.labels.is_empty() || t.object.labels.is_empty() {
        warn!("No labels found in {:?}", t);
        return None;
    }
    let subject = neo4j_label_to_object_id(&t.subject.labels[0], t.subject.id);
    let predicate = neo4j_relation_to_relation(&t.predicate);
    let object = neo4j_label_to_object_id(&t.object.labels[0], t.object.id);

    if let (Some(s), Some(p), Some(o)) = (subject, predicate, object) {
        Some(Triple {
            subject: s,
            predicate: p,
            object: o
        })
    } else {
        None
    }
}


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
        "MATCH (u:User)
         WHERE u.username = {username}
         RETURN u",
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
             email:{email}
           }
        )",
        {
            "username" => &u.username,
            "password" => &u.password,
            "email"    => &u.email
        })?)?;
        Ok(())
    }

    fn create_rating(&mut self, r: &Rating) -> Result<()> {
        let s = r.source.clone();
        println!("source: {}", s.unwrap_or("no source".into()));
        self.exec(cypher_stmt!(
        "MERGE (r:Rating {
             id      : {id},
             title   : {title},
             created : {created},
             value   : {value},
             context : {context},
             source  : {source}
        })",
        {
            "id"        => &r.id,
            "title"     => &r.title,
            "created"   => &r.created,
            "value"     => &r.value,
            "context"   => &r.context,
            "source"    => &r.source
        })?)?;
        println!("Ok");
        Ok(())
    }

    fn create_comment(&mut self, c: &Comment) -> Result<()> {
        self.exec(cypher_stmt!(
        "MERGE (
           c:Comment {
             id      : {id},
             created : {created},
             text    : {text}
           }
        )",
        {
            "id"        => &c.id,
            "created"   => &c.created,
            "text"      => &c.text
        })?)?;
        Ok(())
    }

    fn create_triple(&mut self, t: &Triple) -> Result<()> {

        let predicate = neo4j_edge_name(t);
        let (subject_type, subject_id) = object_id_to_neo4j_label(&t.subject);
        let (object_type, object_id) = object_id_to_neo4j_label(&t.object);

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
        let result = self.exec(
        "MATCH (s)-[p]->(o)
         WHERE (s.id IS NOT NULL) AND (o.id IS NOT NULL)
         RETURN {
            subject: { id: s.id, labels: labels(s) },
            predicate: type(p),
            object: { id: o.id, labels: labels(o) }
         } AS t")?;
        Ok(result
            .rows()
            .filter_map(|r|{
                match r.get::<Neo4jTriple>("t") {
                    Err(err) => {
                         warn!("{}",err);
                         None
                    }
                    Ok(t) => from_neo_triple(t)
                }
            })
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

    fn all_ratings(&self) -> Result<Vec<Rating>> {
        let result = self.exec(
        "MATCH (r:Rating) RETURN r")?;
        Ok(result
            .rows()
            .filter_map(|r| r.get::<Rating>("r").ok())
            .collect::<Vec<Rating>>())
    }

    fn all_comments(&self) -> Result<Vec<Comment>> {
        let result = self.exec(
        "MATCH (c:Comment) RETURN c")?;
        Ok(result
            .rows()
            .filter_map(|r| r.get::<Comment>("c").ok())
            .collect::<Vec<Comment>>())
    }

    fn delete_triple(&mut self, t: &Triple) -> Result<()> {
        let predicate = neo4j_edge_name(t);
        let (subject_type, subject_id) = object_id_to_neo4j_label(&t.subject);
        let (object_type, object_id) = object_id_to_neo4j_label(&t.object);
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
