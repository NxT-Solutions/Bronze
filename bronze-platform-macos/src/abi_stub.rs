//! In-crate C ABI stand-in so crate-local tests resolve symbols without
//! linking BronzeNative. The Tauri binary is the linked Swift subject
//! (`default-features = false` on bronze-desktop). Do not re-enable
//! `abi-stub` there.

use crate::abi::{
    BronzeIngressSnapshot, BronzeNativeUtf8View, BRONZE_ABI_VERSION, BRONZE_EVENT_TAP_DEGRADED,
    BRONZE_EVENT_TAP_IDLE, BRONZE_STATUS_CANCELLED, BRONZE_STATUS_CONTEXT_UNAVAILABLE,
    BRONZE_STATUS_DEGRADED, BRONZE_STATUS_DOUBLE_COMPLETION, BRONZE_STATUS_INVALID_UTF8,
    BRONZE_STATUS_NOT_FOUND, BRONZE_STATUS_OK, BRONZE_STATUS_SHUTTING_DOWN, BRONZE_TAP_FEED_CANCEL,
    BRONZE_TAP_FEED_DOWN, BRONZE_TAP_FEED_UP, BRONZE_TAP_REC_DISABLED, BRONZE_TAP_REC_TRIGGER,
};
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Mutex;
use std::thread::ThreadId;

enum ProbePhase {
    Open { ptr: usize, len: u64 },
    Completed,
    Cancelled,
}

struct Tables {
    shutting_down: bool,
    owned: HashSet<(usize, u64)>,
    empty_owned: u32,
    probes: HashMap<u64, ProbePhase>,
}

fn lock_tables() -> std::sync::MutexGuard<'static, Tables> {
    static TABLES: std::sync::OnceLock<Mutex<Tables>> = std::sync::OnceLock::new();
    TABLES
        .get_or_init(|| {
            Mutex::new(Tables {
                shutting_down: false,
                owned: HashSet::new(),
                empty_owned: 0,
                probes: HashMap::new(),
            })
        })
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn validate(view: BronzeNativeUtf8View) -> u32 {
    if view.ptr.is_null() {
        return if view.len == 0 {
            BRONZE_STATUS_OK
        } else {
            BRONZE_STATUS_INVALID_UTF8
        };
    }
    let Some(len) = usize::try_from(view.len).ok() else {
        return BRONZE_STATUS_INVALID_UTF8;
    };
    // SAFETY: caller owns [ptr, ptr+len); null+nonzero is rejected above.
    let bytes = unsafe { std::slice::from_raw_parts(view.ptr, len) };
    if std::str::from_utf8(bytes).is_ok() {
        BRONZE_STATUS_OK
    } else {
        BRONZE_STATUS_INVALID_UTF8
    }
}

fn owned_copy_locked(
    tables: &mut Tables,
    src: BronzeNativeUtf8View,
) -> Result<BronzeNativeUtf8View, u32> {
    if src.len == 0 {
        tables.empty_owned += 1;
        return Ok(BronzeNativeUtf8View {
            ptr: std::ptr::null(),
            len: 0,
        });
    }
    if src.ptr.is_null() {
        return Err(BRONZE_STATUS_INVALID_UTF8);
    }
    let len = usize::try_from(src.len).map_err(|_| BRONZE_STATUS_INVALID_UTF8)?;
    let mut buf = vec![0u8; len].into_boxed_slice();
    // SAFETY: src.ptr is non-null and len fits; copy into the new box.
    unsafe {
        std::ptr::copy_nonoverlapping(src.ptr, buf.as_mut_ptr(), len);
    }
    let ptr = Box::into_raw(buf) as *const u8;
    tables.owned.insert((ptr as usize, src.len));
    Ok(BronzeNativeUtf8View { ptr, len: src.len })
}

fn free_locked(tables: &mut Tables, view: BronzeNativeUtf8View) -> u32 {
    if view.len == 0 {
        if !view.ptr.is_null() {
            return BRONZE_STATUS_NOT_FOUND;
        }
        if tables.empty_owned == 0 {
            return BRONZE_STATUS_DOUBLE_COMPLETION;
        }
        tables.empty_owned -= 1;
        return BRONZE_STATUS_OK;
    }
    if view.ptr.is_null() {
        return BRONZE_STATUS_NOT_FOUND;
    }
    if !tables.owned.remove(&(view.ptr as usize, view.len)) {
        return BRONZE_STATUS_DOUBLE_COMPLETION;
    }
    let len = usize::try_from(view.len).expect("owned len fits usize");
    // SAFETY: ptr+len was produced by Box::into_raw in owned_copy_locked.
    let _ = unsafe { Box::from_raw(std::ptr::slice_from_raw_parts_mut(view.ptr as *mut u8, len)) };
    BRONZE_STATUS_OK
}

fn release_open(tables: &mut Tables) {
    let open_ids: Vec<u64> = tables
        .probes
        .iter()
        .filter_map(|(id, phase)| match phase {
            ProbePhase::Open { .. } => Some(*id),
            _ => None,
        })
        .collect();
    for id in open_ids {
        if let Some(ProbePhase::Open { ptr, len }) = tables.probes.remove(&id) {
            let _ = free_locked(
                tables,
                BronzeNativeUtf8View {
                    ptr: ptr as *const u8,
                    len,
                },
            );
            tables.probes.insert(id, ProbePhase::Cancelled);
        }
    }
}

fn drop_all_owned(tables: &mut Tables) {
    release_open(tables);
    let leftover: Vec<(usize, u64)> = tables.owned.iter().copied().collect();
    for (ptr, len) in leftover {
        let _ = free_locked(
            tables,
            BronzeNativeUtf8View {
                ptr: ptr as *const u8,
                len,
            },
        );
    }
    tables.empty_owned = 0;
}

#[no_mangle]
pub extern "C" fn bronze_native_abi_version() -> u32 {
    BRONZE_ABI_VERSION
}

#[no_mangle]
pub extern "C" fn bronze_native_validate_utf8(view: BronzeNativeUtf8View) -> u32 {
    validate(view)
}

#[no_mangle]
pub extern "C" fn bronze_native_test_view_len(view: BronzeNativeUtf8View) -> u64 {
    view.len
}

#[no_mangle]
pub extern "C" fn bronze_native_init() {
    let mut tables = lock_tables();
    drop_all_owned(&mut tables);
    tables.probes.clear();
    tables.shutting_down = false;
}

#[no_mangle]
pub extern "C" fn bronze_native_shutdown() {
    let mut tables = lock_tables();
    tables.shutting_down = true;
    release_open(&mut tables);
}

#[no_mangle]
pub extern "C" fn bronze_native_utf8_owned_copy(
    src: BronzeNativeUtf8View,
    out: *mut BronzeNativeUtf8View,
) -> u32 {
    if out.is_null() {
        return BRONZE_STATUS_NOT_FOUND;
    }
    let status = validate(src);
    if status != BRONZE_STATUS_OK {
        return status;
    }
    let mut tables = lock_tables();
    if tables.shutting_down {
        return BRONZE_STATUS_SHUTTING_DOWN;
    }
    match owned_copy_locked(&mut tables, src) {
        Ok(view) => {
            // SAFETY: caller provided a writable out slot.
            unsafe { *out = view };
            BRONZE_STATUS_OK
        }
        Err(status) => status,
    }
}

#[no_mangle]
pub extern "C" fn bronze_native_utf8_free(view: BronzeNativeUtf8View) -> u32 {
    let mut tables = lock_tables();
    free_locked(&mut tables, view)
}

#[no_mangle]
pub extern "C" fn bronze_native_probe_begin(request_id: u64, view: BronzeNativeUtf8View) -> u32 {
    let status = validate(view);
    if status != BRONZE_STATUS_OK {
        return status;
    }
    let mut tables = lock_tables();
    if tables.shutting_down {
        return BRONZE_STATUS_SHUTTING_DOWN;
    }
    if tables.probes.contains_key(&request_id) {
        return BRONZE_STATUS_DOUBLE_COMPLETION;
    }
    match owned_copy_locked(&mut tables, view) {
        Ok(owned) => {
            tables.probes.insert(
                request_id,
                ProbePhase::Open {
                    ptr: owned.ptr as usize,
                    len: owned.len,
                },
            );
            BRONZE_STATUS_OK
        }
        Err(status) => status,
    }
}

#[no_mangle]
pub extern "C" fn bronze_native_probe_complete(request_id: u64) -> u32 {
    let mut tables = lock_tables();
    match tables.probes.get(&request_id) {
        Some(ProbePhase::Open { .. }) => {
            if let Some(ProbePhase::Open { ptr, len }) = tables.probes.remove(&request_id) {
                let _ = free_locked(
                    &mut tables,
                    BronzeNativeUtf8View {
                        ptr: ptr as *const u8,
                        len,
                    },
                );
                tables.probes.insert(request_id, ProbePhase::Completed);
            }
            BRONZE_STATUS_OK
        }
        Some(ProbePhase::Completed | ProbePhase::Cancelled) => BRONZE_STATUS_DOUBLE_COMPLETION,
        None => BRONZE_STATUS_NOT_FOUND,
    }
}

#[no_mangle]
pub extern "C" fn bronze_native_probe_cancel(request_id: u64) -> u32 {
    let mut tables = lock_tables();
    match tables.probes.get(&request_id) {
        Some(ProbePhase::Open { .. }) => {
            if let Some(ProbePhase::Open { ptr, len }) = tables.probes.remove(&request_id) {
                let _ = free_locked(
                    &mut tables,
                    BronzeNativeUtf8View {
                        ptr: ptr as *const u8,
                        len,
                    },
                );
                tables.probes.insert(request_id, ProbePhase::Cancelled);
            }
            BRONZE_STATUS_CANCELLED
        }
        Some(ProbePhase::Completed | ProbePhase::Cancelled) => BRONZE_STATUS_DOUBLE_COMPLETION,
        None => BRONZE_STATUS_NOT_FOUND,
    }
}

#[no_mangle]
pub extern "C" fn bronze_native_probe_outstanding() -> u64 {
    let tables = lock_tables();
    tables
        .probes
        .values()
        .filter(|phase| matches!(phase, ProbePhase::Open { .. }))
        .count() as u64
}

struct EventTapStub {
    health: u32,
    fsm: u32,
    enabled: bool,
    producer: Option<ThreadId>,
    queue: VecDeque<(u32, u64)>,
    next_seq: u64,
}

fn lock_tap() -> std::sync::MutexGuard<'static, EventTapStub> {
    static TAP: std::sync::OnceLock<Mutex<EventTapStub>> = std::sync::OnceLock::new();
    TAP.get_or_init(|| {
        Mutex::new(EventTapStub {
            health: BRONZE_EVENT_TAP_IDLE,
            fsm: 0,
            enabled: false,
            producer: None,
            queue: VecDeque::new(),
            next_seq: 1,
        })
    })
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn tap_enqueue(tap: &mut EventTapStub, kind: u32) -> bool {
    let Some(producer) = tap.producer else {
        return false;
    };
    if std::thread::current().id() != producer {
        return false;
    }
    let seq = tap.next_seq;
    tap.next_seq = tap.next_seq.wrapping_add(1);
    if tap.queue.len() < 32 {
        tap.queue.push_back((kind, seq));
    }
    true
}

#[no_mangle]
pub extern "C" fn bronze_native_event_tap_start() -> u32 {
    let mut tap = lock_tap();
    tap.health = BRONZE_EVENT_TAP_DEGRADED;
    tap.producer = Some(std::thread::current().id());
    tap.fsm = 0;
    tap.queue.clear();
    BRONZE_STATUS_DEGRADED
}

#[no_mangle]
pub extern "C" fn bronze_native_event_tap_stop() -> u32 {
    let mut tap = lock_tap();
    tap.health = BRONZE_EVENT_TAP_IDLE;
    tap.fsm = 0;
    tap.producer = None;
    tap.queue.clear();
    tap.next_seq = 1;
    BRONZE_STATUS_OK
}

#[no_mangle]
pub extern "C" fn bronze_native_event_tap_health() -> u32 {
    lock_tap().health
}

#[no_mangle]
pub extern "C" fn bronze_native_event_tap_set_enabled(enabled: u32) -> u32 {
    let mut tap = lock_tap();
    tap.enabled = enabled != 0;
    tap.fsm = 0;
    BRONZE_STATUS_OK
}

#[no_mangle]
pub extern "C" fn bronze_native_event_tap_set_tap_count(_count: u32) -> u32 {
    BRONZE_STATUS_OK
}

#[no_mangle]
pub extern "C" fn bronze_native_event_tap_drain(kind: *mut u32, sequence: *mut u64) -> u32 {
    if kind.is_null() || sequence.is_null() {
        return BRONZE_STATUS_NOT_FOUND;
    }
    let mut tap = lock_tap();
    match tap.queue.pop_front() {
        Some((k, seq)) => {
            unsafe {
                *kind = k;
                *sequence = seq;
            }
            BRONZE_STATUS_OK
        }
        None => {
            unsafe {
                *kind = 0;
                *sequence = 0;
            }
            BRONZE_STATUS_OK
        }
    }
}

#[no_mangle]
pub extern "C" fn bronze_native_event_tap_fsm_state() -> u32 {
    lock_tap().fsm
}

#[no_mangle]
pub extern "C" fn bronze_native_event_tap_test_attach() -> u32 {
    let mut tap = lock_tap();
    tap.producer = Some(std::thread::current().id());
    tap.health = BRONZE_EVENT_TAP_IDLE;
    tap.fsm = 0;
    tap.queue.clear();
    tap.next_seq = 1;
    BRONZE_STATUS_OK
}

#[no_mangle]
pub extern "C" fn bronze_native_frontmost_pid() -> i32 {
    0
}

#[no_mangle]
pub extern "C" fn bronze_native_item_title(
    _body: BronzeNativeUtf8View,
    _out: *mut BronzeNativeUtf8View,
) -> u32 {
    BRONZE_STATUS_DEGRADED
}

#[no_mangle]
pub extern "C" fn bronze_native_event_tap_test_feed(
    kind: u32,
    _carbon_key: i32,
    _time_ns: u64,
) -> u32 {
    let mut tap = lock_tap();
    if tap.producer != Some(std::thread::current().id()) {
        return BRONZE_STATUS_NOT_FOUND;
    }
    if kind == BRONZE_TAP_FEED_CANCEL {
        tap.fsm = 0;
        return BRONZE_STATUS_OK;
    }
    if !tap.enabled {
        tap.fsm = 0;
        return BRONZE_STATUS_OK;
    }
    match kind {
        BRONZE_TAP_FEED_DOWN if tap.fsm == 0 => tap.fsm = 1,
        BRONZE_TAP_FEED_UP if tap.fsm == 1 => tap.fsm = 2,
        BRONZE_TAP_FEED_DOWN if tap.fsm == 2 => tap.fsm = 3,
        BRONZE_TAP_FEED_UP if tap.fsm == 3 => {
            tap.fsm = 0;
            let _ = tap_enqueue(&mut tap, BRONZE_TAP_REC_TRIGGER);
        }
        _ => tap.fsm = 0,
    }
    BRONZE_STATUS_OK
}

#[no_mangle]
pub extern "C" fn bronze_native_event_tap_test_disable() -> u32 {
    let mut tap = lock_tap();
    tap.fsm = 0;
    if tap_enqueue(&mut tap, BRONZE_TAP_REC_DISABLED) {
        BRONZE_STATUS_OK
    } else {
        BRONZE_STATUS_NOT_FOUND
    }
}

#[no_mangle]
pub extern "C" fn bronze_native_event_tap_test_enqueue_from_caller(kind: u32) -> u32 {
    let mut tap = lock_tap();
    if tap_enqueue(&mut tap, kind) {
        BRONZE_STATUS_OK
    } else {
        BRONZE_STATUS_NOT_FOUND
    }
}

struct IngressStub {
    snap: BronzeIngressSnapshot,
    inconsistent: bool,
}

fn lock_ingress() -> std::sync::MutexGuard<'static, IngressStub> {
    static INGRESS: std::sync::OnceLock<Mutex<IngressStub>> = std::sync::OnceLock::new();
    INGRESS
        .get_or_init(|| {
            Mutex::new(IngressStub {
                snap: BronzeIngressSnapshot {
                    target_pid: 0,
                    bundle_token: 0,
                    activation_generation: 0,
                    destination_uuid: [0; 16],
                    accept_capture_generation: 0,
                    policy_revision: 0,
                    settings_revision: 0,
                    context_generation: 0,
                    route: 0,
                    monotonic_time_ns: 0,
                },
                inconsistent: false,
            })
        })
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

#[no_mangle]
pub extern "C" fn bronze_native_ingress_publish(
    target_pid: i32,
    bundle_token: u64,
    activation_generation: u64,
    destination_uuid: *const u8,
    accept_capture_generation: u64,
    policy_revision: u64,
    settings_revision: u64,
    context_generation: u64,
    route: u32,
    monotonic_time_ns: u64,
) -> u32 {
    let mut uuid = [0u8; 16];
    if !destination_uuid.is_null() {
        unsafe {
            std::ptr::copy_nonoverlapping(destination_uuid, uuid.as_mut_ptr(), 16);
        }
    }
    lock_ingress().snap = BronzeIngressSnapshot {
        target_pid,
        bundle_token,
        activation_generation,
        destination_uuid: uuid,
        accept_capture_generation,
        policy_revision,
        settings_revision,
        context_generation,
        route,
        monotonic_time_ns,
    };
    BRONZE_STATUS_OK
}

#[no_mangle]
pub extern "C" fn bronze_native_ingress_load(
    target_pid: *mut i32,
    bundle_token: *mut u64,
    activation_generation: *mut u64,
    destination_uuid: *mut u8,
    accept_capture_generation: *mut u64,
    policy_revision: *mut u64,
    settings_revision: *mut u64,
    context_generation: *mut u64,
    route: *mut u32,
    monotonic_time_ns: *mut u64,
) -> u32 {
    let ingress = lock_ingress();
    if ingress.inconsistent {
        return BRONZE_STATUS_CONTEXT_UNAVAILABLE;
    }
    let snap = ingress.snap;
    unsafe {
        if !target_pid.is_null() {
            *target_pid = snap.target_pid;
        }
        if !bundle_token.is_null() {
            *bundle_token = snap.bundle_token;
        }
        if !activation_generation.is_null() {
            *activation_generation = snap.activation_generation;
        }
        if !destination_uuid.is_null() {
            std::ptr::copy_nonoverlapping(snap.destination_uuid.as_ptr(), destination_uuid, 16);
        }
        if !accept_capture_generation.is_null() {
            *accept_capture_generation = snap.accept_capture_generation;
        }
        if !policy_revision.is_null() {
            *policy_revision = snap.policy_revision;
        }
        if !settings_revision.is_null() {
            *settings_revision = snap.settings_revision;
        }
        if !context_generation.is_null() {
            *context_generation = snap.context_generation;
        }
        if !route.is_null() {
            *route = snap.route;
        }
        if !monotonic_time_ns.is_null() {
            *monotonic_time_ns = snap.monotonic_time_ns;
        }
    }
    BRONZE_STATUS_OK
}

#[no_mangle]
pub extern "C" fn bronze_native_ingress_test_begin_inconsistent() -> u32 {
    lock_ingress().inconsistent = true;
    BRONZE_STATUS_OK
}

#[no_mangle]
pub extern "C" fn bronze_native_ingress_test_end_inconsistent() -> u32 {
    lock_ingress().inconsistent = false;
    BRONZE_STATUS_OK
}

#[no_mangle]
pub extern "C" fn bronze_native_pasteboard_write(
    plain: BronzeNativeUtf8View,
    html: BronzeNativeUtf8View,
) -> u32 {
    let plain_status = validate(plain);
    if plain_status != BRONZE_STATUS_OK {
        return plain_status;
    }
    let html_status = validate(html);
    if html_status != BRONZE_STATUS_OK {
        return html_status;
    }
    BRONZE_STATUS_DEGRADED
}

#[no_mangle]
pub extern "C" fn bronze_native_bundle_id_for_pid(
    _pid: i32,
    _out: *mut BronzeNativeUtf8View,
) -> u32 {
    BRONZE_STATUS_DEGRADED
}

#[no_mangle]
pub extern "C" fn bronze_native_app_icon_png(
    bundle_or_name: BronzeNativeUtf8View,
    _out: *mut BronzeNativeUtf8View,
) -> u32 {
    let status = validate(bundle_or_name);
    if status != BRONZE_STATUS_OK {
        return status;
    }
    BRONZE_STATUS_DEGRADED
}

#[no_mangle]
pub extern "C" fn bronze_native_list_installed_apps(_out: *mut BronzeNativeUtf8View) -> u32 {
    BRONZE_STATUS_DEGRADED
}

#[no_mangle]
pub extern "C" fn bronze_native_pick_installed_app(_out: *mut BronzeNativeUtf8View) -> u32 {
    BRONZE_STATUS_DEGRADED
}

#[no_mangle]
pub extern "C" fn bronze_native_deliver_user_notice(
    title: BronzeNativeUtf8View,
    body: BronzeNativeUtf8View,
) -> u32 {
    let title_status = validate(title);
    if title_status != BRONZE_STATUS_OK {
        return title_status;
    }
    let body_status = validate(body);
    if body_status != BRONZE_STATUS_OK {
        return body_status;
    }
    BRONZE_STATUS_DEGRADED
}

#[no_mangle]
pub extern "C" fn bronze_native_notification_authorization_status() -> u32 {
    BRONZE_STATUS_DEGRADED
}

#[no_mangle]
pub extern "C" fn bronze_native_request_notification_authorization() -> u32 {
    BRONZE_STATUS_DEGRADED
}
