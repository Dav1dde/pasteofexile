use sycamore::prelude::*;

use crate::utils::if_browser;

#[cfg(not(feature = "ssr"))]
thread_local!(static PROGRESS: State = State::new());

#[cfg(not(feature = "ssr"))]
struct State {
    progress: Signal<i32>,
    worker: std::cell::Cell<Option<gloo_timers::callback::Interval>>,
    finish: std::cell::Cell<Option<gloo_timers::callback::Timeout>>,
    width: Signal<f32>,
}

#[cfg(not(feature = "ssr"))]
impl State {
    fn new() -> Self {
        Self {
            progress: Signal::new(0),
            worker: std::cell::Cell::new(None),
            finish: std::cell::Cell::new(None),
            width: Signal::new(0.0),
        }
    }

    fn start_request(&self) {
        self.progress.set(*self.progress.get() + 1);

        if *self.progress.get() == 1 {
            self.finish.set(None);

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

    fn end_request(&self) {
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

pub struct InFlight;

impl Drop for InFlight {
    fn drop(&mut self) {
        #[cfg(not(feature = "ssr"))]
        PROGRESS.with(|state| state.end_request());
    }
}

#[must_use]
pub fn start_request() -> InFlight {
    #[cfg(not(feature = "ssr"))]
    PROGRESS.with(|state| state.start_request());
    InFlight
}

#[component(Progress<G>)]
pub fn progress() -> View<G> {
    let style = if_browser!(
        {
            let width = PROGRESS.with(|state| state.width.clone());
            crate::memo!(width, {
                if *width.get() == 0.0 {
                    format!("width: {}%; transition: none", width.get())
                } else {
                    format!("width: {}%", width.get())
                }
            })
        },
        Signal::new("")
    );

    view! {
        div(class="fixed transition-[width] duration-300 ease-linear top-0 left-0 h-0.5 z-50 bg-sky-400",
            style=style.get()) {
        }
    }
}
