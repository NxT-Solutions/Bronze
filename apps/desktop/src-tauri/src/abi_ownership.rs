//! Linked-app ABI ownership suite (story 2.4, CAP-004, SEC-006).
//! Runs only when `build.rs` linked `libBronzeNative.a`.

#[cfg(all(target_os = "macos", bronze_native_linked))]
mod linked {
    use bronze_platform_macos::{NativeError, NativeRuntime, ProbeTerminal};

    fn start() -> NativeRuntime {
        NativeRuntime::start().expect("linked BronzeNative start (story 2.4)")
    }

    #[test]
    fn empty_embedded_nul_invalid_utf8_and_large_payload() {
        let _guard = crate::lock_native_runtime();
        let runtime = start();

        runtime.probe_begin(10, b"").expect("empty");
        assert_eq!(
            runtime.probe_complete(10).expect("empty done"),
            ProbeTerminal::Completed
        );

        runtime.probe_begin(11, b"a\0b").expect("embedded NUL");
        assert_eq!(
            runtime.probe_complete(11).expect("nul done"),
            ProbeTerminal::Completed
        );

        assert_eq!(
            runtime.probe_begin(12, &[0xff]).unwrap_err(),
            NativeError::InvalidUtf8
        );
        assert_eq!(runtime.probe_outstanding().expect("no invalid probe"), 0);

        let large = vec![b'x'; 1024 * 1024];
        runtime.probe_begin(13, &large).expect("large");
        assert_eq!(
            runtime.probe_complete(13).expect("large done"),
            ProbeTerminal::Completed
        );
        assert_eq!(runtime.probe_outstanding().expect("drained"), 0);

        let owned = runtime.copy_utf8(b"secret-not-for-logs").expect("copy");
        let rendered = format!("{owned:?}");
        assert!(
            !rendered.contains("secret"),
            "diagnostics must be content-free (SEC-006): {rendered}"
        );
        owned.free().expect("free");
        runtime.shutdown().expect("shutdown");
    }

    #[test]
    fn each_case_completes_exactly_once_or_cancels() {
        let _guard = crate::lock_native_runtime();
        let runtime = start();
        runtime.probe_begin(20, b"once").expect("begin");
        assert_eq!(runtime.probe_outstanding().expect("open"), 1);
        assert_eq!(
            runtime.probe_complete(20).expect("complete"),
            ProbeTerminal::Completed
        );
        assert_eq!(
            runtime.probe_complete(20).unwrap_err(),
            NativeError::DoubleCompletion
        );
        runtime.probe_begin(21, b"cancel-me").expect("begin cancel");
        assert_eq!(
            runtime.probe_cancel(21).expect("cancel"),
            ProbeTerminal::Cancelled
        );
        assert_eq!(
            runtime.probe_cancel(21).unwrap_err(),
            NativeError::DoubleCompletion
        );
        assert_eq!(runtime.probe_outstanding().expect("drained"), 0);
        runtime.shutdown().expect("shutdown");
    }

    #[test]
    fn shutdown_race_completes_once_or_cancels() {
        let _guard = crate::lock_native_runtime();
        let runtime = start();
        runtime.probe_begin(30, b"race").expect("begin");
        let handle = std::thread::spawn(|| NativeRuntime::probe_complete_id(30));
        let _ = runtime.shutdown();
        let complete = handle.join().expect("join");
        let once_or_cancel = matches!(
            complete,
            Ok(ProbeTerminal::Completed)
                | Ok(ProbeTerminal::Cancelled)
                | Err(NativeError::DoubleCompletion)
                | Err(NativeError::Cancelled)
                | Err(NativeError::NotFound)
        );
        assert!(
            once_or_cancel,
            "shutdown race must complete once or cancel: {complete:?}"
        );
        assert_ne!(
            NativeRuntime::probe_complete_id(30),
            Ok(ProbeTerminal::Completed)
        );
        assert_eq!(NativeRuntime::probe_outstanding_id().expect("count"), 0);
    }

    #[test]
    fn ten_thousand_round_trip_optional_smoke() {
        let _guard = crate::lock_native_runtime();
        let runtime = start();
        for id in 1_000..11_000 {
            runtime
                .probe_begin(id, b"rt")
                .unwrap_or_else(|err| panic!("begin {id}: {err}"));
            runtime
                .probe_complete(id)
                .unwrap_or_else(|err| panic!("complete {id}: {err}"));
        }
        assert_eq!(runtime.probe_outstanding().expect("drained"), 0);
        runtime.shutdown().expect("shutdown");
    }
}
