use crate::id::Id;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Category {
    pub id: Id,
    pub tag: String,
}

impl Category {
    pub fn name(&self) -> String {
        format!("#{}", self.tag)
    }
}

impl Category {
    pub const ID_NON_PROFIT: &'static str = "2cd00bebec0c48ba9db761da48678134";
    pub const ID_COMMERCIAL: &'static str = "77b3c33a92554bcf8e8c2c86cedd6f6f";
    pub const ID_EVENT: &'static str = "c2dc278a2d6a4b9b8a50cb606fc017ed";

    pub const TAG_NON_PROFIT: &'static str = "non-profit";
    pub const TAG_COMMERCIAL: &'static str = "commercial";
    pub const TAG_EVENT: &'static str = "event";

    pub fn new_non_profit() -> Self {
        Self {
            id: Self::ID_NON_PROFIT.into(),
            tag: Self::TAG_NON_PROFIT.into(),
        }
    }

    pub fn new_commercial() -> Self {
        Self {
            id: Self::ID_COMMERCIAL.into(),
            tag: Self::TAG_COMMERCIAL.into(),
        }
    }

    pub fn new_event() -> Self {
        Self {
            id: Self::ID_EVENT.into(),
            tag: Self::TAG_EVENT.into(),
        }
    }

    pub fn split_from_tags(tags: Vec<String>) -> (Vec<String>, Vec<Category>) {
        let mut categories = Vec::with_capacity(3);
        let tags = tags
            .into_iter()
            .filter(|t| match t.as_str() {
                Self::TAG_NON_PROFIT => {
                    categories.push(Self::new_non_profit());
                    false
                }
                Self::TAG_COMMERCIAL => {
                    categories.push(Self::new_commercial());
                    false
                }
                Self::TAG_EVENT => {
                    categories.push(Self::new_event());
                    false
                }
                _ => true,
            })
            .collect();
        (tags, categories)
    }

    pub fn merge_ids_into_tags(ids: &[Id], mut tags: Vec<String>) -> Vec<String> {
        tags.reserve(ids.len());
        tags = ids.iter().fold(tags, |mut tags, id| {
            match id.as_ref() {
                Self::ID_NON_PROFIT => tags.push(Self::TAG_NON_PROFIT.into()),
                Self::ID_COMMERCIAL => tags.push(Self::TAG_COMMERCIAL.into()),
                Self::ID_EVENT => tags.push(Self::TAG_EVENT.into()),
                _ => (),
            }
            tags
        });
        tags.sort_unstable();
        tags.dedup();
        tags
    }
}
