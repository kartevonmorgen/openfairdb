-- Add image and image link URLs to events
ALTER TABLE events ADD COLUMN image_url TEXT;
ALTER TABLE events ADD COLUMN image_link_url TEXT;
