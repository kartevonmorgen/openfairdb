use super::error::{Error, ParameterError, RepoError};
use std::result;
use chrono::*;
use entities::*;
use super::db::Db;
use super::filter;
use super::validate::{self, Validate};
use uuid::Uuid;
use std::collections::HashMap;
use pwhash::bcrypt;
use super::geo;

#[cfg(test)]
pub mod tests;

type Result<T> = result::Result<T, Error>;

trait Id {
    fn id(&self) -> String;
}

impl Id for Entry {
    fn id(&self) -> String {
        self.id.clone()
    }
}

impl Id for Category {
    fn id(&self) -> String {
        self.id.clone()
    }
}

impl Id for Tag {
    fn id(&self) -> String {
        self.id.clone()
    }
}

impl Id for User {
    fn id(&self) -> String {
        self.id.clone()
    }
}

impl Id for Comment {
    fn id(&self) -> String {
        self.id.clone()
    }
}

impl Id for Rating {
    fn id(&self) -> String {
        self.id.clone()
    }
}

impl Id for BboxSubscription {
    fn id(&self) -> String {
        self.id.clone()
    }
}

fn triple_id(t: &Triple) -> String {
    let (s_type, s_id) = match t.subject {
        ObjectId::Entry(ref id) => ("entry", id),
        ObjectId::Tag(ref id) => ("tag", id),
        ObjectId::User(ref id) => ("user", id),
        ObjectId::Comment(ref id) => ("comment", id),
        ObjectId::Rating(ref id) => ("rating", id),
        ObjectId::BboxSubscription(ref id) => ("bbox_subscription", id)
    };
    let (o_type, o_id) = match t.object {
        ObjectId::Entry(ref id) => ("entry", id),
        ObjectId::Tag(ref id) => ("tag", id),
        ObjectId::User(ref id) => ("user", id),
        ObjectId::Comment(ref id) => ("comment", id),
        ObjectId::Rating(ref id) => ("rating", id),
        ObjectId::BboxSubscription(ref id) => ("bbox_subscription", id)
    };
    let p_type = match t.predicate {
        Relation::IsTaggedWith => "is_tagged_with",
        Relation::IsRatedWith => "is_rated_with",
        Relation::IsCommentedWith => "is_commented_with",
        Relation::CreatedBy => "created_by",
        Relation::SubscribedTo => "subscribed_to"
    };
    format!("{}-{}-{}-{}-{}", s_type, s_id, p_type, o_type, o_id)
}

impl Id for Triple {
    fn id(&self) -> String {
        triple_id(self)
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct NewEntry {
    pub title       : String,
    pub description : String,
    pub lat         : f64,
    pub lng         : f64,
    pub street      : Option<String>,
    pub zip         : Option<String>,
    pub city        : Option<String>,
    pub country     : Option<String>,
    pub email       : Option<String>,
    pub telephone   : Option<String>,
    pub homepage    : Option<String>,
    pub categories  : Vec<String>,
    pub tags        : Vec<String>,
    pub license     : String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct NewUser {
    pub username: String,
    pub password: String,
    pub email: String
}

#[derive(Deserialize, Debug, Clone)]
pub struct Login {
    username: String,
    password: String
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UpdateEntry {
    pub id          : String,
    pub version     : u64,
    pub title       : String,
    pub description : String,
    pub lat         : f64,
    pub lng         : f64,
    pub street      : Option<String>,
    pub zip         : Option<String>,
    pub city        : Option<String>,
    pub country     : Option<String>,
    pub email       : Option<String>,
    pub telephone   : Option<String>,
    pub homepage    : Option<String>,
    pub categories  : Vec<String>,
    pub tags        : Vec<String>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct RateEntry {
    pub entry: String,
    pub title: String,
    pub value: i8,
    pub context: RatingContext,
    pub comment: String,
    pub source: Option<String>,
    pub user: Option<String>
}

fn create_missing_tags<D: Db>(db: &mut D, tags: &[String]) -> Result<()> {
    let existing_tags = db.all_tags()?;
    for new_t in tags {
        if !existing_tags.iter().any(|t| t.id == *new_t) {
            db.create_tag(&Tag { id: new_t.clone() })?;
        }
    }
    Ok(())
}

struct Diff<T> {
    new: Vec<T>,
    deleted: Vec<T>
}

fn get_triple_diff(old: &[Triple], new: &[Triple]) -> Diff<Triple> {

    let to_create = new
        .iter()
        .filter(|t|!old.iter().any(|x| x == *t))
        .cloned()
        .collect::<Vec<Triple>>();

    let to_delete = old
        .iter()
        .filter(|t|!new.iter().any(|x| x == *t))
        .cloned()
        .collect::<Vec<Triple>>();

    Diff {
        new: to_create,
        deleted: to_delete
    }
}


fn set_tag_relations<D: Db>(db: &mut D, entry: &str, tags: &[String]) -> Result<()> {
    create_missing_tags(db, tags)?;
    let subject = ObjectId::Entry(entry.into());
    let old_triples = db.all_triples()?
        .into_iter()
        .filter(|x| x.subject == subject)
        .filter(|x| x.predicate == Relation::IsTaggedWith)
        .collect::<Vec<Triple>>();
    let new_triples = tags
        .into_iter()
        .map(|x| Triple{
            subject: subject.clone(),
            predicate: Relation::IsTaggedWith,
            object: ObjectId::Tag(x.clone())
        })
        .collect::<Vec<Triple>>();

    let diff = get_triple_diff(&old_triples, &new_triples);

    for t in diff.new {
        db.create_triple(&t)?;
    }
    for t in diff.deleted {
        db.delete_triple(&t)?;
    }
    Ok(())
}

pub fn get_tag_ids<D:Db>(db: &D) -> Result<Vec<String>> {
    let mut tags : Vec<String> = db
        .all_triples()?
        .into_iter()
        .filter(|t| t.predicate == Relation::IsTaggedWith)
        .filter_map(|t| match t.object {
           ObjectId::Tag(id) => Some(id),
            _ => None
        })
        .collect();
    tags.dedup();
    Ok(tags)
}

pub fn get_tag_ids_for_entry_id(triples: &[Triple], entry_id: &str) -> Vec<String> {
    triples
        .iter()
        .filter(&*filter::triple_by_subject(ObjectId::Entry(entry_id.into())))
        .filter(|triple| triple.predicate == Relation::IsTaggedWith)
        .map(|triple|&triple.object)
        .filter_map(|object|
            match *object {
                ObjectId::Tag(ref tag_id) => Some(tag_id),
                _ => None
            })
        .cloned()
        .collect()
}

pub fn get_rating_ids_for_entry_id(triples: &[Triple], entry_id: &str) -> Vec<String> {
    triples
        .iter()
        .filter(&*filter::triple_by_subject(ObjectId::Entry(entry_id.into())))
        .filter(|triple| triple.predicate == Relation::IsRatedWith)
        .map(|triple|&triple.object)
        .filter_map(|object|
            match *object {
                ObjectId::Rating(ref r_id) => Some(r_id),
                _ => None
            })
        .cloned()
        .collect()
}

pub fn get_ratings<D:Db>(db: &D, ids : &[String]) -> Result<Vec<Rating>> {
    Ok(db
        .all_ratings()?
        .iter()
        .filter(|x|ids.iter().any(|id|*id==x.id))
        .cloned()
        .collect())
}

pub fn get_comment_ids_for_rating_id(triples: &[Triple], rating_id: &str) -> Vec<String> {
    triples
        .iter()
        .filter(&*filter::triple_by_subject(ObjectId::Rating(rating_id.into())))
        .filter(|triple| triple.predicate == Relation::IsCommentedWith)
        .map(|triple|&triple.object)
        .filter_map(|object|
            match *object {
                ObjectId::Comment(ref r_id) => Some(r_id),
                _ => None
            })
        .cloned()
        .collect()
}

pub fn get_user_id_for_comment_id(triples: &[Triple], comment_id: &str) -> Option<String> {
    triples
        .iter()
        .filter(&*filter::triple_by_subject(ObjectId::Comment(comment_id.into())))
        .filter(|triple| triple.predicate == Relation::CreatedBy)
        .map(|triple|&triple.object)
        .filter_map(|object|
            match *object {
                ObjectId::User(ref r_id) => Some(r_id),
                _ => None
            })
        .cloned()
        .last()
}

pub fn get_user_id_for_rating_id(triples: &[Triple], rating_id: &str) -> Option<String> {
    let r_id = ObjectId::Rating(rating_id.to_string());
    triples
        .iter()
        .filter(&*filter::triple_by_subject(r_id))
        .filter(|triple| triple.predicate == Relation::CreatedBy)
        .map(|triple|&triple.object)
        .filter_map(|object|
            match *object {
                ObjectId::User(ref r_id) => Some(r_id),
                _ => None
            })
        .cloned()
        .last()
}

pub fn get_tags_by_entry_ids<D: Db>(db: &D, ids: &[String]) -> Result<HashMap<String, Vec<Tag>>> {
    let triples = db.all_triples()?;
    Ok(ids
        .iter()
        .map(|id|(
            id.clone(),
            get_tag_ids_for_entry_id(&triples, id)
                .into_iter()
                .map(|tag_id|Tag{id: tag_id})
                .collect()
        ))
        .collect())
}

pub fn get_ratings_by_entry_ids<D:Db>(db : &D, ids : &[String]) -> Result<HashMap<String, Vec<Rating>>> {
    let triples = db.all_triples()?;
    let ratings = db.all_ratings()?;
    Ok(ids
        .iter()
        .map(|id|(
            id.clone(),
            get_rating_ids_for_entry_id(&triples, id)
                .iter()
                .filter_map(|r_id| ratings.iter().find(|x| x.id == *r_id))
                .cloned()
                .collect()
        ))
        .collect())
}

pub fn get_comments_by_rating_ids<D:Db>(db : &D, ids : &[String]) -> Result<HashMap<String, Vec<Comment>>> {
    let triples = db.all_triples()?;
    let comments = db.all_comments()?;
    Ok(ids
        .iter()
        .map(|id|(
            id.clone(),
            get_comment_ids_for_rating_id(&triples, id)
                .iter()
                .filter_map(|c_id| comments.iter().find(|x| x.id == *c_id))
                .cloned()
                .collect()
        ))
        .collect())
}

pub fn get_entries<D:Db>(db : &D, ids : &[String]) -> Result<Vec<Entry>> {
    let entries = db
        .all_entries()?
        .into_iter()
        .filter(|e| ids.iter().any(|id| *id == e.id))
        .collect();
    Ok(entries)
}

pub fn create_new_user<D: Db>(db: &mut D, u: NewUser) -> Result<()> {
    debug!("create new user");
    validate::username(&u.username)?;
    validate::password(&u.password)?;
    validate::email(&u.email)?;
    debug!("validated data");
    if db.get_user(&u.username).is_ok() {
        return Err(Error::Parameter(ParameterError::UserExists));
    }
    debug!("username is still free");

    let pw = bcrypt::hash(&u.password)?;
    db.create_user(&User{
        id: Uuid::new_v4().simple().to_string(),
        username: u.username,
        password: pw,
        email: u.email,
        email_confirmed: false
    })?;
    Ok(())
}

pub fn get_user<D: Db>(db: &mut D, login_id: &str, username: &str) -> Result<(String,String)> {
    let users : Vec<User> = db.all_users()?
        .into_iter()
        .filter(|u| u.id == login_id)
        .collect();
    if users.len() > 0 {
        let login_name = &users[0].username;
        if login_name != username {
            return Err(Error::Parameter(ParameterError::Forbidden))
        }
        let u = db.get_user(username)?;
        Ok((u.id, u.email))
    } else {
        return Err(Error::Repo(RepoError::NotFound))
    }
}

pub fn delete_user(db: &mut Db, login_id: &str, username: &str) -> Result<()> {
    let users : Vec<User> = db.all_users()?
        .into_iter()
        .filter(|u| u.id == login_id)
        .collect();
    if users.len() > 0 {
        let login_name = &users[0].username;
        if login_name != username {
            return Err(Error::Parameter(ParameterError::Forbidden))
        }
        db.delete_user(login_id)?;
        Ok(())
    } else {
        return Err(Error::Repo(RepoError::NotFound))
    }    
}

pub fn login<D: Db>(db: &mut D, login: Login) -> Result<String> {
    match db.get_user(&login.username) {
        Ok(u) => {
            if bcrypt::verify(&login.password, &u.password) {
                if u.email_confirmed {
                    Ok(u.id)
                } else{
                    Err(Error::Parameter(ParameterError::EmailNotConfirmed))
                }
            } else {
                Err(Error::Parameter(ParameterError::Credentials))
            }
        }
        Err(err) => {
            match err {
                RepoError::NotFound => {
                    Err(Error::Parameter(ParameterError::Credentials))
                }
                _=> Err(Error::Repo(RepoError::Other(Box::new(err))))
            }
        }
    }
}

pub fn create_new_entry<D: Db>(db: &mut D, e: NewEntry) -> Result<String> {
    let new_entry = Entry{
        id          :  Uuid::new_v4().simple().to_string(),
        created     :  Utc::now().timestamp() as u64,
        version     :  0,
        title       :  e.title,
        description :  e.description,
        lat         :  e.lat,
        lng         :  e.lng,
        street      :  e.street,
        zip         :  e.zip,
        city        :  e.city,
        country     :  e.country,
        email       :  e.email,
        telephone   :  e.telephone,
        homepage    :  e.homepage,
        categories  :  e.categories,
        license     :  Some(e.license)
    };
    new_entry.validate()?;
    db.create_entry(&new_entry)?;
    set_tag_relations(db, &new_entry.id, &e.tags)?;
    Ok(new_entry.id)
}

pub fn update_entry<D: Db>(db: &mut D, e: UpdateEntry) -> Result<()> {
    let old : Entry = db.get_entry(&e.id)?;
    if (old.version + 1) != e.version {
        return Err(Error::Repo(RepoError::InvalidVersion))
    }
    let new_entry = Entry{
        id          :  e.id,
        created     :  Utc::now().timestamp() as u64,
        version     :  e.version,
        title       :  e.title,
        description :  e.description,
        lat         :  e.lat,
        lng         :  e.lng,
        street      :  e.street,
        zip         :  e.zip,
        city        :  e.city,
        country     :  e.country,
        email       :  e.email,
        telephone   :  e.telephone,
        homepage    :  e.homepage,
        categories  :  e.categories,
        license     :  old.license
    };
    db.update_entry(&new_entry)?;
    set_tag_relations(db, &new_entry.id, &e.tags)?;
    Ok(())
}

pub fn rate_entry<D: Db>(db: &mut D, r: RateEntry) -> Result<()> {
    let e = db.get_entry(&r.entry)?;
    if r.comment.len() < 1 {
        return Err(Error::Parameter(ParameterError::EmptyComment));
    }
    if r.value > 2 || r.value < -1 {
        return Err(Error::Parameter(ParameterError::RatingValue));
    }
    let now = Utc::now().timestamp() as u64;
    let rating_id = Uuid::new_v4().simple().to_string();
    let comment_id = Uuid::new_v4().simple().to_string();
    db.create_rating(&Rating{
        id      : rating_id.clone(),
        created : now,
        title   : r.title,
        value   : r.value,
        context : r.context,
        source  : r.source
    })?;
    db.create_comment(&Comment{
        id      : comment_id.clone(),
        created : now,
        text    : r.comment,
    })?;
    db.create_triple(&Triple{
        subject: ObjectId::Entry(e.id),
        predicate: Relation::IsRatedWith,
        object: ObjectId::Rating(rating_id.clone()),
    })?;
    db.create_triple(&Triple{
        subject: ObjectId::Rating(rating_id),
        predicate: Relation::IsCommentedWith,
        object: ObjectId::Comment(comment_id),
    })?;
    Ok(())
}

pub fn subscribe_to_bbox(coordinates: &Vec<Coordinate>, username: &str, db: &mut Db) -> Result<()>{
    if coordinates.len() != 2 {
        return Err(Error::Parameter(ParameterError::Bbox));
    }
    let bbox = Bbox {
        south_west: coordinates[0].clone(),
        north_east: coordinates[1].clone()
    };
    validate::bbox(&bbox)?;

    create_or_modify_subscription(&bbox, username.into(), db)?;
    Ok(())
}

pub fn get_bbox_subscriptions(username: &str, db: &Db) -> Result<Vec<BboxSubscription>>{
    let user_subscriptions : Vec<String>  = db.all_triples()?
        .into_iter()
        .filter_map(|triple| match triple {
            Triple {
                subject     : ObjectId::User(ref u_id),
                predicate   : Relation::SubscribedTo,
                object      : ObjectId::BboxSubscription(ref s_id)
            } => Some((u_id.clone(), s_id.clone())),
            _ => None
        })
        .filter(|user_subscription| *user_subscription.0 == *username)
        .map(|user_and_subscription| user_and_subscription.1)
        .collect();
    if user_subscriptions.len() > 0 {
        return Ok(db.all_bbox_subscriptions()?
            .into_iter()
            .filter(|s| user_subscriptions
                .clone()
                .into_iter()
                .any(|id| s.id == id))
            .collect());
    } else{
        return Ok(vec![]);
    }
}

pub fn create_or_modify_subscription(bbox: &Bbox, username: String, db: &mut Db) -> Result<()>{
    let user_subscriptions : Vec<String>  = db.all_triples()?
        .into_iter()
        .filter_map(|triple| match triple {
            Triple {
                subject     : ObjectId::User(ref u_id),
                predicate   : Relation::SubscribedTo,
                object      : ObjectId::BboxSubscription(ref s_id)
            } => Some((u_id.clone(), s_id.clone())),
            _ => None
        })
        .filter(|user_subscription| *user_subscription.0 == *username)
        .map(|user_and_subscription| user_and_subscription.1)
        .collect();

    if user_subscriptions.len() > 0 {
        db.delete_bbox_subscription(&user_subscriptions[0].clone())?;      
    }

    let s_id = Uuid::new_v4().simple().to_string();
    db.create_bbox_subscription(&BboxSubscription{
        id: s_id.clone(),
        south_west_lat: bbox.south_west.lat,
        south_west_lng: bbox.south_west.lng,
        north_east_lat: bbox.north_east.lat,
        north_east_lng: bbox.north_east.lng,
    })?;

    db.create_triple(&Triple{
        subject     : ObjectId::User(username),
        predicate   : Relation::SubscribedTo,
        object      : ObjectId::BboxSubscription(s_id.into())
    })?;
    Ok(())
}

pub fn unsubscribe_all_bboxes(username: &str, db: &mut Db) -> Result<()>{
    let user_subscriptions : Vec<String>  = db.all_triples()?
        .into_iter()
        .filter_map(|triple| match triple {
            Triple {
                subject     : ObjectId::User(ref u_id),
                predicate   : Relation::SubscribedTo,
                object      : ObjectId::BboxSubscription(ref s_id)
            } => Some((u_id.clone(), s_id.clone())),
            _ => None
        })
        .filter(|user_subscription| *user_subscription.0 == *username)
        .map(|user_and_subscription| user_and_subscription.1)
        .collect();

    for s_id in user_subscriptions {
        db.delete_bbox_subscription(&s_id)?;
    }
    Ok(())
}

pub fn email_addresses_to_notify(lat: &f64, lng: &f64, db: &mut Db) -> Vec<String>{
    debug!("finding notification email addresses");
    let users_and_bboxes : Vec<(String, Bbox)> = db.all_triples()
        .unwrap()
        .into_iter()
        .filter_map(|triple| match triple {
            Triple {
                subject     : ObjectId::User(ref u_id),
                predicate   : Relation::SubscribedTo,
                object      : ObjectId::BboxSubscription(ref s_id)
            } => Some((u_id.clone(), s_id.clone())),
            _ => None
        })
        .map(|(u_id, s_id)| (db.all_users()
            .unwrap()
            .into_iter()
            .filter(|u| u.id == u_id)
            .map(|u| u.email)
            .nth(0).unwrap(),
            s_id))
        .map(|(u_id, s_id)| (u_id, db.all_bbox_subscriptions()
            .unwrap()
            .into_iter()
            .filter(|s| s.id == s_id)
            .map(|s| Bbox{
                south_west: Coordinate {
                    lat: s.south_west_lat,
                    lng: s.south_west_lng
                },
                north_east: Coordinate {
                    lat: s.north_east_lat,
                    lng: s.north_east_lng
                }
            })
            .nth(0).unwrap()))
        .collect();

    let emails_to_notify : Vec<String> = users_and_bboxes.clone()
        .into_iter()
        .filter(|&(_, ref bbox)| geo::is_in_bbox(lat, lng, &bbox))
        .map(|(email, _)| email)
        .collect();

    emails_to_notify
}
