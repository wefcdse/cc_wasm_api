pub use lua_result::LuaError;
pub use lua_result::LuaResult;

mod lua_result {
    pub type LuaResult<T> = Result<T, LuaError>;
    use std::borrow::Cow;
    #[derive(Debug, Clone, Default, Hash, PartialEq, Eq, PartialOrd, Ord)]
    pub struct LuaError(Cow<'static, str>);
    impl<T: ToString> From<T> for LuaError {
        fn from(value: T) -> Self {
            Self(Cow::Owned(value.to_string()))
        }
    }
    impl LuaError {
        #[allow(clippy::should_implement_trait)]
        pub fn from_str(msg: &'static str) -> Self {
            Self(Cow::Borrowed(msg))
        }
        pub fn from_string(msg: String) -> Self {
            Self(Cow::Owned(msg))
        }
        pub fn as_str(&self) -> &str {
            &self.0
        }
    }
}
/// types which can be exported to computer craft
///
/// [i32], [i64], [f32], [f64], [String], ([Exportable], ...), [Option]<[Exportable]>, [[Exportable]], [Vec]<[Exportable]>
/// and some other types impled [Exportable]
pub trait Exportable {
    fn export(&self);
}
/// types which can be imported from computer craft's lua function call
///
/// [i32], [i64], [f32], [f64], [String], ([Importable], ...), [Option]<[Importable]>, [Vec]<[Importable]>
/// and some other types impled [Importable]
pub trait Importable: Sized {
    fn import() -> LuaResult<Self>;
}
pub(crate) mod debug {
    use super::lua_ffi::{addrof, ffi};
    #[allow(unused)]
    pub fn show_str(s: &str) {
        unsafe {
            ffi::show_str(addrof(s), s.len() as i32);
        }
    }
}

pub use lua_ffi::abort_next_import;
pub use lua_ffi::ffi::Typed;
pub use lua_ffi::next_import_type;
pub use lua_ffi::{failed, success};
pub(crate) mod lua_ffi {

    use ffi::Typed;

    use super::{LuaError, LuaResult};

    pub(crate) mod ffi {
        use std::fmt::Display;

        use crate::lua_api::{lua_result::LuaError, Importable};

        use super::next_import_type;

        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
        pub enum Typed {
            None,
            I32,
            I64,
            String,
            F32,
            F64,
            Type,
            Object,
            Nil,
            Bool,
            Error,
        }
        impl Typed {
            pub(crate) fn from_i32(t: i32) -> Self {
                match t {
                    0 => Typed::None,
                    1 => Typed::I32,
                    2 => Typed::I64,
                    3 => Typed::String,
                    4 => Typed::F32,
                    5 => Typed::F64,
                    6 => Typed::Type,
                    7 => Typed::Object,
                    8 => Typed::Nil,
                    9 => Typed::Bool,
                    _ => Typed::Error,
                }
            }
        }
        impl Display for Typed {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                let msg = match self {
                    Typed::None => "none",
                    Typed::I32 => "i32",
                    Typed::I64 => "i64",
                    Typed::String => "string",
                    Typed::F32 => "f32",
                    Typed::F64 => "f64",
                    Typed::Type => "type",
                    Typed::Object => "table",
                    Typed::Nil => "nil",
                    Typed::Bool => "bool",
                    Typed::Error => "error type",
                };
                write!(f, "{}", msg)?;
                Ok(())
            }
        }
        impl Importable for Typed {
            fn import() -> crate::lua_api::LuaResult<Self> {
                if next_import_type() != Typed::Type {
                    Err(LuaError::from_str("not receiving type"))?;
                }
                Ok(Typed::from_i32(unsafe { import_i32() }))
            }
        }
        #[allow(unused)]
        #[link(wasm_import_module = "host")]
        extern "C" {
            pub fn show_str(addr: i32, len: i32);
            pub fn next_type() -> i32;

            pub fn export_string(addr: i32, len: i32);
            pub fn import_string_length() -> i32;
            pub fn import_string_data(addr: i32);

            pub fn import_i32() -> i32;
            pub fn export_i32(data: i32);

            pub fn import_i64() -> i64;
            pub fn export_i64(data: i64);

            pub fn import_f32() -> f32;
            pub fn export_f32(data: f32);

            pub fn import_f64() -> f64;
            pub fn export_f64(data: f64);

            pub fn import_bool() -> i32;
            pub fn export_bool(data: i32);

            pub fn export_nil();

            pub fn abort_next_import();
            pub fn success();
            pub fn failed();
        }
    }
    pub fn next_import_type() -> Typed {
        Typed::from_i32(unsafe { ffi::next_type() })
    }
    pub fn addrof<T: ?Sized>(s: *const T) -> i32 {
        s as *const () as usize as i32
    }
    /// abort a import
    /// # Safety
    /// should be called carefully, otherwise this will cause weird behavior
    pub unsafe fn abort_next_import() {
        unsafe {
            ffi::abort_next_import();
        }
    }
    /// mark the function to be successfully runned
    /// # Safety
    /// should only be called at the end of a exported function.
    ///
    /// should not be manually called
    pub unsafe fn success() {
        unsafe {
            ffi::success();
        }
    }
    /// mark the function to be failed
    /// # Safety
    /// should only be called at the end of a exported function.
    ///
    /// should not be manually called
    pub unsafe fn failed() {
        unsafe {
            ffi::failed();
        }
    }
    pub fn assert_type(t: Typed) -> LuaResult<()> {
        let next = next_import_type();
        if t == next {
            Ok(())
        } else {
            Err(LuaError::from_string(format!("expect {}, got {}", t, next)))
        }
    }
}

pub mod nil {
    use std::fmt::Display;

    use super::{lua_ffi::ffi::export_nil, Exportable, Importable};

    /// a lua nil value
    ///
    /// # Difference with `()` and [Option::None]
    /// when `Nil` is return value, it will be a `nil` value, but `()` and [Option::None]
    /// will be simply nothing
    ///
    #[derive(Debug, Clone, Copy, Default, Hash, PartialEq, Eq, PartialOrd, Ord)]
    pub struct Nil;
    impl Display for Nil {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "nil")?;
            Ok(())
        }
    }
    impl Importable for Nil {
        fn import() -> super::LuaResult<Self> {
            <()>::import()?;
            Ok(Self)
        }
    }
    impl Exportable for Nil {
        fn export(&self) {
            unsafe {
                export_nil();
            }
        }
    }
}

mod io_impl_string {
    use super::{
        lua_ffi::{
            addrof, assert_type,
            ffi::{self, Typed},
        },
        lua_result::LuaError,
        Exportable, Importable, LuaResult,
    };

    impl Exportable for String {
        fn export(&self) {
            export_string(self);
        }
    }
    impl Exportable for str {
        fn export(&self) {
            export_string(self);
        }
    }
    impl Exportable for &str {
        fn export(&self) {
            export_string(self);
        }
    }
    impl Exportable for &String {
        fn export(&self) {
            export_string(self);
        }
    }
    impl Importable for String {
        fn import() -> LuaResult<Self> {
            import_string()
        }
    }

    fn export_string(s: &str) {
        unsafe {
            ffi::export_string(addrof(s), s.len() as i32);
        }
    }
    fn import_string() -> LuaResult<String> {
        // if next_import_type() != Typed::String {
        //     Err(LuaError::from_str("not receiving String"))?;
        // }
        assert_type(Typed::String)?;

        let mut a = vec![0u8; unsafe { ffi::import_string_length() } as usize];
        unsafe {
            let addr = a.as_mut_ptr();
            ffi::import_string_data(addr as *const u8 as usize as i32);
        }
        String::from_utf8(a).map_err(|_| LuaError::from_str("non utf8 string"))
    }
}

mod io_impl_number {
    use super::{
        lua_ffi::ffi::{
            export_f32, export_f64, export_i32, export_i64, import_f32, import_f64, import_i32,
            import_i64, Typed,
        },
        Exportable, Importable,
    };
    use crate::lua_api::lua_ffi::assert_type;
    macro_rules! impl_for {
        ($t:ty, $typname:ident, $if:ident, $of:ident) => {
            impl Importable for $t {
                fn import() -> super::LuaResult<Self> {
                    // if next_import_type() != Typed::$typname {
                    //     return Err(LuaError::from_str(concat!(
                    //         "not receiving ",
                    //         stringify!($t)
                    //     )))?;
                    // }
                    assert_type(Typed::$typname)?;
                    Ok(unsafe { $if() })
                }
            }
            impl Exportable for $t {
                fn export(&self) {
                    unsafe {
                        $of(*self);
                    }
                }
            }
        };
    }
    impl_for!(i32, I32, import_i32, export_i32);
    impl_for!(i64, I64, import_i64, export_i64);
    impl_for!(f32, F32, import_f32, export_f32);
    impl_for!(f64, F64, import_f64, export_f64);
}
fn _a() {
    concat!();
    stringify!();
}
mod io_impl_utils {

    use super::{abort_next_import, next_import_type, Exportable, Importable, Typed};

    impl Importable for () {
        fn import() -> super::LuaResult<Self> {
            if next_import_type() == Typed::Nil {
                unsafe { abort_next_import() };
            }
            Ok(())
        }
    }
    impl Exportable for () {
        fn export(&self) {}
    }

    impl<T: Importable> Importable for Option<T> {
        fn import() -> super::LuaResult<Self> {
            if next_import_type() == Typed::None {
                Ok(None)
            } else if next_import_type() == Typed::Nil {
                unsafe { abort_next_import() };
                Ok(None)
            } else {
                Ok(Some(Importable::import()?))
            }
        }
    }
    impl<T: Exportable> Exportable for Option<T> {
        fn export(&self) {
            match self {
                Some(v) => v.export(),
                None => {}
            }
        }
    }

    macro_rules! impl_tuple {
        ($($t:ident),*) => {
            impl<$($t:Importable),*> Importable for ($($t,)*){
                fn import() -> super::LuaResult<Self> {
                    Ok(( $(
                        $t::import()?,
                    )* ))
                }
            }

            impl<$($t:Exportable),*> Exportable for ($($t,)*){
                fn export(&self) {
                    #[allow(non_snake_case)]
                    let ($($t,)*) = self;
                    $(
                        $t.export();
                    )*
                }
            }
        };
    }
    impl_tuple!(T0);
    impl_tuple!(T0, T1);
    impl_tuple!(T0, T1, T2);
    impl_tuple!(T0, T1, T2, T3);
    impl_tuple!(T0, T1, T2, T3, T4);
    impl_tuple!(T0, T1, T2, T3, T4, T5);
    impl_tuple!(T0, T1, T2, T3, T4, T5, T6);
    impl_tuple!(T0, T1, T2, T3, T4, T5, T6, T7);
    impl_tuple!(T0, T1, T2, T3, T4, T5, T6, T7, T8);
    impl_tuple!(T0, T1, T2, T3, T4, T5, T6, T7, T8, Y9);
    impl_tuple!(T0, T1, T2, T3, T4, T5, T6, T7, T8, Y9, T10);
    impl_tuple!(T0, T1, T2, T3, T4, T5, T6, T7, T8, Y9, T10, T11);
    impl_tuple!(T0, T1, T2, T3, T4, T5, T6, T7, T8, Y9, T10, T11, T12);
    impl_tuple!(T0, T1, T2, T3, T4, T5, T6, T7, T8, Y9, T10, T11, T12, T13);

    impl<T: Importable> Importable for Vec<T> {
        fn import() -> super::LuaResult<Self> {
            let mut v = Vec::new();
            loop {
                if next_import_type() != Typed::None {
                    v.push(T::import()?);
                } else {
                    break;
                }
            }
            Ok(v)
        }
    }
    // impl<T: Importable + Default> Importable for [T; const { ("s", 1).1 + (1) }] {
    //     fn import() -> super::LuaResult<Self> {
    //         Ok([T::import()?, T::import()?])
    //     }
    // }
    macro_rules! impl_arr {
        ($($t:ident),*) => {
            impl<T: Importable + Default> Importable for [T; const {$((stringify!($t), 1).1 + ) * 0}] {
                fn import() -> super::LuaResult<Self> {
                    Ok([$(
                        $t::import()?
                    ),*])
                }
            }
        };
    }
    impl_arr!(T);
    impl_arr!(T, T);
    impl_arr!(T, T, T);
    impl_arr!(T, T, T, T);
    impl_arr!(T, T, T, T, T);
    impl_arr!(T, T, T, T, T, T);
    impl_arr!(T, T, T, T, T, T, T);
    impl_arr!(T, T, T, T, T, T, T, T);
    impl_arr!(T, T, T, T, T, T, T, T, T);
    impl_arr!(T, T, T, T, T, T, T, T, T, T);
    impl_arr!(T, T, T, T, T, T, T, T, T, T, T);
    impl_arr!(T, T, T, T, T, T, T, T, T, T, T, T);
    impl_arr!(T, T, T, T, T, T, T, T, T, T, T, T, T);
    impl_arr!(T, T, T, T, T, T, T, T, T, T, T, T, T, T);
    impl_arr!(T, T, T, T, T, T, T, T, T, T, T, T, T, T, T);
    impl_arr!(T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T);
    impl_arr!(T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T);
    impl_arr!(T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T);
    impl_arr!(T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T);
    impl_arr!(T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T);
    impl_arr!(T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T);
    impl_arr!(T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T);
    impl_arr!(T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T);
    impl_arr!(T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T);
    impl_arr!(T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T);
    impl_arr!(T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T);
    impl_arr!(T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T);
    impl_arr!(T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T);
    impl_arr!(
        T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T
    );
    impl_arr!(
        T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T
    );
    impl_arr!(
        T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T
    );
    impl_arr!(
        T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T, T,
        T, T
    );

    impl<T: Exportable> Exportable for [T] {
        fn export(&self) {
            for i in self {
                i.export();
            }
        }
    }
    impl<T: Exportable> Exportable for &[T] {
        fn export(&self) {
            (*self).export();
        }
    }
    impl<T: Exportable> Exportable for Vec<T> {
        fn export(&self) {
            self[..].export();
        }
    }
    impl<T: Exportable> Exportable for &Vec<T> {
        fn export(&self) {
            self[..].export();
        }
    }
    impl<T: Exportable, const L: usize> Exportable for [T; L] {
        fn export(&self) {
            self[..].export();
        }
    }
    impl<T: Exportable, const L: usize> Exportable for &[T; L] {
        fn export(&self) {
            self[..].export();
        }
    }
}

mod io_impl_bool {
    use super::{
        lua_ffi::{
            assert_type,
            ffi::{export_bool, import_bool},
        },
        Exportable, Importable, Typed,
    };

    impl Importable for bool {
        fn import() -> super::LuaResult<Self> {
            assert_type(Typed::Bool)?;
            Ok((unsafe { import_bool() }) != 0)
        }
    }
    impl Exportable for bool {
        fn export(&self) {
            unsafe {
                export_bool(if *self { 1 } else { 0 });
            }
        }
    }
}
