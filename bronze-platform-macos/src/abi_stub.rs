//! In-crate C ABI stand-in so crate-local tests resolve symbols without
//! linking BronzeNative. The Tauri binary is the linked Swift subject
//! (`default-features = false` on bronze-desktop). Do not re-enable
//! `abi-stub` there.

use crate::abi::{
    BronzeNativeUtf8View, BRONZE_ABI_VERSION, BRONZE_STATUS_CANCELLED,
    BRONZE_STATUS_DOUBLE_COMPLETION, BRONZE_STATUS_INVALID_UTF8, BRONZE_STATUS_NOT_FOUND,
    BRONZE_STATUS_OK, BRONZE_STATUS_SHUTTING_DOWN,
};
use std::collections::{HashMap, HashSet};
use std::sync::Mutex;

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
