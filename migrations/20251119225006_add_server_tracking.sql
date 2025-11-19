-- Add Server tracking tables
CREATE TABLE Server (
  serverId INTEGER,
  trackedSince INTEGER,
  PRIMARY KEY(serverId)
);

CREATE TABLE ServerPicture (
  checksum TEXT,
  serverId INTEGER,
  changedAt INTEGER,
  link TEXT,
  PRIMARY KEY(checksum, changedAt, serverId),
  FOREIGN KEY(serverId) REFERENCES Server(serverId) ON DELETE CASCADE
);
