CREATE TABLE accounts (
  id uuid PRIMARY KEY,
  google_subject_hmac text NOT NULL UNIQUE,
  created_at timestamptz NOT NULL DEFAULT now(),
  last_login_at timestamptz NOT NULL DEFAULT now()
);

ALTER TABLE devices
  ADD COLUMN account_id uuid REFERENCES accounts(id) ON DELETE RESTRICT,
  ALTER COLUMN age_scope_version DROP NOT NULL,
  ALTER COLUMN age_declared_at DROP NOT NULL;

CREATE INDEX devices_account_idx ON devices (account_id);
