use sycamore::{prelude::*, reactive::use_context};
use web_sys::Event;

use crate::{
    session::{Session, SessionValue},
    svg,
};

#[component]
pub fn LoginStatus<G: Html>(cx: Scope) -> View<G> {
    let session = use_context::<SessionValue>(cx);

    let name = create_memo(cx, move || {
        let logout = |e: Event| {
            session.set(Session::logout());
            e.prevent_default();
        };

        match &*session.get() {
            Session::None => {
                view! { cx, a(class="text-sky-400 hover:text-sky-200", href="/login", rel="external") { "Login" } }
            }
            Session::LoggedIn(user) => {
                let name = user.name.clone();
                let href = user.name.to_url();

                view! { cx,
                    div(class="flex gap-x-2 items-center") {
                        a(href=href) { (name) }
                        a(on:click=logout,
                          title="Logout",
                          class="cursor-pointer h-4/6 text-sky-400 hover:text-red-600",
                          dangerously_set_inner_html=svg::LOGOUT) {}
                    }
                }
            }
        }
    });

    view! { cx,
        (*name.get())
    }
}
