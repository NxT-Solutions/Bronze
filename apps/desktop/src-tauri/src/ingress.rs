//! Linked ingress seqlock (story 3.4, CAP-004).

#[cfg(all(target_os = "macos", bronze_native_linked))]
mod linked {
    use bronze_platform_macos::{BronzeIngressSnapshot, NativeError, NativeRuntime};

    #[test]
    fn ingress_inconsistent_read_is_context_unavailable() {
        let _guard = crate::lock_native_runtime();
        let runtime = NativeRuntime::start().expect("start");
        let snap = BronzeIngressSnapshot {
            target_pid: 4242,
            bundle_token: 7,
            activation_generation: 3,
            destination_uuid: [9; 16],
            accept_capture_generation: 2,
            policy_revision: 11,
            settings_revision: 12,
            context_generation: 5,
            route: 1,
            monotonic_time_ns: 99,
        };
        runtime.ingress_publish(snap).expect("publish");
        let loaded = runtime.ingress_load().expect("load");
        assert_eq!(loaded.target_pid, 4242);
        assert_eq!(loaded.destination_uuid, [9; 16]);
        runtime
            .ingress_test_begin_inconsistent()
            .expect("begin torn");
        assert_eq!(
            runtime.ingress_load().unwrap_err(),
            NativeError::ContextUnavailable
        );
        runtime.ingress_test_end_inconsistent().expect("end torn");
        runtime.shutdown().expect("shutdown");
    }
}
