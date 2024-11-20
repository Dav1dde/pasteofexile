use sycamore::prelude::*;

#[cfg(feature = "browser")]
mod browser {
    use sycamore::prelude::*;

    thread_local!(pub static PROGRESS: State = State::new());

    pub struct State {
        pub progress: RcSignal<i32>,
        pub worker: std::cell::Cell<Option<gloo_timers::callback::Interval>>,
        pub finish: std::cell::Cell<Option<gloo_timers::callback::Timeout>>,
        pub width: RcSignal<f32>,
    }

    impl State {
        pub fn new() -> Self {
            Self {
                progress: create_rc_signal(0),
                worker: std::cell::Cell::new(None),
                finish: std::cell::Cell::new(None),
                width: create_rc_signal(0.0),
            }
        }

        pub fn start_request(&self) {
            self.progress.set(*self.progress.get() + 1);

            if *self.progress.get() == 1 {
                // self.finish.set(None);

                let width = self.width.clone();
                width.set(10.0);

                self.worker
                    .set(Some(gloo_timers::callback::Interval::new(350, move || {
                        let step = match *width.get() {
                            n if n >= 95.5 => 0.0,
                            n if n >= 80.0 => 0.5,
                            n if n >= 50.0 => 2.0,
                            n if n >= 20.0 => 4.0,
                            _ => 10.0,
                        };
                        width.set(*width.get() + step);
                    })));
            }
        }

        pub fn end_request(&self) {
            self.progress.set(*self.progress.get() - 1);

            if *self.progress.get() == 0 {
                self.worker.set(None);

                let width = self.width.clone();
                width.set(100.0);

                self.finish
                    .set(Some(gloo_timers::callback::Timeout::new(350, move || {
                        width.set(0.0);
                    })));
            }
        }
    }
}

pub struct InFlight;

impl Drop for InFlight {
    fn drop(&mut self) {
        #[cfg(feature = "browser")]
        browser::PROGRESS.with(|state| state.end_request());
    }
}

#[must_use]
pub fn start_request() -> InFlight {
    #[cfg(feature = "browser")]
    browser::PROGRESS.with(|state| state.start_request());
    InFlight
}

fn width() -> RcSignal<f32> {
    #[cfg(feature = "browser")]
    {
        browser::PROGRESS.with(|state| state.width.clone())
    }
    #[cfg(not(feature = "browser"))]
    {
        create_rc_signal(0.0)
    }
}

#[component]
pub fn Progress<G: Html>(cx: Scope) -> View<G> {
    let width = width();
    let style = create_memo(cx, move || {
        if *width.get() == 0.0 {
            format!("width: {}%; transition: none", width.get())
        } else {
            format!("width: {}%", width.get())
        }
    });

    view! { cx,
        div(class="fixed transition-[width] duration-300 ease-linear top-0 left-0 h-0.5 z-50 bg-sky-400",
            style=style.get()) {
        }
    }
}
