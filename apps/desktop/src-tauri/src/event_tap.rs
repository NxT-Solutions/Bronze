//! Linked listen-only event tap (story 3.3, CAP-002, CAP-004).

#[cfg(all(target_os = "macos", bronze_native_linked))]
mod linked {
    use bronze_platform_macos::{
        event_tap_enqueue_raw, EventTapHealth, NativeRuntime, BRONZE_STATUS_NOT_FOUND,
        BRONZE_TAP_FEED_DOWN, BRONZE_TAP_REC_DISABLED, BRONZE_TAP_REC_RESET,
    };

    #[test]
    fn listen_only_denial_degrades_and_disable_resets_fsm() {
        let _guard = crate::lock_native_runtime();
        let runtime = NativeRuntime::start().expect("start");
        let health = runtime.event_tap_start().expect("tap start");
        assert!(
            health == EventTapHealth::Degraded || health == EventTapHealth::Listening,
            "start must listen or degrade without prompting: {health:?}"
        );
        runtime.event_tap_stop().expect("stop");
        runtime.event_tap_test_attach().expect("attach");
        runtime.event_tap_set_enabled(true).expect("enable");
        runtime
            .event_tap_test_feed(BRONZE_TAP_FEED_DOWN, 56, 1_000)
            .expect("down");
        assert_eq!(runtime.event_tap_fsm_state().expect("firstDown"), 1);
        runtime.event_tap_test_disable().expect("disable");
        assert_eq!(runtime.event_tap_fsm_state().expect("idle"), 0);
        let rec = runtime.event_tap_drain().expect("drain").expect("record");
        assert_eq!(rec.kind, BRONZE_TAP_REC_DISABLED);
        runtime.event_tap_stop().expect("stop tap");
        runtime.shutdown().expect("shutdown");
    }

    #[test]
    fn spsc_rejects_foreign_producer() {
        let _guard = crate::lock_native_runtime();
        let runtime = NativeRuntime::start().expect("start");
        runtime.event_tap_test_attach().expect("attach");
        runtime
            .event_tap_test_enqueue_from_caller(BRONZE_TAP_REC_RESET)
            .expect("bound producer");
        let status = std::thread::spawn(|| event_tap_enqueue_raw(BRONZE_TAP_REC_RESET))
            .join()
            .expect("join");
        assert_eq!(status, BRONZE_STATUS_NOT_FOUND);
        runtime.event_tap_stop().expect("stop");
        runtime.shutdown().expect("shutdown");
    }
}
