use std::{sync::Arc, time::Duration};

use contracts::{AppError, ApplicationRef, ErrorCode};
use tokio::time::{Instant, sleep};
use tokio_util::sync::CancellationToken;

use crate::services::{
    application::ApplicationBackend,
    observation::{Observation, ObservationBackend, ObservationMode},
    user_activity::{ActivitySnapshot, UserActivityBackend},
};

const POLL_INTERVAL: Duration = Duration::from_millis(100);
const EQUIVALENT_WINDOW: Duration = Duration::from_millis(200);
const ACTION_TIMEOUT: Duration = Duration::from_secs(5);
const ACTIVATION_TIMEOUT: Duration = Duration::from_secs(8);

pub struct Stabilizer {
    applications: Arc<dyn ApplicationBackend>,
    observer: Arc<dyn ObservationBackend>,
    activity: Arc<dyn UserActivityBackend>,
}

impl Stabilizer {
    pub fn new(
        applications: Arc<dyn ApplicationBackend>,
        observer: Arc<dyn ObservationBackend>,
        activity: Arc<dyn UserActivityBackend>,
    ) -> Self {
        Self {
            applications,
            observer,
            activity,
        }
    }

    pub fn activity_snapshot(&self) -> ActivitySnapshot {
        self.activity.snapshot()
    }

    pub fn ensure_no_takeover(
        &self,
        snapshot: ActivitySnapshot,
        cancellation: &CancellationToken,
    ) -> Result<(), AppError> {
        guard(cancellation, self.activity.as_ref(), snapshot)
    }

    pub async fn wait_for_takeover(
        &self,
        snapshot: ActivitySnapshot,
        cancellation: &CancellationToken,
    ) -> AppError {
        loop {
            if let Err(error) = guard(cancellation, self.activity.as_ref(), snapshot) {
                return error;
            }
            tokio::select! {
                () = cancellation.cancelled() => return cancelled(),
                () = sleep(Duration::from_millis(50)) => {}
            }
        }
    }

    pub async fn wait_for_activation(
        &self,
        app: &ApplicationRef,
        cancellation: &CancellationToken,
    ) -> Result<Observation, AppError> {
        let deadline = Instant::now() + ACTIVATION_TIMEOUT;
        let activity = self.activity.snapshot();
        loop {
            guard(cancellation, self.activity.as_ref(), activity)?;
            if self
                .applications
                .identity_state(&app.app_id)
                .is_ok_and(|state| state.focused && state.visible)
            {
                return self.observer.observe(app, ObservationMode::Full);
            }
            if Instant::now() >= deadline {
                return Err(AppError::new(
                    ErrorCode::TargetAppUnavailable,
                    "Ứng dụng chưa sẵn sàng. Hãy mở ứng dụng và thử lại.",
                    true,
                ));
            }
            tokio::select! {
                () = cancellation.cancelled() => return Err(cancelled()),
                () = sleep(POLL_INTERVAL) => {}
            }
        }
    }

    pub async fn wait_for_stable(
        &self,
        app: &ApplicationRef,
        previous: blake3::Hash,
        allow_no_change: bool,
        cancellation: &CancellationToken,
    ) -> Result<Observation, AppError> {
        let deadline = Instant::now() + ACTION_TIMEOUT;
        let activity = self.activity.snapshot();
        let mut change_seen = allow_no_change;
        let mut previous_sample: Option<(blake3::Hash, Instant)> = None;
        loop {
            guard(cancellation, self.activity.as_ref(), activity)?;
            let identity = self.applications.identity_state(&app.app_id)?;
            if !identity.focused || !identity.visible {
                return Err(AppError::new(
                    ErrorCode::UserTakeover,
                    "Ứng dụng khác đã nhận điều khiển — Tro đã tạm dừng.",
                    false,
                ));
            }
            let sample = self.observer.observe(app, ObservationMode::Lightweight)?;
            let digest = sample.digest();
            change_seen |= digest != previous;
            if change_seen
                && previous_sample
                    .as_ref()
                    .is_some_and(|(last, at)| *last == digest && at.elapsed() >= EQUIVALENT_WINDOW)
            {
                return self.observer.observe(app, ObservationMode::Full);
            }
            if previous_sample
                .as_ref()
                .is_none_or(|(last, _)| *last != digest)
            {
                previous_sample = Some((digest, Instant::now()));
            }
            if Instant::now() >= deadline {
                return Err(AppError::new(
                    ErrorCode::AgentTimeout,
                    "Ứng dụng chưa ổn định sau thao tác; Tro đã dừng để tránh bấm nhầm.",
                    true,
                ));
            }
            tokio::select! {
                () = cancellation.cancelled() => return Err(cancelled()),
                () = sleep(POLL_INTERVAL) => {}
            }
        }
    }
}

fn guard(
    cancellation: &CancellationToken,
    activity: &dyn UserActivityBackend,
    snapshot: ActivitySnapshot,
) -> Result<(), AppError> {
    if cancellation.is_cancelled() {
        return Err(cancelled());
    }
    if activity.changed_since(snapshot) {
        return Err(AppError::new(
            ErrorCode::UserTakeover,
            "Bạn đã tiếp quản — Tro đã tạm dừng.",
            false,
        ));
    }
    Ok(())
}

fn cancelled() -> AppError {
    AppError::new(ErrorCode::Cancelled, "Đã dừng computer use.", false)
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    #[test]
    fn stability_requires_two_samples_separated_by_a_real_window() {
        assert!(super::EQUIVALENT_WINDOW >= Duration::from_millis(200));
        assert!(super::POLL_INTERVAL <= Duration::from_millis(100));
    }
}
