#![feature(box_syntax)]
#![feature(iter_advance_by)]
#![feature(const_for)]

use leptos::*;
// use src_ui::*;

mod editable_page; use editable_page::*;
mod render_in_view; use render_in_view::*;
mod page_data; use page_data::*;

pub fn main() {
    _ = console_log::init_with_level(log::Level::Debug);
    console_error_panic_hook::set_once();
    mount_to_body(|cx| view! { cx, 
        <div style="position: fixed; height: 100vh; width: 100vw">
            <EditablePage />
        </div>
    })
}
