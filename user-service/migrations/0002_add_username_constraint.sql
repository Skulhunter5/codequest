ALTER TABLE users
ADD CONSTRAINT CHK_username CHECK (username SIMILAR TO '[a-zA-Z0-9]+');
