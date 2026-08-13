#[test]
#[ignore = "run only on a supervised signed OS fixture"]
fn native_capability_smoke_requires_supervision() {
    let fixture = std::env::var("TRO_NATIVE_SMOKE_FIXTURE")
        .expect("set TRO_NATIVE_SMOKE_FIXTURE to the dedicated fixture window title");
    assert!(!fixture.trim().is_empty());
}
