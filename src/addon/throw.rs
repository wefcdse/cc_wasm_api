use std::{fmt::Display, future::Future};

use crate::{debug::show_str, eval::exec, lua_api::LuaError};

pub trait Throw<T> {
    fn throw(self) -> impl Future<Output = T>;
    fn throw_with_info(self, file_line: (&'static str, u32)) -> impl Future<Output = T>;
}
#[macro_export]
macro_rules! info {
    () => {
        (::std::file!(), ::std::line!())
    };
}
#[macro_export]
macro_rules! throw {
    ($msg:literal) => {{
        let _: () = $crate::addon::throw::Throw::throw_with_info(
            Err($msg),
            (::std::file!(), ::std::line!()),
        )
        .await;
    };};
    ($e:expr) => {
        $crate::addon::throw::Throw::throw_with_info($e, (::std::file!(), ::std::line!())).await
    };
}

impl<T, E: Display> Throw<T> for Result<T, E> {
    async fn throw(self) -> T {
        match self {
            Ok(t) => t,
            Err(e) => {
                let msg = format!("{}", e);
                // let msg = format!("{}\nbacktrace:\n{}", e, Backtrace::capture());
                let script = &format!("error({:?})", msg);
                show_str(script);
                exec(script).await.unwrap();
                panic!()
            }
        }
    }

    async fn throw_with_info(self, (file, line): (&'static str, u32)) -> T {
        match self {
            Ok(t) => t,
            Err(e) => {
                let msg = format!("{file}:{line}\n{}", e);
                // let msg = format!("{}\nbacktrace:\n{}", e, Backtrace::capture());
                let script = &format!("error({:?})", msg);
                show_str(script);
                exec(script).await.unwrap();
                panic!()
            }
        }
    }
}

impl<T> Throw<T> for Result<T, LuaError> {
    async fn throw(self) -> T {
        match self {
            Ok(t) => t,
            Err(e) => {
                let msg = e.0.into_owned();
                // let msg = format!("{}\nbacktrace:\n{}", e.as_str(), Backtrace::capture());
                let script = &format!("error({:?})", msg);
                show_str(script);
                exec(script).await.unwrap();
                panic!()
            }
        }
    }
    async fn throw_with_info(self, (file, line): (&'static str, u32)) -> T {
        match self {
            Ok(t) => t,
            Err(e) => {
                let msg = format!("{file}:{line}\n{}", e.as_str());
                // let msg = format!("{}\nbacktrace:\n{}", e, Backtrace::capture());
                let script = &format!("error({:?})", msg);
                show_str(script);
                exec(script).await.unwrap();
                panic!()
            }
        }
    }
}
