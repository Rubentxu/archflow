// ═══════════════════════════════════════════════════════════════════════════════
// ArchFlow Web - Callback Registry for WASM-JavaScript Interop
//
// EPIC-WEB-013 HU-003: Callback Registry with wasm-bindgen Closures
//
// This module provides a thread-safe registry for JavaScript callbacks
// that can be invoked from Rust/WASM, enabling real-time event pushing
// from Rust to JavaScript without polling overhead.
// ═══════════════════════════════════════════════════════════════════════════════

#![allow(missing_docs)]

use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicU8, AtomicU32, Ordering};
use wasm_bindgen::prelude::*;

// ═══════════════════════════════════════════════════════════════════════════════
// CALLBACK IDENTIFICATION
// ═══════════════════════════════════════════════════════════════════════════════

/// Unique callback identifier
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[wasm_bindgen]
pub struct CallbackId(u32);

#[wasm_bindgen]
impl CallbackId {
    #[wasm_bindgen(getter)]
    pub fn value(&self) -> u32 {
        self.0
    }
}

static NEXT_CALLBACK_ID: AtomicU32 = AtomicU32::new(1);

fn next_callback_id() -> CallbackId {
    let id = NEXT_CALLBACK_ID.fetch_add(1, Ordering::SeqCst);
    CallbackId(id)
}

// ═══════════════════════════════════════════════════════════════════════════════
// STORAGE - Raw pointer approach for Rust 2024 compatibility
// ═══════════════════════════════════════════════════════════════════════════════

struct CallbackWrapper {
    function: js_sys::Function,
    is_oneshot: bool,
    call_count: u32,
    event_type: String,
}

impl CallbackWrapper {
    fn new(function: js_sys::Function, event_type: String, is_oneshot: bool) -> Self {
        Self {
            function,
            is_oneshot,
            call_count: 0,
            event_type,
        }
    }

    fn should_remove(&self) -> bool {
        self.is_oneshot && self.call_count > 0
    }

    fn invoke(&mut self, data: &JsValue) {
        self.call_count += 1;
        let _ = self.function.call1(&JsValue::NULL, data);
    }
}

struct CallbackRegistryStorage {
    callbacks: BTreeMap<CallbackId, CallbackWrapper>,
    event_index: BTreeMap<String, Vec<CallbackId>>,
}

impl CallbackRegistryStorage {
    fn new() -> Self {
        Self {
            callbacks: BTreeMap::new(),
            event_index: BTreeMap::new(),
        }
    }
}

// Use raw pointer to avoid mutable reference to static
// This is safe in WASM because it's single-threaded
static mut STORAGE_PTR: Option<*mut CallbackRegistryStorage> = None;
static INIT: AtomicU8 = AtomicU8::new(0);

fn with_storage<F, R>(f: F) -> R
where
    F: FnOnce(&mut CallbackRegistryStorage) -> R,
{
    unsafe {
        // Initialize if needed
        if INIT.load(Ordering::SeqCst) == 0 {
            if INIT
                .compare_exchange(0, 1, Ordering::SeqCst, Ordering::SeqCst)
                .is_ok()
            {
                let boxed = Box::new(CallbackRegistryStorage::new());
                STORAGE_PTR = Some(Box::leak(boxed) as *mut CallbackRegistryStorage);
                INIT.store(2, Ordering::SeqCst);
            } else {
                // Wait for initialization
                while INIT.load(Ordering::SeqCst) != 2 {
                    core::hint::spin_loop();
                }
            }
        }

        // Get raw pointer and convert to reference
        // This is safe because:
        // 1. WASM is single-threaded
        // 2. The reference only lives for the duration of 'f'
        // 3. STORAGE_PTR is guaranteed to be valid after initialization
        let storage_ptr = STORAGE_PTR.unwrap();
        let storage = &mut *storage_ptr;
        f(storage)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// CALLBACK REGISTRY
// ═══════════════════════════════════════════════════════════════════════════════

#[wasm_bindgen]
pub struct CallbackRegistry;

#[wasm_bindgen]
impl CallbackRegistry {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        let _ = with_storage(|_| ()); // Ensure storage is initialized
        Self
    }

    #[wasm_bindgen]
    pub fn register(
        &mut self,
        callback: &js_sys::Function,
        event_type: String,
        is_oneshot: bool,
    ) -> CallbackId {
        let function = js_sys::Function::from(callback.clone());
        let id = next_callback_id();
        let wrapper = CallbackWrapper::new(function, event_type.clone(), is_oneshot);

        with_storage(|storage| {
            storage.callbacks.insert(id, wrapper);
            storage
                .event_index
                .entry(event_type)
                .or_insert_with(Vec::new)
                .push(id);
        });

        id
    }

    #[wasm_bindgen]
    pub fn unregister(&mut self, id: CallbackId) -> bool {
        let event_type =
            with_storage(|storage| storage.callbacks.get(&id).map(|w| w.event_type.clone()));

        let removed = with_storage(|storage| storage.callbacks.remove(&id).is_some());

        if let Some(et) = event_type {
            with_storage(|storage| {
                if let Some(ids) = storage.event_index.get_mut(&et) {
                    ids.retain(|&x| x != id);
                }
            });
        }

        removed
    }

    #[wasm_bindgen]
    pub fn unregister_all(&mut self, event_type: String) -> u32 {
        let ids_to_remove = with_storage(|storage| {
            storage
                .event_index
                .get(&event_type)
                .map(|ids| ids.clone())
                .unwrap_or_default()
        });

        let count = ids_to_remove.len() as u32;

        with_storage(|storage| {
            for id in &ids_to_remove {
                storage.callbacks.remove(id);
            }
            storage.event_index.remove(&event_type);
        });

        count
    }

    #[wasm_bindgen]
    pub fn invoke(&mut self, event_type: String, data: &JsValue) -> u32 {
        let ids = with_storage(|storage| {
            storage
                .event_index
                .get(&event_type)
                .map(|ids| ids.clone())
                .unwrap_or_default()
        });

        let mut invoked = 0;
        let mut to_remove = Vec::new();

        for id in ids {
            let should_remove = with_storage(|storage| {
                if let Some(wrapper) = storage.callbacks.get_mut(&id) {
                    wrapper.invoke(data);
                    invoked += 1;
                    wrapper.should_remove()
                } else {
                    false
                }
            });

            if should_remove {
                to_remove.push(id);
            }
        }

        for id in to_remove {
            with_storage(|storage| {
                storage.callbacks.remove(&id);
                if let Some(ids) = storage.event_index.get_mut(&event_type) {
                    ids.retain(|&x| x != id);
                }
            });
        }

        invoked
    }

    #[wasm_bindgen]
    pub fn event_callback_count(&mut self, event_type: String) -> u32 {
        with_storage(|storage| {
            storage
                .event_index
                .get(&event_type)
                .map(|ids| ids.len() as u32)
                .unwrap_or(0)
        })
    }

    #[wasm_bindgen(getter)]
    pub fn total_count(&self) -> u32 {
        with_storage(|storage| storage.callbacks.len() as u32)
    }

    #[wasm_bindgen]
    pub fn clear(&mut self) {
        with_storage(|storage| {
            storage.callbacks.clear();
            storage.event_index.clear();
        });
    }
}

#[wasm_bindgen]
pub fn get_global_callback_registry() -> CallbackRegistry {
    CallbackRegistry::new()
}
