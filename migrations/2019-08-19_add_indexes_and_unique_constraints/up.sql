-- Add missing indexes for secondary primary key columns
CREATE INDEX event_tag_relations_fk_tag_id ON event_tag_relations (tag_id);
CREATE INDEX org_tag_relations_fk_tag_id ON org_tag_relations (tag_id);

-- Organizations must be identifiable by their unique API token
CREATE UNIQUE INDEX organizations_api_token ON organizations (api_token);
