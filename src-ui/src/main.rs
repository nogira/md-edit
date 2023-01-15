#![feature(box_syntax)]
#![feature(iter_advance_by)]

use std::collections::HashMap;

use leptos::{*, js_sys::Function};
use src_ui::*;

mod md_page; use md_page::*;
mod text_conversion; use text_conversion::*;

pub fn main() {
    _ = console_log::init_with_level(log::Level::Debug);
    console_error_panic_hook::set_once();
    mount_to_body(|cx| view! { cx, 
        <div>
            <MarkdownPage name="Michal".to_string() />
            // <PageBlock text="weeeeee".to_string() idx=0 />
            // <li>"hi"</li>
            // <li>"hi"<ul><li>"hi"</li></ul></li>
            // <SimpleCounter name="Michal".to_string() />
        </div>
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SubBlock {
    id: usize,
    text: String,
}

// struct PageNode {
//     kind: PageNodeType,
//     contents: PageNodeContents
// }
// enum PageNodeContents {
//     Children(Vec<PageNode>), Content(HashMap<String, String>)
// }
// enum PageNodeType {
//     Quote, H1, H2, H3
// }

// TODO: change PageBlock to EditablePage prototype

#[component]
pub fn PageBlock(cx: Scope, text: String, idx: usize) -> Element {

    let (
        top_line_num,
        set_top_line_num
    ) = create_signal::<u32>(cx, 0);

    // parse block_text to block
    let mut sub_blocks: Vec<SubBlock> = Vec::new();
    if idx == 0 {
        sub_blocks.push(SubBlock { id: 1, text: "one".to_string() });
        sub_blocks.push(SubBlock { id: 2, text: "two".to_string() });
        sub_blocks.push(SubBlock { id: 3, text: "thr".to_string() });
    }

    console_log(&format!("{:?}", cx.all_resources()));

    view! {cx,
        <div block=0 on:click=|_| console_log("click")>
            {text}
            <For each=move || {sub_blocks.clone()} key=|e| e.id>
                {|cx: Scope, e: &SubBlock| {
                    view! {cx,
                        <PageBlock text=e.text.clone() idx=e.id />
                    }
                }}
            </For>
        </div>
    }
}
