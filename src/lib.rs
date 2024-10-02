use lua_api::Exportable;

pub mod cc_mod;
pub mod lua_api;
pub mod utils;
pub mod debug {
    pub use crate::lua_api::debug::show_str;
}

#[cfg(feature = "coroutine")]
#[cfg_attr(docsrs, doc(cfg(feature = "coroutine")))]
pub mod coroutine;

pub fn lib_exports() {
    #[cfg(feature = "coroutine")]
    "tick".export();
}
