ALTER TABLE users
ADD CONSTRAINT CHK_username CHECK ((username SIMILAR TO '[a-zA-Z0-9 ]{1,30}') AND (username NOT LIKE '%  %') AND (username = TRIM(username)));
