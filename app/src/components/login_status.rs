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
            Session::logout();
            e.prevent_default();
        };

        match &*session.get() {
            Session::None => {
                view! { cx, a(class="text-sky-500 dark:text-sky-400", href="/login") { "Login" } }
            }
            Session::LoggedIn(user) => {
                let name = user.name.clone();
                let href = format!("/u/{name}");

                // component not wrapped in router, need to manually navigate
                let href2 = href.clone(); // TODO: is there a better way now? create_ref?
                let navigate_user = move |ev: web_sys::Event| {
                    sycamore_router::navigate(&href2);
                    ev.prevent_default();
                };

                view! { cx,
                    div(class="flex gap-x-2 items-center") {
                        a(on:click=navigate_user, href=href) { (name) }
                        a(on:click=logout,
                          title="Logout",
                          class="cursor-pointer h-4/6 text-sky-500 dark:text-sky-400",
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
