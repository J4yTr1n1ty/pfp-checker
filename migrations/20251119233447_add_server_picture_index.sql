-- Add index for efficient ServerPicture queries by serverId
-- This supports queries like: SELECT ... WHERE serverId = ? ORDER BY changedAt DESC
CREATE INDEX IF NOT EXISTS idx_ServerPicture_serverId_changedAt
ON ServerPicture(serverId, changedAt DESC);
