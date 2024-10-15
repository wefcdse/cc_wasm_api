#![cfg_attr(docsrs, feature(doc_cfg))]

use lua_api::Exportable;
#[cfg(feature = "addon")]
#[cfg_attr(docsrs, doc(cfg(feature = "addon")))]
pub mod addon {
    pub mod local_monitor;
    pub mod misc;
    pub mod throw;
    pub mod vec2d;
}

pub mod prelude {
    #[cfg(feature = "coroutine")]
    #[cfg_attr(docsrs, doc(cfg(feature = "coroutine")))]
    pub use crate::coroutine::{
        sleep, spawn, yield_now, AsyncLock, AsyncLockGuard, CoroutineSpawn, TickSyncer,
        UnsyncChannel,
    };
    #[cfg(feature = "eval")]
    #[cfg_attr(docsrs, doc(cfg(feature = "eval")))]
    pub use crate::eval::{eval, exec};
    pub use crate::export_funcs;
    pub use crate::lua_api::{nil::Nil, Exportable, Importable, LuaResult};
    pub use crate::utils::{either::Either, Number, SyncNonSync};
}

pub mod cc_mod;
#[cfg(feature = "eval")]
#[cfg_attr(docsrs, doc(cfg(feature = "eval")))]
pub mod eval;
pub mod lua_api;
pub mod utils;
pub mod debug {
    pub use dbg_inner::*;
    #[cfg(feature = "debug")]
    mod dbg_inner {
        pub use crate::lua_api::debug::show_str;
        use std::fmt::Debug;
        pub fn show_debug(value: &impl Debug) {
            show_str(&format!("{:?}", value));
        }
        pub fn show_debug_desc(description: &str, value: &impl Debug) {
            show_str(&format!("{description}: {:?}", value));
        }
    }
    #[cfg(not(feature = "debug"))]
    mod dbg_inner {
        use std::fmt::Debug;

        pub fn show_str(value: &str) {
            let _ = value;
        }
        pub fn show_debug(value: &impl Debug) {
            let _ = value;
        }
        pub fn show_debug_desc(description: &str, value: &impl Debug) {
            let _ = (description, value);
        }
    }
}

/// a simple coroutine runtime, use to run coroutine in single thread environment.
///
/// when `coroutine` feature is enabled, the crate will export a `tick` function to computer craft
#[cfg(feature = "coroutine")]
#[cfg_attr(docsrs, doc(cfg(feature = "coroutine")))]
pub mod coroutine;

/// used in [export_funcs], should not be called otherwise
///
/// # Safety
/// this function should not be manually called
///
pub unsafe fn lib_exports() {
    #[cfg(feature = "coroutine")]
    "tick".export();

    #[cfg(feature = "eval")]
    "eval_result".export();

    #[cfg(feature = "eval")]
    "eval_string".export();
}
