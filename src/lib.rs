#![feature(once_cell)]

use std::{cell::UnsafeCell, lazy::SyncOnceCell, mem::transmute};

use quest_hook::{
    inline_hook::Hook,
    libil2cpp::{Il2CppClass, Parameters, Return, WrapRaw},
};
use rand::{rngs::SmallRng, Rng, SeedableRng};
use tracing::info;

static BINARY_HOOKS: SyncOnceCell<Box<[Hook]>> = SyncOnceCell::new();
static UNARY_HOOKS: SyncOnceCell<Box<[Hook]>> = SyncOnceCell::new();

fn pick<T>(items: &[T]) -> &T {
    thread_local! {
        static RNG: UnsafeCell<SmallRng> = UnsafeCell::new(SmallRng::from_entropy());
    }

    let rng = unsafe { &mut *RNG.with(UnsafeCell::get) };
    let range = 0..items.len();
    let idx = rng.gen_range(range);

    unsafe { items.get_unchecked(idx) }
}

pub extern "C" fn binary_hook(n1: f32, n2: f32) -> f32 {
    let hooks = BINARY_HOOKS.get().unwrap();
    let original = pick(hooks).original().unwrap();
    let lmao = unsafe { transmute::<*const (), extern "C" fn(f32, f32) -> f32>(original) };
    lmao(n1, n2)
}

pub extern "C" fn unary_hook(n: f32) -> f32 {
    let hooks = UNARY_HOOKS.get().unwrap();
    let original = pick(hooks).original().unwrap();
    let lmao = unsafe { transmute::<*const (), extern "C" fn(f32) -> f32>(original) };
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
            unsafe { hook.install(target, binary_hook as *const ()) };
            binary.push(hook);

            info!("hooked binary operation `{}`", method.name())
        } else if <f32 as Parameters>::matches(method) {
            unsafe { hook.install(target, unary_hook as *const ()) };
            unary.push(hook);

            info!("hooked unary operation `{}`", method.name())
        }
    }

    BINARY_HOOKS.set(binary.into_boxed_slice()).unwrap();
    UNARY_HOOKS.set(unary.into_boxed_slice()).unwrap();
}
