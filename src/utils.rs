use std::{
    fmt::Debug,
    ops::{Deref, DerefMut},
};

use crate::lua_api::LuaResult;
pub struct ImplResult;
pub struct ImplIter;
pub struct ImplValue;

pub trait Debuged<T> {
    fn debuged(self) -> LuaResult<T>;
}
impl<T, E: Debug> Debuged<T> for Result<T, E> {
    fn debuged(self) -> LuaResult<T> {
        Ok(self.map_err(|e| format!("{:?}", e))?)
    }
}
/// wasm does not support thread, so this is safe
pub struct SyncNonSync<T>(pub T);
// wasm doesnot support thread
unsafe impl<T> Send for SyncNonSync<T> {}
// wasm doesnot support thread
unsafe impl<T> Sync for SyncNonSync<T> {}

impl<T> Deref for SyncNonSync<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<T> DerefMut for SyncNonSync<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

pub use other_type::Number;

mod other_type {

    use std::fmt::Display;

    use crate::lua_api::{next_import_type, Exportable, Importable, LuaError, LuaResult, Typed};

    /// a Number type, can be both float or int.
    /// impls [Importable] and [Exportable]
    #[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
    pub enum Number {
        Int(i64),
        Float(f64),
    }
    impl Number {
        pub fn to_i32(self) -> i32 {
            match self {
                Number::Int(i) => i as i32,
                Number::Float(i) => i as i32,
            }
        }
        pub fn to_i64(self) -> i64 {
            match self {
                Number::Int(i) => i,
                Number::Float(i) => i as i64,
            }
        }
        pub fn to_f32(self) -> f32 {
            match self {
                Number::Int(i) => i as f32,
                Number::Float(i) => i as f32,
            }
        }
        pub fn to_f64(self) -> f64 {
            match self {
                Number::Int(i) => i as f64,
                Number::Float(i) => i,
            }
        }
        // fn to_number<T: From<f64> + From<i64>>(self) -> T {
        //     match self {
        //         Number::Int(i) => i.into(),
        //         Number::Float(i) => i.into(),
        //     }
        // }
    }
    impl Exportable for Number {
        fn export(&self) {
            match self {
                Number::Int(i) => i.export(),
                Number::Float(f) => f.export(),
            }
        }
    }
    impl Importable for Number {
        fn import() -> LuaResult<Self> {
            match next_import_type() {
                Typed::None => Err(LuaError::from_str("not receiving any value")),
                Typed::I32 => Ok(Self::Int(i32::import()? as i64)),
                Typed::I64 => Ok(Self::Int(i64::import()?)),
                Typed::String => Err(LuaError::from_str("receiving string")),
                Typed::F32 => Ok(Self::Float(f32::import()? as f64)),
                Typed::F64 => Ok(Self::Float(f64::import()?)),
                Typed::Type => Err(LuaError::from_str("receiving type")),
                Typed::Object => Err(LuaError::from_str("receiving obj")),
                Typed::Error => Err(LuaError::from_str("receiving error type")),
                Typed::Nil => Err(LuaError::from_str("receiving nil")),
                Typed::Bool => Err(LuaError::from_str("receiving bool")),
            }
        }
    }
    impl Display for Number {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Number::Int(s) => s.fmt(f),
                Number::Float(s) => s.fmt(f),
            }
        }
    }
}

pub mod either {
    use crate::{
        lua_api::LuaError,
        prelude::{Exportable, Importable},
    };

    #[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
    pub enum Either<First, Second> {
        First(First),
        Second(Second),
    }
    impl<L: Exportable, R: Exportable> Exportable for Either<L, R> {
        fn export(&self) {
            match self {
                Either::First(l) => l.export(),
                Either::Second(r) => r.export(),
            }
        }
    }
    impl<L: Importable, R: Importable> Importable for Either<L, R> {
        fn import() -> crate::prelude::LuaResult<Self> {
            if let Ok(v) = L::import() {
                Ok(Self::First(v))
            } else if let Ok(v) = R::import() {
                Ok(Self::Second(v))
            } else {
                Err(LuaError::from_str("neither of the value matches"))
            }
        }
    }
}
