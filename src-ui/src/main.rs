#![feature(box_syntax)]
#![feature(iter_advance_by)]

use leptos::*;
use src_ui::*;

mod editable_page; use editable_page::*;
mod render_in_view; use render_in_view::*;
mod page_data; use page_data::*;

pub fn main() {
    _ = console_log::init_with_level(log::Level::Debug);
    console_error_panic_hook::set_once();
    mount_to_body(|cx| view! { cx, 
        <div style="position: fixed">
            <EditablePage />
            // <Main />
        </div>
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Elem {
    id: usize,
    text: String,
}

#[component]
pub fn Main(cx: Scope) -> impl IntoView {

    // parse block_text to block
    let items: RwSignal<Vec<RwSignal<Elem>>> = create_rw_signal(cx, vec![
        create_rw_signal(cx, Elem { id:0, text: "zero".into() }),
        create_rw_signal(cx, Elem { id:1, text: "one".into() }),
        create_rw_signal(cx, Elem { id:2, text: "two".into() }),
        create_rw_signal(cx, Elem { id:3, text: "three".into() }),
    ]);
    let current_items: RwSignal<Vec<RwSignal<Elem>>> =  create_rw_signal(cx, Vec::new());

    let start_idx: RwSignal<usize> = create_rw_signal(cx, 0);
    let update_displayed = move |idx| {
        let v = items.get();
        let mut in_view = Vec::new();
        for e in v {
            let elem = e.get();
            if elem.id >= idx {
                in_view.push(e);
            };
        }
        current_items.update(|v| *v = in_view);
    };
    let dec = move |_| {
        let current = start_idx.get();
        if current != 0 {
            start_idx.update(|i| { *i -= 1});
            update_displayed(current - 1);
        }
    };
    let inc = move |_| {
        let current = start_idx.get();
        if current != usize::MAX {
            start_idx.update(|i| { *i += 1});
            update_displayed(current + 1);
            console_log(&format!("{:?}", items.get()));
        }
    };

    update_displayed(0);

    view! {cx,
        <div>
            " start idx: "<span>{move || start_idx.get()}</span>
            <button on:click=dec>"-"</button>
            <button on:click=inc>"+"</button>
        </div>
        <br />
        <For each=current_items key=move|e| e.get().id
        view=move |e| {
            console_log(&format!("1. RERENDERING: {}", e.get().id)); // THIS TRIGGERS ON RE-RENDER
            view! {cx,
                {console_log(&format!("2. RERENDERING: {}", e.get().id));} // THIS TRIGGERS ON RE-RENDER
                <div id=e.get().id>
                    {e.get().text.clone()}
                </div>
            }
        } />
    }
}
