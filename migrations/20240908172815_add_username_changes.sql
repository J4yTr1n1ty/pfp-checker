-- Add migration script here
CREATE TABLE UsernameChange (
  userId INTEGER,
  changedAt INTEGER,
  username TEXT,
  FOREIGN KEY(userId) REFERENCES User(discordId) ON DELETE CASCADE
)
