use std::{
    cell::{Cell, RefCell},
    collections::VecDeque,
    future::Future,
    mem::{self, ManuallyDrop},
    num::NonZeroUsize,
    pin::Pin,
    ptr,
    task::{Context, Poll, RawWaker, RawWakerVTable, Waker},
    time::{Duration, Instant},
};

use fut_blocker::Stopper;

pub use stop::stop;

mod stop {
    use std::cell::Cell;

    use crate::{lua_api::success, prelude::Exportable, utils::SyncNonSync};

    static STOPPED: SyncNonSync<Cell<bool>> = SyncNonSync(Cell::new(false));
    pub(crate) fn get_stopped() -> bool {
        STOPPED.get()
    }
    pub fn stop() {
        STOPPED.set(true);
    }
    #[no_mangle]
    pub extern "C" fn stopped() {
        unsafe {
            success();
        }
        get_stopped().export();
    }
}

use crate::{
    lua_api::Importable,
    utils::{Number, SyncNonSync},
};

pub use async_lock::{AsyncLock, AsyncLockGuard};
mod async_lock {
    use std::{
        cell::{Cell, UnsafeCell},
        future::Future,
        ops::{Deref, DerefMut},
        task::Poll,
    };

    pub struct AsyncLock<T> {
        data: UnsafeCell<T>,
        locked: Cell<bool>,
    }
    pub struct AsyncLockGuard<'a, T> {
        target: &'a AsyncLock<T>,
    }
    impl<T> Deref for AsyncLockGuard<'_, T> {
        type Target = T;

        fn deref(&self) -> &Self::Target {
            unsafe { &*self.target.data.get() }
        }
    }
    impl<T> DerefMut for AsyncLockGuard<'_, T> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            unsafe { &mut *self.target.data.get() }
        }
    }
    impl<T> Drop for AsyncLockGuard<'_, T> {
        fn drop(&mut self) {
            self.target.locked.set(false);
        }
    }
    impl<T> AsyncLock<T> {
        pub const fn new(value: T) -> Self {
            Self {
                data: UnsafeCell::new(value),
                locked: Cell::new(false),
            }
        }
        pub fn lock(&self) -> impl '_ + Future<Output = AsyncLockGuard<'_, T>> {
            struct Lock<'a, T>(&'a AsyncLock<T>);
            impl<'a, T> Future for Lock<'a, T> {
                type Output = AsyncLockGuard<'a, T>;

                fn poll(
                    self: std::pin::Pin<&mut Self>,
                    cx: &mut std::task::Context<'_>,
                ) -> std::task::Poll<Self::Output> {
                    if self.0.locked.get() {
                        cx.waker().wake_by_ref();
                        Poll::Pending
                    } else {
                        self.0.locked.set(true);
                        Poll::Ready(AsyncLockGuard { target: self.0 })
                    }
                }
            }
            Lock(self)
        }
    }
}

pub use tick_sync::TickSyncer;
mod tick_sync {
    use std::{
        cell::{Cell, RefCell},
        fmt::Debug,
        future::Future,
        task::Poll,
        time::Duration,
    };

    use crate::{
        coroutine::{fut_blocker::Stopper, CoroutineSpawn as _},
        eval::yield_lua,
        utils::SyncNonSync,
    };

    use super::sleep;
    /// limit the corountine's loop to run exactly once every tick.
    ///
    /// # Example
    /// ```no_run
    /// use cc_wasm_api::prelude::*;
    /// fn init(){
    ///     TickSyncer::spawn_handle_coroutine();
    ///     async {
    ///         let mut ts = TickSyncer::new();
    ///         loop{
    ///             // code here will run once every tick
    ///             ts.sync().await;
    ///         }
    ///     }.spawn();
    ///
    ///     async {
    ///         let mut ts = TickSyncer::new();
    ///         // if not disable sync will cause other coroutines to wait for this sleep to finish
    ///         let no_sync = ts.no_sync();
    ///         sleep(Duration::from_sec(5)).await;
    ///         drop(no_sync);
    ///         loop{
    ///             // code here will run once every tick
    ///             ts.sync().await;
    ///         }
    ///     }.spawn();
    ///
    ///     async {
    ///         let mut ts = TickSyncer::new();
    ///         ts.sleep(Duration::from_sec(5)).await;
    ///         loop{
    ///             // code here will run once every tick
    ///             ts.sync().await;
    ///         }
    ///     }.spawn();
    /// }
    ///
    /// ```
    pub struct TickSyncer;
    impl TickSyncer {
        #[allow(clippy::new_without_default)]
        pub fn new() -> Self {
            subscribe();
            Self
        }
        /// limit the corountine's loop to run exactly once every tick.
        pub fn sync(&mut self) -> impl '_ + Future<Output = ()> {
            tick_sync()
        }

        /// temporarily disable sync, allow other corountine to run
        pub fn no_sync(&mut self) -> impl '_ + Drop {
            struct NoSync;
            impl Drop for NoSync {
                fn drop(&mut self) {
                    subscribe();
                }
            }
            desubscribe();
            NoSync
        }
        pub async fn sleep(&mut self, dur: Duration) {
            desubscribe();
            sleep(dur).await;
            subscribe();
        }

        /// handle the sync
        /// # Safety
        /// should be called every tick in one and only one corountine which didn't subscribed TickSyncer
        pub unsafe fn handle_sync() -> impl Future<Output = ()> {
            tick_sync_handle()
        }
        pub fn spawn_handle_coroutine() -> Stopper {
            static RUNNED: SyncNonSync<RefCell<Option<Stopper>>> = SyncNonSync(RefCell::new(None));

            if RUNNED.borrow().is_none() {
                let stop = async {
                    loop {
                        unsafe { TickSyncer::handle_sync() }.await;
                        yield_lua().await;
                    }
                }
                .spawn();
                *RUNNED.borrow_mut() = Some(stop.clone());
                stop
            } else {
                RUNNED.borrow().as_ref().unwrap().clone()
            }
        }
    }
    impl Drop for TickSyncer {
        fn drop(&mut self) {
            desubscribe();
        }
    }

    #[derive(Debug, Default)]
    struct TickSyncCtx {
        subscribed: Cell<usize>,
        epoch: Cell<usize>,
        runned_this_epoch: Cell<usize>,
    }

    static TICK_SYNCER: SyncNonSync<TickSyncCtx> = SyncNonSync(TickSyncCtx {
        subscribed: Cell::new(0),
        epoch: Cell::new(0),
        runned_this_epoch: Cell::new(0),
    });
    fn subscribe() {
        TICK_SYNCER.subscribed.set(TICK_SYNCER.subscribed.get() + 1);
    }
    fn desubscribe() {
        TICK_SYNCER.subscribed.set(TICK_SYNCER.subscribed.get() - 1);
    }
    fn increase_epoch() {
        TICK_SYNCER.epoch.set(TICK_SYNCER.epoch.get() + 1);
    }
    fn clear_run() {
        TICK_SYNCER.runned_this_epoch.set(0);
    }
    fn increase_run() {
        TICK_SYNCER
            .runned_this_epoch
            .set(TICK_SYNCER.runned_this_epoch.get() + 1);
    }
    fn epoch() -> usize {
        TICK_SYNCER.epoch.get()
    }
    fn tick_sync_handle() -> impl Future<Output = ()> {
        struct TickSyncHandle;
        impl Future for TickSyncHandle {
            type Output = ();

            fn poll(
                self: std::pin::Pin<&mut Self>,
                cx: &mut std::task::Context<'_>,
            ) -> Poll<Self::Output> {
                let runned = TICK_SYNCER.runned_this_epoch.get();
                let subscribed = TICK_SYNCER.subscribed.get();
                if runned >= subscribed {
                    clear_run();
                    increase_epoch();
                    Poll::Ready(())
                } else {
                    cx.waker().wake_by_ref();
                    Poll::Pending
                }
            }
        }
        TickSyncHandle
    }
    fn tick_sync() -> impl Future<Output = ()> + Debug {
        #[derive(Debug, Clone, Copy)]
        struct TickSync {
            epoch: usize,
            runned: bool,
        }
        impl Future for TickSync {
            type Output = ();

            fn poll(
                self: std::pin::Pin<&mut Self>,
                cx: &mut std::task::Context<'_>,
            ) -> Poll<Self::Output> {
                let unpin = self.get_mut();

                if unpin.epoch < epoch() {
                    Poll::Ready(())
                } else {
                    if !unpin.runned {
                        increase_run();
                        unpin.runned = true;
                    }
                    cx.waker().wake_by_ref();
                    Poll::Pending
                }
            }
        }

        TickSync {
            epoch: epoch(),
            runned: false,
        }
    }
}

#[allow(unused)]
mod fut_blocker {
    use std::{cell::Cell, future::Future, rc::Rc, task::Context};

    #[derive(Debug, Clone)]
    pub struct Stopper(Rc<Cell<bool>>);
    // impl Default for Stopper {
    //     fn default() -> Self {
    //         Self::new()
    //     }
    // }
    impl Stopper {
        pub(crate) fn new() -> Self {
            Self(Rc::new(Cell::new(false)))
        }
        pub(crate) const fn stopper(&self) -> impl '_ + Fn() -> bool {
            || self.0.get()
        }
        pub(crate) fn stoped(&self) -> bool {
            self.0.get()
        }
        pub(crate) fn not_stopped(&self) -> bool {
            !self.0.get()
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

type TaskCtx = SyncNonSync<RefCell<Vec<Task>>>;
struct Task {
    fut: Pin<Box<dyn Future<Output = ()>>>,
    stopper: Stopper,
}

static COROUTINES: TaskCtx = SyncNonSync(RefCell::new(Vec::new()));
static SPAWNED: TaskCtx = SyncNonSync(RefCell::new(Vec::new()));
static YIELD_COUNTER: SyncNonSync<Cell<usize>> = SyncNonSync(Cell::new(0));
static ACTIVE_COROUTINE_COUNT: SyncNonSync<Cell<usize>> = SyncNonSync(Cell::new(0));
fn increase_yield_counter() {
    YIELD_COUNTER.set(YIELD_COUNTER.get() + 1);
}
pub fn clear_yield_counter() {
    YIELD_COUNTER.set(0);
}
pub fn yield_counter() -> usize {
    YIELD_COUNTER.get()
}

#[cfg(target_os = "unknown")]
#[no_mangle]
pub extern "C" fn tick() {
    use stop::stop;

    COROUTINES
        .borrow_mut()
        .append(SPAWNED.borrow_mut().as_mut());
    let mut workload = COROUTINES.0.borrow_mut();
    let w = do_nothing_waker();
    let mut c = Context::from_waker(&w);

    // let new = workload.drain(..).into_iter().filter_map();
    workload.retain_mut(|a| {
        let pinned = a.fut.as_mut();
        pinned.poll(&mut c).is_pending() && a.stopper.not_stopped()
    });

    ACTIVE_COROUTINE_COUNT.set(workload.len());
    if coroutines() == 0 {
        stop();
    }
}
#[cfg(not(target_os = "unknown"))]
#[no_mangle]
extern "C" fn tick() {
    use stop::stop;

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
            let pinned = a.fut.as_mut();
            pinned.poll(&mut c).is_pending() && a.stopper.not_stopped()
        });
        if start.elapsed().as_secs_f64() >= timeout {
            break;
        }
    }
    ACTIVE_COROUTINE_COUNT.set(workload.len());
    if coroutines() == 0 {
        stop();
    }
}
/// spawns a coroutine, which will be executed in tick function.
pub trait CoroutineSpawn {
    /// spawns a coroutine, which will be executed in tick function.
    fn spawn(self) -> Stopper;
}
impl<F: 'static + Future> CoroutineSpawn for F {
    fn spawn(self) -> Stopper {
        spawn(self)
    }
}

/// spawns a coroutine, which will be executed in tick function.
pub fn spawn(fut: impl 'static + Future) -> Stopper {
    struct IgnoreRtnFut<F: Future>(F);
    impl<F: Future> Future for IgnoreRtnFut<F> {
        type Output = ();

        fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
            // SAFETY: self is pinned and self.0 is also pinned
            let pinned = unsafe { self.map_unchecked_mut(|f| &mut f.0) };
            match pinned.poll(cx) {
                Poll::Ready(_) => Poll::Ready(()),
                Poll::Pending => Poll::Pending,
            }
        }
    }
    let stopper = Stopper::new();
    let task = Task {
        fut: Box::pin(IgnoreRtnFut(fut)),
        stopper: stopper.clone(),
    };

    SPAWNED.borrow_mut().push(task);
    stopper
}
/// returns the running coroutines.
pub fn coroutines() -> usize {
    ACTIVE_COROUTINE_COUNT.get()
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
                increase_yield_counter();
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
        impl<O> Future for Get<'_, O> {
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
        impl<T> Future for Insert<'_, T> {
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

        impl<T> Drop for Insert<'_, T> {
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
