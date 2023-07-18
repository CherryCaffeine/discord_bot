CREATE TABLE IF NOT EXISTS app_users (
  discord_id bigint NOT NULL,
  exp bigint NOT NULL DEFAULT 0,
  on_server boolean NOT NULL DEFAULT true,
  PRIMARY KEY (discord_id)
)