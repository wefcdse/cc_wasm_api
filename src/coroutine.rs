use std::{
    cell::RefCell,
    collections::VecDeque,
    future::Future,
    mem::{self, ManuallyDrop},
    num::NonZeroUsize,
    pin::Pin,
    ptr,
    task::{Context, Poll, RawWaker, RawWakerVTable, Waker},
    time::{Duration, Instant},
};

use crate::{
    lua_api::Importable,
    utils::{Number, SyncNonSync},
};

#[allow(unused)]
mod fut_blocker {
    use std::{cell::Cell, future::Future, task::Context};

    pub struct Stopper(Cell<bool>);
    impl Default for Stopper {
        fn default() -> Self {
            Self::new()
        }
    }
    impl Stopper {
        pub const fn new() -> Self {
            Self(Cell::new(false))
        }
        pub const fn stopper(&self) -> impl '_ + Fn() -> bool {
            || self.0.get()
        }
        pub fn stop(&self) {
            self.0.set(true);
        }
    }

    use super::do_nothing_waker;

    pub trait BlocedFuture: Future {
        fn block_on(self) -> Self::Output;
        fn block_on_limited(self, count: usize) -> Option<Self::Output>;
        fn block_on_stopable(self, stop: impl FnMut() -> bool) -> Option<Self::Output>;
    }
    impl<F: Future> BlocedFuture for F {
        fn block_on(self) -> Self::Output {
            block_on_no_alloc(self)
        }

        fn block_on_limited(self, count: usize) -> Option<Self::Output> {
            block_on_no_alloc_limited(self, count)
        }

        fn block_on_stopable(self, stop: impl FnMut() -> bool) -> Option<Self::Output> {
            block_on_no_alloc_stopable(self, stop)
        }
    }

    pub fn block_on_no_alloc<F: Future>(f: F) -> F::Output {
        let w = do_nothing_waker();
        let mut c = Context::from_waker(&w);
        let mut pined = core::pin::pin!(f);
        loop {
            match pined.as_mut().poll(&mut c) {
                core::task::Poll::Ready(v) => {
                    break v;
                }
                core::task::Poll::Pending => {
                    // core::hint::spin_loop();
                    continue;
                }
            }
        }
    }
    pub fn block_on_no_alloc_limited<F: Future>(f: F, count: usize) -> Option<F::Output> {
        let w = do_nothing_waker();
        let mut c = Context::from_waker(&w);
        let mut pined = core::pin::pin!(f);
        let mut polled = 0;
        loop {
            match pined.as_mut().poll(&mut c) {
                core::task::Poll::Ready(v) => break Some(v),
                core::task::Poll::Pending => {
                    polled += 1;
                    if polled > count {
                        return None;
                    }
                    // core::hint::spin_loop();
                    continue;
                }
            }
        }
    }
    pub fn block_on_no_alloc_stopable<F: Future>(
        f: F,
        mut stop: impl FnMut() -> bool,
    ) -> Option<F::Output> {
        let w = do_nothing_waker();
        let mut c = Context::from_waker(&w);
        let mut pined = core::pin::pin!(f);

        loop {
            match pined.as_mut().poll(&mut c) {
                core::task::Poll::Ready(v) => {
                    // dbg!("R");
                    break Some(v);
                }
                core::task::Poll::Pending => {
                    // dbg!("P");
                    if stop() {
                        return None;
                    }
                    // core::hint::spin_loop();
                    continue;
                }
            }
        }
    }
}

type TaskCtx = SyncNonSync<RefCell<Vec<Pin<Box<dyn Future<Output = ()>>>>>>;
static COROUTINES: TaskCtx = SyncNonSync(RefCell::new(Vec::new()));
static SPAWNED: TaskCtx = SyncNonSync(RefCell::new(Vec::new()));

#[cfg(target_os = "unknown")]
#[no_mangle]
pub extern "C" fn tick() {
    COROUTINES
        .borrow_mut()
        .append(SPAWNED.borrow_mut().as_mut());
    let mut workload = COROUTINES.0.borrow_mut();
    let w = do_nothing_waker();
    let mut c = Context::from_waker(&w);

    // let new = workload.drain(..).into_iter().filter_map();
    workload.retain_mut(|a| {
        let pinned = a.as_mut();
        pinned.poll(&mut c).is_pending()
    });
}
#[cfg(not(target_os = "unknown"))]
#[no_mangle]
pub extern "C" fn tick() {
    COROUTINES
        .borrow_mut()
        .append(SPAWNED.borrow_mut().as_mut());
    let timeout = Option::<Number>::import()
        .unwrap_or(None)
        .unwrap_or(Number::Int(0))
        .to_f64();
    let start = Instant::now();
    let mut workload = COROUTINES.0.borrow_mut();
    let w = do_nothing_waker();
    let mut c = Context::from_waker(&w);

    // let new = workload.drain(..).into_iter().filter_map();
    loop {
        workload.retain_mut(|a| {
            let pinned = a.as_mut();
            pinned.poll(&mut c).is_pending()
        });
        if start.elapsed().as_secs_f64() >= timeout {
            break;
        }
    }
}
/// spawns a coroutine, which will be executed in tick function.
pub fn spawn(fut: impl 'static + Future<Output = ()>) {
    SPAWNED.borrow_mut().push(Box::pin(fut));
}
/// returns the running coroutines.
pub fn coroutines() -> usize {
    COROUTINES.borrow().len()
}

/// yield from the coroutine, allow the executor to run other coroutine.
// #[cfg(not(feature = "eval"))]
pub fn yield_now() -> impl Future<Output = ()> {
    struct Y(bool);
    impl Future for Y {
        type Output = ();

        fn poll(
            mut self: core::pin::Pin<&mut Self>,
            cx: &mut Context<'_>,
        ) -> core::task::Poll<Self::Output> {
            let ok = self.0;
            if ok {
                core::task::Poll::Ready(())
            } else {
                self.0 = true;
                cx.waker().wake_by_ref();
                core::task::Poll::Pending
            }
        }
    }
    Y(false)
}

#[cfg(not(target_os = "unknown"))]
#[cfg_attr(docsrs, doc(cfg(not(target_os = "unknown"))))]
/// sleep for some time.
pub fn sleep(time: Duration) -> impl Future<Output = ()> {
    struct Sleep(Duration, Instant);
    impl Future for Sleep {
        type Output = ();

        fn poll(
            self: std::pin::Pin<&mut Self>,
            cx: &mut std::task::Context<'_>,
        ) -> std::task::Poll<Self::Output> {
            if self.1.elapsed() > self.0 {
                Poll::Ready(())
            } else {
                cx.waker().wake_by_ref();
                Poll::Pending
            }
        }
    }
    Sleep(time, Instant::now())
}

fn do_nothing_waker() -> Waker {
    fn raw_waker_clone(_v: *const ()) -> RawWaker {
        RawWaker::new(
            ptr::NonNull::dangling().as_ptr(),
            &RawWakerVTable::new(raw_waker_clone, do_nothing, do_nothing, do_nothing),
        )
    }
    fn do_nothing(_v: *const ()) {}
    let w = RawWaker::new(
        ptr::NonNull::dangling().as_ptr(),
        &RawWakerVTable::new(raw_waker_clone, do_nothing, do_nothing, do_nothing),
    );
    // SAFETY: *const () is never derefed, and raw_waker_clone returns exactly same as w
    unsafe { Waker::from_raw(w) }
}

/// a simple channel which is not sync, but it can push and pop both in async and non-async.
#[derive(Default)]
pub struct UnsyncChannel<V> {
    cell: RefCell<VecDeque<V>>,
    limit: Option<NonZeroUsize>,
}

impl<V> UnsyncChannel<V> {
    pub const fn new(limit: usize) -> Self {
        Self {
            cell: RefCell::new(VecDeque::new()),
            limit: NonZeroUsize::new(limit),
        }
    }
    pub fn try_insert(&self, value: V) -> Option<V> {
        if let Some(limit) = self.limit {
            let mut vec = self.cell.borrow_mut();
            if vec.len() >= limit.get() {
                Some(value)
            } else {
                vec.push_back(value);
                None
            }
        } else {
            self.cell.borrow_mut().push_back(value);
            None
        }
    }
    pub fn try_get(&self) -> Option<V> {
        self.cell.borrow_mut().pop_front()
    }
    pub fn get(&self) -> impl '_ + Future<Output = V> {
        struct Get<'a, O>(&'a UnsyncChannel<O>);
        impl<'a, O> Future for Get<'a, O> {
            type Output = O;

            fn poll(
                self: core::pin::Pin<&mut Self>,
                cx: &mut Context<'_>,
            ) -> core::task::Poll<Self::Output> {
                let s = self.0;
                if let Some(v) = s.try_get() {
                    Poll::Ready(v)
                } else {
                    cx.waker().wake_by_ref();
                    Poll::Pending
                }
            }
        }
        // async {
        //     loop {
        //         if let Some(v) = self.try_get() {
        //             break v;
        //         }
        //         yield_now().await;
        //     }
        // }
        Get(self)
    }
    pub fn insert(&self, value: V) -> impl '_ + Future<Output = ()> {
        struct Insert<'a, T>(Option<&'a UnsyncChannel<T>>, ManuallyDrop<T>);
        impl<'a, T> Future for Insert<'a, T> {
            type Output = ();

            fn poll(self: core::pin::Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
                // SAFETY:因为Self包含Option<&'a UnsyncChannel<T>>，所以不Send不Sync不会被跨线程调用，只需要考虑单线程就可以了。
                // SAFETY:不管v是不是Unpin但在这里只是作为一个值来传递的不会改变而且本来就是move进来的，原来就没pin住
                let Self(a, b) = unsafe { self.get_unchecked_mut() };
                let channel = if let Some(v) = a {
                    v
                } else {
                    panic!("should not be polled again after done");
                };
                // SAFETY:之后这个会被forget掉的，如果插入成功了Self.0会被置None供Drop判断
                let v1 = unsafe { ManuallyDrop::take(b) };
                if let Some(v) = channel.try_insert(v1) {
                    mem::forget(v);
                    cx.waker().wake_by_ref();
                    Poll::Pending
                } else {
                    *a = None;
                    Poll::Ready(())
                }
            }
        }

        impl<'a, T> Drop for Insert<'a, T> {
            fn drop(&mut self) {
                // SAFETY:self.0会指示是否完成。如果已经完成了就不drop了。
                if self.0.is_some() {
                    unsafe { ManuallyDrop::drop(&mut self.1) };
                }
            }
        }
        // async {
        //     let mut value = value;
        //     loop {
        //         if let Some(v) = self.try_insert(value) {
        //             value = v;
        //         } else {
        //             break;
        //         }
        //         yield_now().await;
        //     }
        // }
        Insert(Some(self), ManuallyDrop::new(value))
    }
}
