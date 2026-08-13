use std::{collections::HashMap, sync::Mutex};

use async_trait::async_trait;
use sqlx::{PgPool, Row};
use time::{Duration, OffsetDateTime};
use uuid::Uuid;

use super::{AgentRunRecord, RunStatus, UsageSnapshot};

#[derive(Clone, Debug)]
pub struct InviteRecord {
    pub id: Uuid,
    pub code_hash: String,
}

#[async_trait]
pub trait Repository: Send + Sync {
    async fn active_invites(&self) -> Result<Vec<InviteRecord>, sqlx::Error>;
    async fn redeem_invite_and_create_device(
        &self,
        invite_id: Uuid,
        public_id_hash: &str,
        app_version: &str,
        platform: &str,
    ) -> Result<Option<Uuid>, sqlx::Error>;
    async fn store_device_token(
        &self,
        device_id: Uuid,
        token_hmac: &str,
        expires_at: OffsetDateTime,
    ) -> Result<(), sqlx::Error>;
    async fn rotate_device_token(
        &self,
        old_token_hmac: &str,
        device_id: Uuid,
        new_token_hmac: &str,
        new_expires_at: OffsetDateTime,
        old_expires_at: OffsetDateTime,
    ) -> Result<(), sqlx::Error>;
    async fn device_for_token(&self, token_hmac: &str) -> Result<Option<Uuid>, sqlx::Error>;
    async fn create_run(&self, run: AgentRunRecord) -> Result<(), sqlx::Error>;
    async fn get_run(&self, run_id: Uuid) -> Result<Option<AgentRunRecord>, sqlx::Error>;
    async fn update_run(&self, run: AgentRunRecord) -> Result<(), sqlx::Error>;
    async fn usage_today(&self, device_id: Uuid) -> Result<UsageSnapshot, sqlx::Error>;
    async fn reserve_tutor_usage(
        &self,
        device_id: Uuid,
        screenshot_limit: u32,
    ) -> Result<bool, sqlx::Error>;
    async fn increment_agent_usage(&self, device_id: Uuid) -> Result<(), sqlx::Error>;
}

#[derive(Default)]
pub struct MemoryRepository {
    invites: Mutex<Vec<(InviteRecord, u32, u32, OffsetDateTime)>>,
    tokens: Mutex<HashMap<String, (Uuid, OffsetDateTime)>>,
    runs: Mutex<HashMap<Uuid, AgentRunRecord>>,
    usage: Mutex<HashMap<Uuid, UsageSnapshot>>,
}

impl MemoryRepository {
    pub fn seed_invite_hash(&self, code_hash: String, max_redemptions: u32) -> Uuid {
        let id = Uuid::new_v4();
        let record = InviteRecord { id, code_hash };
        self.invites.lock().expect("test repository mutex").push((
            record,
            max_redemptions,
            0,
            OffsetDateTime::now_utc() + Duration::days(1),
        ));
        id
    }

    pub fn seed_device_token_digest(&self, digest: String) -> Uuid {
        let device_id = Uuid::new_v4();
        self.tokens
            .lock()
            .expect("development repository mutex")
            .insert(
                digest,
                (device_id, OffsetDateTime::now_utc() + Duration::days(30)),
            );
        device_id
    }
}

#[async_trait]
impl Repository for MemoryRepository {
    async fn active_invites(&self) -> Result<Vec<InviteRecord>, sqlx::Error> {
        let now = OffsetDateTime::now_utc();
        Ok(self
            .invites
            .lock()
            .expect("test repository mutex")
            .iter()
            .filter(|(_, max, used, expires)| used < max && *expires > now)
            .map(|(invite, _, _, _)| invite.clone())
            .collect())
    }

    async fn redeem_invite_and_create_device(
        &self,
        invite_id: Uuid,
        _public_id_hash: &str,
        _app_version: &str,
        _platform: &str,
    ) -> Result<Option<Uuid>, sqlx::Error> {
        let now = OffsetDateTime::now_utc();
        let mut invites = self.invites.lock().expect("test repository mutex");
        let Some((_, max, used, expires)) = invites
            .iter_mut()
            .find(|(invite, _, _, _)| invite.id == invite_id)
        else {
            return Ok(None);
        };
        if *used >= *max || *expires <= now {
            return Ok(None);
        }
        *used += 1;
        Ok(Some(Uuid::new_v4()))
    }

    async fn store_device_token(
        &self,
        device_id: Uuid,
        token_hmac: &str,
        expires_at: OffsetDateTime,
    ) -> Result<(), sqlx::Error> {
        self.tokens
            .lock()
            .expect("test repository mutex")
            .insert(token_hmac.to_owned(), (device_id, expires_at));
        Ok(())
    }

    async fn device_for_token(&self, token_hmac: &str) -> Result<Option<Uuid>, sqlx::Error> {
        let now = OffsetDateTime::now_utc();
        Ok(self
            .tokens
            .lock()
            .expect("test repository mutex")
            .get(token_hmac)
            .filter(|(_, expires)| *expires > now)
            .map(|(device, _)| *device))
    }

    async fn rotate_device_token(
        &self,
        old_token_hmac: &str,
        device_id: Uuid,
        new_token_hmac: &str,
        new_expires_at: OffsetDateTime,
        old_expires_at: OffsetDateTime,
    ) -> Result<(), sqlx::Error> {
        let mut tokens = self.tokens.lock().expect("test repository mutex");
        if let Some((_, expiry)) = tokens.get_mut(old_token_hmac) {
            *expiry = old_expires_at;
        }
        tokens.insert(new_token_hmac.to_owned(), (device_id, new_expires_at));
        Ok(())
    }

    async fn create_run(&self, run: AgentRunRecord) -> Result<(), sqlx::Error> {
        self.runs
            .lock()
            .expect("test repository mutex")
            .insert(run.id, run);
        Ok(())
    }

    async fn get_run(&self, run_id: Uuid) -> Result<Option<AgentRunRecord>, sqlx::Error> {
        Ok(self
            .runs
            .lock()
            .expect("test repository mutex")
            .get(&run_id)
            .cloned())
    }

    async fn update_run(&self, run: AgentRunRecord) -> Result<(), sqlx::Error> {
        self.runs
            .lock()
            .expect("test repository mutex")
            .insert(run.id, run);
        Ok(())
    }

    async fn usage_today(&self, device_id: Uuid) -> Result<UsageSnapshot, sqlx::Error> {
        Ok(self
            .usage
            .lock()
            .expect("test repository mutex")
            .get(&device_id)
            .copied()
            .unwrap_or_default())
    }

    async fn increment_agent_usage(&self, device_id: Uuid) -> Result<(), sqlx::Error> {
        let mut usage = self.usage.lock().expect("test repository mutex");
        let entry = usage.entry(device_id).or_default();
        entry.agent_turns = entry.agent_turns.saturating_add(1);
        entry.screenshots = entry.screenshots.saturating_add(1);
        Ok(())
    }

    async fn reserve_tutor_usage(
        &self,
        device_id: Uuid,
        screenshot_limit: u32,
    ) -> Result<bool, sqlx::Error> {
        let mut usage = self.usage.lock().expect("test repository mutex");
        let entry = usage.entry(device_id).or_default();
        if entry.screenshots >= screenshot_limit {
            return Ok(false);
        }
        entry.screenshots = entry.screenshots.saturating_add(1);
        Ok(true)
    }
}

pub struct PgRepository {
    pool: PgPool,
}

impl PgRepository {
    pub const fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl Repository for PgRepository {
    async fn active_invites(&self) -> Result<Vec<InviteRecord>, sqlx::Error> {
        let rows = sqlx::query(
            "SELECT id, code_hash FROM invites WHERE expires_at > now() AND redeemed_count < max_redemptions LIMIT 100",
        )
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter()
            .map(|row| {
                Ok(InviteRecord {
                    id: row.try_get("id")?,
                    code_hash: row.try_get("code_hash")?,
                })
            })
            .collect()
    }

    async fn redeem_invite_and_create_device(
        &self,
        invite_id: Uuid,
        public_id_hash: &str,
        app_version: &str,
        platform: &str,
    ) -> Result<Option<Uuid>, sqlx::Error> {
        let mut transaction = self.pool.begin().await?;
        let redeemed = sqlx::query(
            "UPDATE invites SET redeemed_count = redeemed_count + 1 WHERE id = $1 AND expires_at > now() AND redeemed_count < max_redemptions RETURNING id",
        )
        .bind(invite_id)
        .fetch_optional(&mut *transaction)
        .await?;
        if redeemed.is_none() {
            transaction.rollback().await?;
            return Ok(None);
        }
        let device_id = Uuid::new_v4();
        sqlx::query("INSERT INTO devices (id, public_id_hash, status, app_version, platform, age_scope_version, age_declared_at) VALUES ($1, $2, 'active', $3, $4, 'university_18_plus_v1', now())")
            .bind(device_id)
            .bind(public_id_hash)
            .bind(app_version)
            .bind(platform)
            .execute(&mut *transaction)
            .await?;
        transaction.commit().await?;
        Ok(Some(device_id))
    }

    async fn store_device_token(
        &self,
        device_id: Uuid,
        token_hmac: &str,
        expires_at: OffsetDateTime,
    ) -> Result<(), sqlx::Error> {
        sqlx::query("INSERT INTO device_tokens (id, device_id, token_hmac, expires_at) VALUES ($1, $2, $3, $4)")
            .bind(Uuid::new_v4())
            .bind(device_id)
            .bind(token_hmac)
            .bind(expires_at)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn device_for_token(&self, token_hmac: &str) -> Result<Option<Uuid>, sqlx::Error> {
        sqlx::query_scalar("SELECT d.id FROM device_tokens t JOIN devices d ON d.id = t.device_id WHERE t.token_hmac = $1 AND t.revoked_at IS NULL AND t.expires_at > now() AND d.status = 'active' AND d.revoked_at IS NULL")
            .bind(token_hmac)
            .fetch_optional(&self.pool)
            .await
    }

    async fn rotate_device_token(
        &self,
        old_token_hmac: &str,
        device_id: Uuid,
        new_token_hmac: &str,
        new_expires_at: OffsetDateTime,
        old_expires_at: OffsetDateTime,
    ) -> Result<(), sqlx::Error> {
        let mut transaction = self.pool.begin().await?;
        let replacement_id = Uuid::new_v4();
        sqlx::query("INSERT INTO device_tokens (id, device_id, token_hmac, expires_at) VALUES ($1, $2, $3, $4)")
            .bind(replacement_id)
            .bind(device_id)
            .bind(new_token_hmac)
            .bind(new_expires_at)
            .execute(&mut *transaction)
            .await?;
        sqlx::query("UPDATE device_tokens SET expires_at = LEAST(expires_at, $2), replaced_by = $3 WHERE token_hmac = $1 AND device_id = $4 AND revoked_at IS NULL")
            .bind(old_token_hmac)
            .bind(old_expires_at)
            .bind(replacement_id)
            .bind(device_id)
            .execute(&mut *transaction)
            .await?;
        transaction.commit().await?;
        Ok(())
    }

    async fn create_run(&self, run: AgentRunRecord) -> Result<(), sqlx::Error> {
        sqlx::query("INSERT INTO agent_runs (id, device_id, provider_response_id_encrypted, status, turn_count, action_count, expires_at) VALUES ($1, $2, $3, $4, $5, $6, $7)")
            .bind(run.id)
            .bind(run.device_id)
            .bind(run.continuation_encrypted)
            .bind(run.status.as_str())
            .bind(i32::try_from(run.turn_count).unwrap_or(i32::MAX))
            .bind(i32::try_from(run.action_count).unwrap_or(i32::MAX))
            .bind(run.expires_at)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn get_run(&self, run_id: Uuid) -> Result<Option<AgentRunRecord>, sqlx::Error> {
        let row = sqlx::query("SELECT id, device_id, provider_response_id_encrypted, status, turn_count, action_count, expires_at, last_idempotency_key, last_response FROM agent_runs WHERE id = $1")
            .bind(run_id)
            .fetch_optional(&self.pool)
            .await?;
        row.map(|row| {
            let status: String = row.try_get("status")?;
            Ok(AgentRunRecord {
                id: row.try_get("id")?,
                device_id: row.try_get("device_id")?,
                continuation_encrypted: row.try_get("provider_response_id_encrypted")?,
                status: match status.as_str() {
                    "completed" => RunStatus::Completed,
                    "stopped" => RunStatus::Stopped,
                    _ => RunStatus::Active,
                },
                turn_count: u32::try_from(row.try_get::<i32, _>("turn_count")?).unwrap_or_default(),
                action_count: u32::try_from(row.try_get::<i32, _>("action_count")?)
                    .unwrap_or_default(),
                expires_at: row.try_get("expires_at")?,
                last_idempotency_key: row.try_get("last_idempotency_key")?,
                last_response: row.try_get("last_response")?,
            })
        })
        .transpose()
    }

    async fn update_run(&self, run: AgentRunRecord) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE agent_runs SET provider_response_id_encrypted = $2, status = $3, turn_count = $4, action_count = $5, last_idempotency_key = $6, last_response = $7, stopped_at = CASE WHEN $3 = 'stopped' THEN now() ELSE stopped_at END WHERE id = $1")
            .bind(run.id)
            .bind(run.continuation_encrypted)
            .bind(run.status.as_str())
            .bind(i32::try_from(run.turn_count).unwrap_or(i32::MAX))
            .bind(i32::try_from(run.action_count).unwrap_or(i32::MAX))
            .bind(run.last_idempotency_key)
            .bind(run.last_response)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn usage_today(&self, device_id: Uuid) -> Result<UsageSnapshot, sqlx::Error> {
        let row = sqlx::query("SELECT realtime_seconds, screenshots, agent_turns FROM usage_daily WHERE device_id = $1 AND usage_date = current_date")
            .bind(device_id)
            .fetch_optional(&self.pool)
            .await?;
        row.map_or(Ok(UsageSnapshot::default()), |row| {
            Ok(UsageSnapshot {
                realtime_seconds: u32::try_from(row.try_get::<i32, _>("realtime_seconds")?)
                    .unwrap_or_default(),
                screenshots: u32::try_from(row.try_get::<i32, _>("screenshots")?)
                    .unwrap_or_default(),
                agent_turns: u32::try_from(row.try_get::<i32, _>("agent_turns")?)
                    .unwrap_or_default(),
            })
        })
    }

    async fn increment_agent_usage(&self, device_id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query("INSERT INTO usage_daily (device_id, usage_date, screenshots, agent_turns) VALUES ($1, current_date, 1, 1) ON CONFLICT (device_id, usage_date) DO UPDATE SET screenshots = usage_daily.screenshots + 1, agent_turns = usage_daily.agent_turns + 1, updated_at = now()")
            .bind(device_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn reserve_tutor_usage(
        &self,
        device_id: Uuid,
        screenshot_limit: u32,
    ) -> Result<bool, sqlx::Error> {
        let reserved = sqlx::query_scalar::<_, i32>("INSERT INTO usage_daily (device_id, usage_date, screenshots) SELECT $1, current_date, 1 WHERE $2 > 0 ON CONFLICT (device_id, usage_date) DO UPDATE SET screenshots = usage_daily.screenshots + 1, updated_at = now() WHERE usage_daily.screenshots < $2 RETURNING screenshots")
            .bind(device_id)
            .bind(i32::try_from(screenshot_limit).unwrap_or(i32::MAX))
            .fetch_optional(&self.pool)
            .await?;
        Ok(reserved.is_some())
    }
}
