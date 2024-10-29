#[macro_export]
macro_rules! time {
    ($st:ident) => {
        let $st = ::std::time::Instant::now();
    };
    ($st:ident, $info:literal) => {
        $crate::throw_exec!(&::std::format!(
            "print({:?})",
            ::std::format!("{}:{:?}", $info, ::std::time::Instant::elapsed(&$st))
        ));
    };
}
