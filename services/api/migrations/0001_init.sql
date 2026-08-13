CREATE TABLE invites (
  id uuid PRIMARY KEY,
  code_hash text NOT NULL,
  max_redemptions integer NOT NULL CHECK (max_redemptions > 0),
  redeemed_count integer NOT NULL DEFAULT 0 CHECK (redeemed_count >= 0),
  expires_at timestamptz NOT NULL,
  created_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE devices (
  id uuid PRIMARY KEY,
  public_id_hash text NOT NULL UNIQUE,
  status text NOT NULL CHECK (status IN ('active', 'revoked')),
  app_version text NOT NULL,
  platform text NOT NULL,
  age_scope_version text NOT NULL,
  age_declared_at timestamptz NOT NULL,
  created_at timestamptz NOT NULL DEFAULT now(),
  last_seen_at timestamptz,
  revoked_at timestamptz
);

CREATE TABLE device_tokens (
  id uuid PRIMARY KEY,
  device_id uuid NOT NULL REFERENCES devices(id) ON DELETE CASCADE,
  token_hmac text NOT NULL UNIQUE,
  expires_at timestamptz NOT NULL,
  created_at timestamptz NOT NULL DEFAULT now(),
  revoked_at timestamptz,
  replaced_by uuid REFERENCES device_tokens(id)
);

CREATE TABLE agent_runs (
  id uuid PRIMARY KEY,
  device_id uuid NOT NULL REFERENCES devices(id) ON DELETE CASCADE,
  provider_response_id_encrypted bytea NOT NULL,
  status text NOT NULL CHECK (status IN ('active', 'completed', 'stopped')),
  turn_count integer NOT NULL DEFAULT 0,
  action_count integer NOT NULL DEFAULT 0,
  expires_at timestamptz NOT NULL,
  created_at timestamptz NOT NULL DEFAULT now(),
  stopped_at timestamptz,
  last_idempotency_key text,
  last_response bytea
);

CREATE TABLE usage_daily (
  device_id uuid NOT NULL REFERENCES devices(id) ON DELETE CASCADE,
  usage_date date NOT NULL,
  realtime_seconds integer NOT NULL DEFAULT 0,
  screenshots integer NOT NULL DEFAULT 0,
  agent_turns integer NOT NULL DEFAULT 0,
  updated_at timestamptz NOT NULL DEFAULT now(),
  PRIMARY KEY (device_id, usage_date)
);

CREATE TABLE audit_events (
  id uuid PRIMARY KEY,
  device_id uuid REFERENCES devices(id) ON DELETE SET NULL,
  event_type text NOT NULL,
  reason_code text,
  request_id text NOT NULL,
  created_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX device_tokens_active_idx ON device_tokens (token_hmac, expires_at) WHERE revoked_at IS NULL;
CREATE INDEX agent_runs_expiry_idx ON agent_runs (expires_at) WHERE status = 'active';
CREATE INDEX audit_events_created_idx ON audit_events (created_at);
