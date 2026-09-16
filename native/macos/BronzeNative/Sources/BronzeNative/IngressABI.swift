@_cdecl("bronze_native_ingress_publish")
public func bronze_native_ingress_publish(
    _ target_pid: Int32,
    _ bundle_token: UInt64,
    _ activation_generation: UInt64,
    _ destination_uuid: UnsafePointer<UInt8>?,
    _ accept_capture_generation: UInt64,
    _ policy_revision: UInt64,
    _ settings_revision: UInt64,
    _ context_generation: UInt64,
    _ route: UInt32,
    _ monotonic_time_ns: UInt64
) -> UInt32 {
    var hi: UInt64 = 0
    var lo: UInt64 = 0
    if let destination_uuid {
        for i in 0..<8 {
            hi |= UInt64(destination_uuid[i]) << (8 * i)
            lo |= UInt64(destination_uuid[i + 8]) << (8 * i)
        }
    }
    IngressRuntime.shared.publish(BronzeIngressSnapshot(
        targetPid: target_pid,
        bundleToken: bundle_token,
        activationGeneration: activation_generation,
        destinationUuidHi: hi,
        destinationUuidLo: lo,
        acceptCaptureGeneration: accept_capture_generation,
        policyRevision: policy_revision,
        settingsRevision: settings_revision,
        contextGeneration: context_generation,
        route: route,
        monotonicTimeNs: monotonic_time_ns
    ))
    return BRONZE_STATUS_OK
}

@_cdecl("bronze_native_ingress_load")
public func bronze_native_ingress_load(
    _ target_pid: UnsafeMutablePointer<Int32>?,
    _ bundle_token: UnsafeMutablePointer<UInt64>?,
    _ activation_generation: UnsafeMutablePointer<UInt64>?,
    _ destination_uuid: UnsafeMutablePointer<UInt8>?,
    _ accept_capture_generation: UnsafeMutablePointer<UInt64>?,
    _ policy_revision: UnsafeMutablePointer<UInt64>?,
    _ settings_revision: UnsafeMutablePointer<UInt64>?,
    _ context_generation: UnsafeMutablePointer<UInt64>?,
    _ route: UnsafeMutablePointer<UInt32>?,
    _ monotonic_time_ns: UnsafeMutablePointer<UInt64>?
) -> UInt32 {
    guard let snap = IngressRuntime.shared.load() else {
        return BRONZE_STATUS_CONTEXT_UNAVAILABLE
    }
    target_pid?.pointee = snap.targetPid
    bundle_token?.pointee = snap.bundleToken
    activation_generation?.pointee = snap.activationGeneration
    accept_capture_generation?.pointee = snap.acceptCaptureGeneration
    policy_revision?.pointee = snap.policyRevision
    settings_revision?.pointee = snap.settingsRevision
    context_generation?.pointee = snap.contextGeneration
    route?.pointee = snap.route
    monotonic_time_ns?.pointee = snap.monotonicTimeNs
    if let destination_uuid {
        for i in 0..<8 {
            destination_uuid[i] = UInt8(truncatingIfNeeded: snap.destinationUuidHi >> (8 * i))
            destination_uuid[i + 8] = UInt8(truncatingIfNeeded: snap.destinationUuidLo >> (8 * i))
        }
    }
    return BRONZE_STATUS_OK
}

@_cdecl("bronze_native_ingress_test_begin_inconsistent")
public func bronze_native_ingress_test_begin_inconsistent() -> UInt32 {
    IngressRuntime.shared.testBeginInconsistentWrite()
    return BRONZE_STATUS_OK
}

@_cdecl("bronze_native_ingress_test_end_inconsistent")
public func bronze_native_ingress_test_end_inconsistent() -> UInt32 {
    IngressRuntime.shared.testEndInconsistentWrite()
    return BRONZE_STATUS_OK
}
