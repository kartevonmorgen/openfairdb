ALTER TABLE users ADD COLUMN role INT DEFAULT 0 NOT NULL;
UPDATE users SET role = 1 WHERE email_confirmed = 1;
