CREATE TABLE IF NOT EXISTS app_users (
  discord_id bigint NOT NULL,
  exp bigint NOT NULL DEFAULT 0,
  PRIMARY KEY (discord_id)
)