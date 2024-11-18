-- Step 1: Create a new table with the desired primary key
CREATE TABLE ProfilePicture_new (
  checksum TEXT,
  userId INTEGER,
  changedAt INTEGER,
  link TEXT,
  PRIMARY KEY(checksum, changedAt, userId),
  FOREIGN KEY(userId) REFERENCES User(discordId) ON DELETE CASCADE
);

-- Step 2: Copy the data from the old table to the new table
INSERT INTO ProfilePicture_new (checksum, userId, changedAt, link)
SELECT checksum, userId, changedAt, link
FROM ProfilePicture;

-- Step 3: Drop the old table
DROP TABLE ProfilePicture;

-- Step 4: Rename the new table to the original table name
ALTER TABLE ProfilePicture_new RENAME TO ProfilePicture;

-- Step 5: Verify data migration
DECLARE
  old_count INTEGER;
  new_count INTEGER;
BEGIN
  SELECT COUNT(*) INTO old_count FROM ProfilePicture;
  SELECT COUNT(*) INTO new_count FROM ProfilePicture_new;
  IF old_count != new_count THEN
    RAISE EXCEPTION 'Data migration verification failed: count mismatch';
  END IF;
