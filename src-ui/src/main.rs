#![feature(box_syntax)]
#![feature(iter_advance_by)]

use leptos::*;
use src_ui::*;

mod editable_page; use editable_page::*;

pub fn main() {
    _ = console_log::init_with_level(log::Level::Debug);
    console_error_panic_hook::set_once();
    mount_to_body(|cx| view! { cx, 
        <>
            <EditablePage />
        </>
    })
}
