-- Add migration script here
CREATE TABLE User (
  discordId INTEGER,
  trackedSince INTEGER,
  PRIMARY KEY(discordId)
);

CREATE TABLE ProfilePicture (
  id INTEGER,
  userId INTEGER,
  link TEXT,
  hash TEXT,
  PRIMARY KEY(id),
  FOREIGN KEY(userId) REFERENCES User(discordId)
);
