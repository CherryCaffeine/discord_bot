CREATE TABLE IF NOT EXISTS app_users (
  discord_id bigint NOT NULL,
  exp bigint NOT NULL DEFAULT 0,
  on_server boolean NOT NULL DEFAULT true,
  PRIMARY KEY (discord_id)
);

CREATE TABLE IF NOT EXISTS exp_based_roles (
  role_id bigint NOT NULL,
  exp_needed bigint NOT NULL,
  PRIMARY KEY (role_id)
);

CREATE INDEX temp_index_exp_needed ON exp_based_roles (exp_needed);
CLUSTER exp_based_roles USING temp_index_exp_needed;
DROP INDEX temp_index_exp_needed;
