use entities as e;
use super::models::*;
use std::str::FromStr;

impl From<e::Entry> for Entry {
    fn from(e: e::Entry) -> Entry {
        let e::Entry {
            id,
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
            license,
            ..
        } = e;

        Entry {
            id,
            created: created as i32,
            version: version as i32,
            current: true,
            title,
            description,
            lat: lat as f32,
            lng: lng as f32,
            street,
            zip,
            city,
            country,
            email,
            telephone,
            homepage,
            license,
        }
    }
}

impl From<Category> for e::Category {
    fn from(c: Category) -> e::Category {
        let Category {
            id,
            name,
            created,
            version,
        } = c;
        e::Category {
            id,
            name,
            created: created as u64,
            version: version as u64,
        }
    }
}

impl From<Tag> for e::Tag {
    fn from(t: Tag) -> e::Tag {
        e::Tag { id: t.id }
    }
}

impl From<e::Tag> for Tag {
    fn from(t: e::Tag) -> Tag {
        Tag { id: t.id }
    }
}

impl From<EntryTagRelation> for e::Triple {
    fn from(r: EntryTagRelation) -> e::Triple {
        e::Triple {
            subject: e::ObjectId::Entry(r.entry_id),
            predicate: e::Relation::IsTaggedWith,
            object: e::ObjectId::Tag(r.tag_id),
        }
    }
}

impl From<Rating> for e::Triple {
    fn from(r: Rating) -> e::Triple {
        e::Triple {
            subject: e::ObjectId::Entry(r.entry_id.unwrap()), //TODO
            predicate: e::Relation::IsRatedWith,
            object: e::ObjectId::Rating(r.id),
        }
    }
}

impl From<Comment> for e::Triple {
    fn from(c: Comment) -> e::Triple {
        e::Triple {
            subject: e::ObjectId::Rating(c.rating_id.unwrap()), //TODO
            predicate: e::Relation::IsCommentedWith,
            object: e::ObjectId::Comment(c.id),
        }
    }
}

impl From<BboxSubscription> for e::Triple {
    fn from(s: BboxSubscription) -> e::Triple {
        e::Triple {
            subject: e::ObjectId::User(s.user_id.unwrap()), //TODO
            predicate: e::Relation::SubscribedTo,
            object: e::ObjectId::BboxSubscription(s.id),
        }
    }
}

impl From<User> for e::User {
    fn from(u: User) -> e::User {
        let User {
            id,
            username,
            password,
            email,
            email_confirmed,
        } = u;
        e::User {
            id,
            username,
            password,
            email,
            email_confirmed,
        }
    }
}

impl From<e::User> for User {
    fn from(u: e::User) -> User {
        let e::User {
            id,
            username,
            password,
            email,
            email_confirmed,
        } = u;
        User {
            id,
            username,
            password,
            email,
            email_confirmed,
        }
    }
}

impl From<Comment> for e::Comment {
    fn from(c: Comment) -> e::Comment {
        let Comment { id, created, text, .. } = c;
        e::Comment {
            id,
            created: created as u64,
            text,
        }
    }
}

impl From<e::Comment> for Comment {
    fn from(c: e::Comment) -> Comment {
        let e::Comment { id, created, text } = c;
        Comment {
            id,
            created: created as i32,
            text,
            rating_id: None,
        }
    }
}

impl From<Rating> for e::Rating {
    fn from(r: Rating) -> e::Rating {
        let Rating {
            id,
            created,
            title,
            context,
            value,
            source,
            ..
        } = r;
        e::Rating {
            id,
            created: created as u64,
            title,
            value: value as i8,
            context: context.parse().unwrap(),
            source,
        }
    }
}

impl From<e::Rating> for Rating {
    fn from(r: e::Rating) -> Rating {
        let e::Rating {
            id,
            created,
            title,
            context,
            value,
            source,
        } = r;
        Rating {
            id,
            created: created as i32,
            title,
            value: value as i32,
            context: context.into(),
            source,
            entry_id: None,
        }
    }
}

impl From<BboxSubscription> for e::BboxSubscription {
    fn from(s: BboxSubscription) -> e::BboxSubscription {
        let BboxSubscription {
            id,
            south_west_lat,
            south_west_lng,
            north_east_lat,
            north_east_lng,
            ..
        } = s;
        e::BboxSubscription {
            id,
            south_west_lat: south_west_lat as f64,
            south_west_lng: south_west_lng as f64,
            north_east_lat: north_east_lat as f64,
            north_east_lng: north_east_lng as f64,
        }
    }
}

impl From<e::BboxSubscription> for BboxSubscription {
    fn from(s: e::BboxSubscription) -> BboxSubscription {
        let e::BboxSubscription {
            id,
            south_west_lat,
            south_west_lng,
            north_east_lat,
            north_east_lng,
        } = s;
        BboxSubscription {
            id,
            south_west_lat: south_west_lat as f32,
            south_west_lng: south_west_lng as f32,
            north_east_lat: north_east_lat as f32,
            north_east_lng: north_east_lng as f32,
            user_id: None,
        }
    }
}


impl From<e::RatingContext> for String {
    fn from(context: e::RatingContext) -> String {
        match context {
            e::RatingContext::Diversity => "diversity",
            e::RatingContext::Renewable => "renewable",
            e::RatingContext::Fairness => "fairness",
            e::RatingContext::Humanity => "humanity",
            e::RatingContext::Transparency => "transparency",
            e::RatingContext::Solidarity => "solidarity",
        }.into()
    }
}

impl From<e::Relation> for String {
    fn from(r: e::Relation) -> String {
        match r {
            e::Relation::IsTaggedWith => "is_tagged_with",
            e::Relation::IsRatedWith => "is_rated_with",
            e::Relation::IsCommentedWith => "is_commented_with",
            e::Relation::CreatedBy => "created_by",
            e::Relation::SubscribedTo => "subscribed_to",
        }.into()
    }
}

impl FromStr for e::Relation {
    type Err = String;
    fn from_str(predicate: &str) -> Result<e::Relation, String> {
        Ok(match predicate {
            "is_tagged_with" => e::Relation::IsTaggedWith,
            "is_rated_with" => e::Relation::IsRatedWith,
            "is_commented_with" => e::Relation::IsCommentedWith,
            "created_by" => e::Relation::CreatedBy,
            "subscribed_to" => e::Relation::SubscribedTo,
            _ => {
                return Err(format!("invalid Relation: '{}'", predicate));
            }
        })
    }
}

impl FromStr for e::RatingContext {
    type Err = String;
    fn from_str(context: &str) -> Result<e::RatingContext, String> {
        Ok(match context {
            "diversity" => e::RatingContext::Diversity,
            "renewable" => e::RatingContext::Renewable,
            "fairness" => e::RatingContext::Fairness,
            "humanity" => e::RatingContext::Humanity,
            "transparency" => e::RatingContext::Transparency,
            "solidarity" => e::RatingContext::Solidarity,
            _ => {
                return Err(format!("invalid RatingContext: '{}'", context));
            }
        })
    }
}
