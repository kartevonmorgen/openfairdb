.mode csv
.import entries.csv entries
.import old_entries.csv entries
.import tags.csv tags
.import categories.csv categories
.import ratings.csv ratings
.import comments.csv comments
.import users.csv users
.import bbox_subscriptions.csv bbox_subscriptions
.import entry_category_relations.csv entry_category_relations
.import entry_tag_relations.csv  entry_tag_relations

UPDATE entries SET osm_node  = NULL WHERE osm_node  = '';
UPDATE entries SET street    = NULL WHERE street    = '';
UPDATE entries SET zip       = NULL WHERE zip       = '';
UPDATE entries SET city      = NULL WHERE city      = '';
UPDATE entries SET country   = NULL WHERE country   = '';
UPDATE entries SET email     = NULL WHERE email     = '';
UPDATE entries SET telephone = NULL WHERE telephone = '';
UPDATE entries SET homepage  = NULL WHERE homepage  = '';
UPDATE entries SET license   = 'CC0-1.0' WHERE license = '';

UPDATE ratings SET source = NULL WHERE source = '';

DELETE FROM entry_tag_relations WHERE tag_id = '';
DELETE FROM entry_tag_relations WHERE entry_id = '';

DELETE FROM tags WHERE id = '';
