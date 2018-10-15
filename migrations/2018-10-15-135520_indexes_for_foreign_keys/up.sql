-- Create missing indexes for foreign key relations

CREATE INDEX entry_category_relations_fk_category_id ON entry_category_relations (category_id);

CREATE INDEX entry_tag_relations_fk_tag_id ON entry_tag_relations (tag_id);

CREATE INDEX comments_fk_rating_id ON comments (rating_id);

CREATE INDEX bbox_subscriptions_fk_username ON bbox_subscriptions (username);
