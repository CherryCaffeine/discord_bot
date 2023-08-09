CREATE TABLE IF NOT EXISTS app_users (
  discord_id bigint NOT NULL,
  exp bigint NOT NULL DEFAULT 0,
  on_server boolean NOT NULL DEFAULT true,
  PRIMARY KEY (discord_id)
);

CREATE TABLE IF NOT EXISTS earned_roles (
  role_id bigint NOT NULL,
  exp_needed bigint NOT NULL,
  PRIMARY KEY (role_id)
);

CREATE TABLE IF NOT EXISTS self_assigned_roles (
  /* id of group of mutually exclusive roles */
  excl_role_group_id bigint NOT NULL,
  role_id bigint NOT NULL UNIQUE,
  message_id bigint NOT NULL,
  emoji_id bigint DEFAULT NULL,
  emoji_name varchar(255) DEFAULT NULL,
  /* CHECK ((emoji_id IS NOT NULL AND emoji_name is NULL) OR (emoji_id is NULL AND emoji_name IS NOT NULL)), */
  PRIMARY KEY (role_id)
);

CREATE INDEX temp_idx_exp_needed ON earned_roles (exp_needed);
CLUSTER earned_roles USING temp_idx_exp_needed;
DROP INDEX temp_idx_exp_needed;

CREATE INDEX temp_idx_message_id ON self_assigned_roles (message_id);
CLUSTER self_assigned_roles USING temp_idx_message_id;
DROP INDEX temp_idx_message_id;
