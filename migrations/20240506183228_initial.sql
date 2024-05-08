-- Add migration script here
CREATE TABLE User (
  discordId INTEGER,
  trackedSince INTEGER,
  PRIMARY KEY(discordId)
);

CREATE TABLE ProfilePicture (
  checksum TEXT,
  userId INTEGER,
  changedAt INTEGER,
  link TEXT,
  PRIMARY KEY(checksum, userId),
  FOREIGN KEY(userId) REFERENCES User(discordId) ON DELETE CASCADE
);
