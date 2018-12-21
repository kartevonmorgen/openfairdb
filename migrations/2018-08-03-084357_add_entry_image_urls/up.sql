-- Add image and image link URLs to entries
-- https://github.com/slowtec/openfairdb/issues/99
ALTER TABLE entries ADD COLUMN image_url TEXT;
ALTER TABLE entries ADD COLUMN image_link_url TEXT;
