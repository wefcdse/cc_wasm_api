#![cfg_attr(docsrs, feature(doc_cfg))]

use lua_api::Exportable;

pub mod prelude {
    #[cfg(feature = "coroutine")]
    #[cfg_attr(docsrs, doc(cfg(feature = "coroutine")))]
    pub use crate::coroutine::{
        sleep, spawn, yield_now, CoroutineSpawn, TickSyncer, UnsyncChannel,
    };
    #[cfg(feature = "eval")]
    #[cfg_attr(docsrs, doc(cfg(feature = "eval")))]
    pub use crate::eval::eval;
    pub use crate::export_funcs;
    pub use crate::lua_api::{Exportable, Importable, LuaResult};
}

pub mod cc_mod;
#[cfg(feature = "eval")]
#[cfg_attr(docsrs, doc(cfg(feature = "eval")))]
pub mod eval;
pub mod lua_api;
pub mod utils;
pub mod debug {
    pub use crate::lua_api::debug::show_str;
}

/// a simple coroutine runtime, use to run coroutine in single thread environment.
///
/// when `coroutine` feature is enabled, the crate will export a `tick` function to computer craft
#[cfg(feature = "coroutine")]
#[cfg_attr(docsrs, doc(cfg(feature = "coroutine")))]
pub mod coroutine;

/// used in [export_funcs], this should not be called
pub fn lib_exports() {
    #[cfg(feature = "coroutine")]
    "tick".export();

    #[cfg(feature = "eval")]
    "eval_result".export();

    #[cfg(feature = "eval")]
    "eval_string".export();
}
