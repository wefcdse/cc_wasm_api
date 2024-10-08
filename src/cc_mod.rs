use crate::{
    lua_api::{failed, success, Exportable, Importable, LuaResult},
    utils::{ImplResult, ImplValue},
};
/// used to run this function with cc_wasm mod
/// # Safety
/// should not be manually impled
pub unsafe trait ExportFunc<Args, Out, ImplType> {
    /// used to run this function with cc_wasm mod
    /// # Safety
    /// should not be manually called
    unsafe fn call(&self);
}

/// export functions to the `cc wasm` mod.
///
/// usage: `export_funcs!((fn1, export_name1), (fn2, export_name2))` or `export_funcs!(fn1, fn2, fn3)`
///
/// the function's arguments must be [Importable], and
/// the function's return value must be [Exportable] or [LuaResult]<[Exportable]>
#[macro_export]
macro_rules! export_funcs {
    ($(($f:ident, $ename:ident)),*) => {
       const _:() = {
        mod inner{
            $(

                #[no_mangle]
                pub extern "C" fn $ename(){
                    use super::$f;
                    unsafe { $crate::cc_mod::ExportFunc::call(&$f); }
                }
            )*

            #[no_mangle]
            pub extern "C" fn export_func() {
                unsafe { $crate::lib_exports(); }
                $(
                    $crate::lua_api::Exportable::export(::core::stringify!($ename));
                )*
            }};
        };

    };
    ($($f:ident),*) => {
        const _:() = {
        mod inner{
            $(

                #[no_mangle]
                pub extern "C" fn $f(){
                    use super::$f;
                    unsafe { $crate::cc_mod::ExportFunc::call(&$f); }
                }
            )*

            #[no_mangle]
            pub extern "C" fn export_func() {
                unsafe { $crate::lib_exports(); }
                $(
                    $crate::lua_api::Exportable::export(::core::stringify!($f));
                )*
            }};
        };

    };

}
// #[no_mangle]
// pub extern "C" fn export_func() {
//     stringify!(a).export();
// }
macro_rules! impl_export {
    ($($t:ident),*) => {
        unsafe impl<$($t: Importable,)* O: Exportable, F: Fn($($t),*) -> LuaResult<O>>
            ExportFunc<($($t,)*), O, ImplResult> for F
        {
            unsafe fn call(&self) {
                let o: LuaResult<O> = (|| {
                    $(
                        #[allow(non_snake_case)]
                        let $t = $t::import()?;
                    )*
                    let out = self($($t),*)?;

                    Ok(out)
                })();
                match o {
                    Ok(o) => {
                        unsafe { success(); };
                        o.export();
                    }
                    Err(err) => {
                        unsafe { failed(); };
                        err.as_str().export();
                    }
                }
            }
        }

        unsafe impl<$($t: Importable,)* O: Exportable, F: Fn($($t),*) -> O>
            ExportFunc<($($t,)*), O, ImplValue> for F
        {
            unsafe fn call(&self) {
                let o: LuaResult<O> = (|| {
                    $(
                        #[allow(non_snake_case)]
                        let $t = $t::import()?;
                    )*
                    let out = self($($t),*);

                    Ok(out)
                })();
                match o {
                    Ok(o) => {
                        unsafe { success(); };
                        o.export();
                    }
                    Err(err) => {
                        unsafe { failed() };
                        err.as_str().export();
                    }
                }
            }
        }
    };
}
unsafe impl<O: Exportable, F: Fn() -> LuaResult<O>> ExportFunc<(), O, ImplResult> for F {
    unsafe fn call(&self) {
        match self() {
            Ok(o) => {
                unsafe { success() };
                o.export();
            }
            Err(err) => {
                unsafe { failed() };
                err.as_str().export();
            }
        }
    }
}
unsafe impl<O: Exportable, F: Fn() -> O> ExportFunc<(), O, ImplValue> for F {
    unsafe fn call(&self) {
        let o = self();
        unsafe { success() };
        o.export();
    }
}

impl_export!(T0);
impl_export!(T0, T1);
impl_export!(T0, T1, T2);
impl_export!(T0, T1, T2, T3);
impl_export!(T0, T1, T2, T3, T4);
impl_export!(T0, T1, T2, T3, T4, T5);
impl_export!(T0, T1, T2, T3, T4, T5, T6);
impl_export!(T0, T1, T2, T3, T4, T5, T6, T7);
impl_export!(T0, T1, T2, T3, T4, T5, T6, T7, T8);
impl_export!(T0, T1, T2, T3, T4, T5, T6, T7, T8, Y9);
impl_export!(T0, T1, T2, T3, T4, T5, T6, T7, T8, Y9, T10);
impl_export!(T0, T1, T2, T3, T4, T5, T6, T7, T8, Y9, T10, T11);
impl_export!(T0, T1, T2, T3, T4, T5, T6, T7, T8, Y9, T10, T11, T12);
impl_export!(T0, T1, T2, T3, T4, T5, T6, T7, T8, Y9, T10, T11, T12, T13);
