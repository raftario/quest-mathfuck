#![feature(once_cell, option_result_unwrap_unchecked)]

use std::{lazy::SyncOnceCell, mem::transmute};

use quest_hook::{
    inline_hook::Hook,
    libil2cpp::{Il2CppClass, Parameters, Return, WrapRaw},
};
use tracing::info;

static BINARY_HOOKS: SyncOnceCell<Box<[Hook]>> = SyncOnceCell::new();
static UNARY_HOOKS: SyncOnceCell<Box<[Hook]>> = SyncOnceCell::new();

fn pick<T>(items: &[T]) -> &T {
    let idx = fastrand::usize(..items.len());
    unsafe { items.get_unchecked(idx) }
}

pub extern "C" fn binary_hook(n1: f32, n2: f32) -> f32 {
    let hooks = unsafe { BINARY_HOOKS.get().unwrap_unchecked() };
    let original = unsafe { pick(hooks).original().unwrap_unchecked() };
    let lmao: extern "C" fn(f32, f32) -> f32 = unsafe { transmute(original) };
    lmao(n1, n2)
}

pub extern "C" fn unary_hook(n: f32) -> f32 {
    let hooks = unsafe { UNARY_HOOKS.get().unwrap_unchecked() };
    let original = unsafe { pick(hooks).original().unwrap_unchecked() };
    let lmao: extern "C" fn(f32) -> f32 = unsafe { transmute(original) };
    lmao(n)
}

#[no_mangle]
pub extern "C" fn setup() {
    quest_hook::setup(env!("CARGO_PKG_NAME"));
}

#[no_mangle]
pub extern "C" fn load() {
    let mathf = Il2CppClass::find("UnityEngine", "Mathf").unwrap();
    let mut binary = Vec::new();
    let mut unary = Vec::new();

    for method in mathf.methods() {
        if !method.is_static() || !<f32 as Return>::matches(method.return_ty()) {
            continue;
        }

        let hook = Hook::new();
        let target: *const () = unsafe { transmute(method.raw().methodPointer) };

        if <(f32, f32) as Parameters>::matches(method) {
            let success = unsafe { hook.install(target, binary_hook as *const ()) };
            if success {
                binary.push(hook);
                info!("hooked binary operation `{}`", method.name());
            } else {
                info!("failed to hook binary operation `{}`", method.name());
            }
        } else if <f32 as Parameters>::matches(method) {
            let success = unsafe { hook.install(target, unary_hook as *const ()) };
            if success {
                unary.push(hook);
                info!("hooked unary operation `{}`", method.name());
            } else {
                info!("failed to hook unary operation `{}`", method.name());
            }
        }
    }

    BINARY_HOOKS.set(binary.into_boxed_slice()).unwrap();
    UNARY_HOOKS.set(unary.into_boxed_slice()).unwrap();
}
