use std::{future::Future, marker::PhantomData, task::Poll};

use crate::lua_api::{Importable, LuaResult};

mod ffi {
    #[allow(unused)]
    #[link(wasm_import_module = "host")]
    extern "C" {

        pub fn call_eval(addr: i32, len: i32) -> i32;
        pub fn eval_ready() -> i32;
        pub fn clear_eval();
        pub fn import_from_eval();
    }
}

fn call_eval(s: &str) -> bool {
    let a = unsafe { ffi::call_eval(s as *const str as *const () as usize as i32, s.len() as i32) };
    a != 0
}
fn eval_ready() -> bool {
    let a = unsafe { ffi::eval_ready() };
    a != 0
}

struct Eval<'a, O: Importable + Unpin> {
    data: Option<&'a str>,
    out: PhantomData<O>,
}
impl<'a, O: Importable + Unpin> Future for Eval<'a, O> {
    type Output = LuaResult<O>;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let unpin = self.get_mut();

        if let Some(v) = &unpin.data {
            if call_eval(v) {
                unpin.data = None;
            } else {
                cx.waker().wake_by_ref();
                return Poll::Pending;
            }
        }

        if !eval_ready() {
            cx.waker().wake_by_ref();
            return Poll::Pending;
        }
        unsafe {
            ffi::import_from_eval();
        }
        let o = O::import();
        unsafe {
            ffi::clear_eval();
        }
        Poll::Ready(o)
    }
}

/// run a lua script in the lua context
pub fn eval<O: Importable + Unpin + 'static>(s: &str) -> impl '_ + Future<Output = LuaResult<O>> {
    Eval {
        data: Some(s),
        out: PhantomData,
    }
}
/// run a lua script in the lua context and returns nothing
pub fn exec(s: &str) -> impl '_ + Future<Output = LuaResult<()>> {
    Eval {
        data: Some(s),
        out: PhantomData,
    }
}
/// yield from the lua loop.
pub async fn yield_lua() {
    use crate::lua_api::LuaResult;
    let _: LuaResult<()> = crate::eval::eval("sleep(0)").await;
}
