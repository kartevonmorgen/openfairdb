use super::*;

impl OrganizationRepo for Connection<'_> {
    fn create_org(&mut self, mut o: Organization) -> Result<()> {
        let org_id = o.id.clone();
        let moderated_tags = std::mem::take(&mut o.moderated_tags);
        let new_org = models::NewOrganization::from(o);
        self.transaction::<_, diesel::result::Error, _>(|| {
            diesel::insert_into(schema::organization::table)
                .values(&new_org)
                .execute(self.0)?;
            let org_rowid = resolve_organization_rowid(self, &org_id).map_err(|err| {
                log::warn!(
                    "Failed to resolve id of newly created organization '{}': {}",
                    org_id,
                    err
                );
                diesel::result::Error::RollbackTransaction
            })?;
            for ModeratedTag {
                label,
                allow_add,
                allow_remove,
                require_clearance,
            } in &moderated_tags
            {
                let org_tag = models::NewOrganizationTag {
                    org_rowid,
                    tag_label: label,
                    tag_allow_add: if *allow_add { 1 } else { 0 },
                    tag_allow_remove: if *allow_remove { 1 } else { 0 },
                    require_clearance: if *require_clearance { 1 } else { 0 },
                };
                diesel::insert_into(schema::organization_tag::table)
                    .values(&org_tag)
                    .execute(self.0)?;
            }
            Ok(())
        })
        .map_err(from_diesel_err)?;
        Ok(())
    }

    fn get_org_by_api_token(&self, token: &str) -> Result<Organization> {
        use schema::{organization::dsl as org_dsl, organization_tag::dsl as org_tag_dsl};

        let models::Organization {
            rowid,
            id,
            name,
            api_token,
        } = org_dsl::organization
            .filter(org_dsl::api_token.eq(token))
            .first(self.deref())
            .map_err(from_diesel_err)?;

        let moderated_tags = org_tag_dsl::organization_tag
            .filter(org_tag_dsl::org_rowid.eq(rowid))
            .load::<models::OrganizationTag>(self.deref())
            .map_err(from_diesel_err)?
            .into_iter()
            .map(Into::into)
            .collect();

        Ok(Organization {
            id: id.into(),
            name,
            api_token,
            moderated_tags,
        })
    }

    fn map_tag_to_clearance_org_id(&self, tag: &str) -> Result<Option<Id>> {
        use schema::{organization::dsl, organization_tag::dsl as tag_dsl};
        Ok(schema::organization::table
            .inner_join(schema::organization_tag::table)
            .select(dsl::id)
            .filter(tag_dsl::tag_label.eq(tag))
            .filter(tag_dsl::require_clearance.ne(0))
            .first::<String>(self.deref())
            .optional()
            .map_err(from_diesel_err)?
            .map(Into::into))
    }

    fn get_moderated_tags_by_org(
        &self,
        excluded_org_id: Option<&Id>,
    ) -> Result<Vec<(Id, ModeratedTag)>> {
        use schema::{organization::dsl as org_dsl, organization_tag::dsl as org_tag_dsl};
        let query = org_tag_dsl::organization_tag
            .inner_join(org_dsl::organization)
            .select((
                org_dsl::id,
                org_tag_dsl::tag_label,
                org_tag_dsl::tag_allow_add,
                org_tag_dsl::tag_allow_remove,
                org_tag_dsl::require_clearance,
            ))
            .order_by(org_dsl::id);
        let moderated_tags = if let Some(excluded_org_id) = excluded_org_id {
            query
                .filter(org_dsl::id.ne(excluded_org_id.as_str()))
                .load::<models::OrganizationTagWithId>(self.deref())
                .map_err(from_diesel_err)?
        } else {
            query
                .load::<models::OrganizationTagWithId>(self.deref())
                .map_err(from_diesel_err)?
        };
        Ok(moderated_tags.into_iter().map(Into::into).collect())
    }
}
