#[cfg(target_os = "macos")]
mod macos {
    use std::sync::Arc;

    use contracts::{ElementOperationKind, UiState};
    use desktop_lib::{
        platform::macos_computer_use::{
            NativeActionError, execute_native_operation, validate_native_operation,
        },
        services::{
            application::{ApplicationBackend, PlatformApplicationBackend},
            capture::{CaptureBackend, XcapCaptureBackend},
            observation::{ObservationBackend, ObservationMode, PlatformObservationBackend},
        },
    };

    #[test]
    #[ignore = "run only on the isolated messaging fixture with Accessibility permission"]
    fn messaging_fixture_source_ax_native_and_stale_smoke() {
        assert_eq!(
            std::env::var("TRO_NATIVE_SMOKE_CONFIRM").as_deref(),
            Ok("messaging-client-fixture-only"),
            "explicitly confirm the isolated fixture before native input",
        );
        let app_name = std::env::var("TRO_NATIVE_SMOKE_APP")
            .expect("set TRO_NATIVE_SMOKE_APP to the dedicated fixture browser app name");
        let applications: Arc<dyn ApplicationBackend> = Arc::new(PlatformApplicationBackend);
        let source = applications
            .focused_application()
            .expect("focused app resolves")
            .expect("fixture browser is focused");
        assert_eq!(source.display_name, app_name);

        let capture: Arc<dyn CaptureBackend> = Arc::new(XcapCaptureBackend);
        let observer = PlatformObservationBackend::new(capture);
        let observation = observer
            .observe(&source, ObservationMode::Full)
            .expect("fixture observation succeeds");
        assert!(
            !observation.metadata.elements.is_empty(),
            "AX element count must be nonzero",
        );

        let composer = observation
            .metadata
            .elements
            .iter()
            .find(|element| {
                element
                    .name
                    .as_ref()
                    .is_some_and(|name| name.expose().eq_ignore_ascii_case("Message composer"))
                    && element.operations.contains(&ElementOperationKind::SetValue)
            })
            .expect("accessible fixture composer");
        let composer_resolved = observation
            .registry
            .resolve(
                &observation.metadata.binding.observation_id,
                &composer.element_id,
            )
            .expect("composer locator resolves");
        assert!(!composer_resolved.states.contains(&UiState::Secure));
        let composer_locator = composer_resolved
            .native_locator
            .as_ref()
            .expect("composer has native locator");
        validate_native_operation(composer_locator, ElementOperationKind::Focus)
            .expect("focus is supported");
        execute_native_operation(composer_locator, ElementOperationKind::Focus, None)
            .expect("native focus executes");
        execute_native_operation(
            composer_locator,
            ElementOperationKind::SetValue,
            Some("Tro native fixture draft"),
        )
        .expect("native set-value executes");

        let probe =
            observation
                .metadata
                .elements
                .iter()
                .find(|element| {
                    element.name.as_ref().is_some_and(|name| {
                        name.expose().eq_ignore_ascii_case("Native action probe")
                    }) && element.operations.contains(&ElementOperationKind::Invoke)
                })
                .expect("accessible native probe");
        let probe_locator = observation
            .registry
            .resolve(
                &observation.metadata.binding.observation_id,
                &probe.element_id,
            )
            .expect("probe locator resolves")
            .native_locator
            .as_ref()
            .expect("probe has native locator");
        execute_native_operation(probe_locator, ElementOperationKind::Invoke, None)
            .expect("native press executes");

        let mut stale_locator = probe_locator.clone();
        stale_locator.bounds_fingerprint ^= 1;
        assert_eq!(
            execute_native_operation(&stale_locator, ElementOperationKind::Invoke, None),
            Err(NativeActionError::Stale),
            "a stale locator must produce zero input",
        );

        if let Some(secure) = observation
            .metadata
            .elements
            .iter()
            .find(|element| element.states.contains(&UiState::Secure))
        {
            let secure_locator = observation
                .registry
                .resolve(
                    &observation.metadata.binding.observation_id,
                    &secure.element_id,
                )
                .expect("secure locator resolves")
                .native_locator
                .as_ref()
                .expect("secure field has native locator");
            assert_eq!(
                validate_native_operation(secure_locator, ElementOperationKind::SetValue),
                Err(NativeActionError::PermissionDenied),
            );
        }
    }
}

#[cfg(not(target_os = "macos"))]
#[test]
#[ignore = "macOS AX tranche only; Windows UIA remains disabled"]
fn native_capability_smoke_is_disabled_off_macos() {}
