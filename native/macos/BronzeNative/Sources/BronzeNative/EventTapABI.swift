// C ABI for the listen-only event tap (story 3.3). No product strings.

@_cdecl("bronze_native_event_tap_start")
public func bronze_native_event_tap_start() -> UInt32 {
    let health = EventTapRuntime.shared.startLive()
    return health == .degraded ? BRONZE_STATUS_DEGRADED : BRONZE_STATUS_OK
}

@_cdecl("bronze_native_event_tap_stop")
public func bronze_native_event_tap_stop() -> UInt32 {
    EventTapRuntime.shared.stop()
    return BRONZE_STATUS_OK
}

@_cdecl("bronze_native_event_tap_health")
public func bronze_native_event_tap_health() -> UInt32 {
    EventTapRuntime.shared.health.rawValue
}

@_cdecl("bronze_native_event_tap_set_enabled")
public func bronze_native_event_tap_set_enabled(_ enabled: UInt32) -> UInt32 {
    EventTapRuntime.shared.setGestureEnabled(enabled != 0)
    return BRONZE_STATUS_OK
}

@_silgen_name("bronze_native_event_tap_drain")
public func bronze_native_event_tap_drain(
    _ kind: UnsafeMutablePointer<UInt32>?,
    _ sequence: UnsafeMutablePointer<UInt64>?
) -> UInt32 {
    guard let kind, let sequence else {
        return BRONZE_STATUS_NOT_FOUND
    }
    guard let record = EventTapRuntime.shared.spsc.drain() else {
        kind.pointee = 0
        sequence.pointee = 0
        return BRONZE_STATUS_OK
    }
    kind.pointee = record.kind.rawValue
    sequence.pointee = record.sequence
    return BRONZE_STATUS_OK
}

@_cdecl("bronze_native_event_tap_fsm_state")
public func bronze_native_event_tap_fsm_state() -> UInt32 {
    EventTapRuntime.shared.fsmStateRaw
}

@_cdecl("bronze_native_event_tap_test_attach")
public func bronze_native_event_tap_test_attach() -> UInt32 {
    EventTapRuntime.shared.attachTestProducer()
    return BRONZE_STATUS_OK
}

@_cdecl("bronze_native_event_tap_test_feed")
public func bronze_native_event_tap_test_feed(_ kind: UInt32, _ carbon_key: Int32, _ time_ns: UInt64) -> UInt32 {
    EventTapRuntime.shared.feedTest(kind: kind, carbonKey: carbon_key, timeNs: time_ns)
        ? BRONZE_STATUS_OK
        : BRONZE_STATUS_NOT_FOUND
}

@_cdecl("bronze_native_event_tap_test_disable")
public func bronze_native_event_tap_test_disable() -> UInt32 {
    EventTapRuntime.shared.simulateTapDisabled()
        ? BRONZE_STATUS_OK
        : BRONZE_STATUS_NOT_FOUND
}

@_cdecl("bronze_native_event_tap_test_enqueue_from_caller")
public func bronze_native_event_tap_test_enqueue_from_caller(_ kind: UInt32) -> UInt32 {
    let rec = EventTapRecordKind(rawValue: kind) ?? .none
    return EventTapRuntime.shared.enqueueFromCaller(rec)
        ? BRONZE_STATUS_OK
        : BRONZE_STATUS_NOT_FOUND
}
