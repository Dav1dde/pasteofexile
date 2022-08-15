use crate::{
    memo,
    session::{Session, SessionValue},
    svg,
};
use sycamore::{context::use_context, prelude::*};
use web_sys::Event;

#[component(LoginStatus<G>)]
pub fn login_status() -> View<G> {
    let session = use_context::<SessionValue>();

    let name = memo!(session, {
        let logout = cloned!(session => move |e: Event| {
            session.logout();
            e.prevent_default();
        });

        match &*session.get() {
            Session::None => {
                // view! { a(class="text-sky-500 dark:text-sky-400", href="/login") { "Login" } }
                view! { div() {} }
            }
            Session::LoggedIn(user) => {
                let name = user.name.clone();
                let href = format!("/u/{name}");

                // component not wrapped in router, need to manually navigate
                let navigate_user = cloned!(href => move |ev: web_sys::Event| {
                    sycamore_router::navigate(&href);
                    ev.prevent_default();
                });

                view! {
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

    view! {
        (*name.get())
    }
}
