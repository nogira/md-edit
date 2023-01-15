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
        <>
            <EditablePage />
        </>
    })
}


#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PageNode {
    pub id: usize,
    pub kind: PageNodeType,
    pub contents: PageNodeContents,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PageNodeContents {
    Children(RwSignal<Vec<PageNode>>), Content(RwSignal<HashMap<String, String>>)
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PageNodeType {
    // block-branch
    Quote,
    // block-leaf
    H1, H2, H3,
    // text-branch
    Bold, Italic,
    // text-leaf
    Text,
}

#[component]
pub fn EditablePage(cx: Scope) -> Element {

    // TODO: * load in a Vec<PageNode> from file (same structure but w/o the RwSignal) *

    let nodes: RwSignal<Vec<PageNode>> = create_rw_signal(cx, vec![
        PageNode {
            id: 0, kind: PageNodeType::H1, contents: PageNodeContents::Children(
                create_rw_signal(cx, vec![
                    PageNode {
                        id: 0, kind: PageNodeType::Text, contents: PageNodeContents::Content(
                            create_rw_signal(cx, HashMap::from([("text".to_string(), "some text".to_string())]))
                        )
                    }
                ])
            )
        }
    ]);

    // TODO: ideally these should be loaded in from the file
    let top_line_num: RwSignal<u32> = create_rw_signal(cx, 0);
    // this might not be needed bc can prob calc from top_line_num ??
    // let scroll_position: RwSignal<u32> = create_rw_signal(cx, 0);

    view! {cx,
        <div contenteditable>
            <PageNodes nodes />
        </div>
    }
}

#[component]
pub fn PageNodes(cx: Scope, nodes: RwSignal<Vec<PageNode>>) -> Memo<Vec<Element>> {

    view! {cx,
        <For each=nodes key=|e| e.id>
            {|cx: Scope, e: &PageNode| {
                match e.contents {
                    PageNodeContents::Children(nodes) => {
                        match e.kind {
                            PageNodeType::H1 => {
                                view! {cx,
                                    <H1 nodes />
                                }
                            }
                            _ => {
                                view! {cx,
                                    <div>"fail!!!"</div>
                                }
                            }
                        }
                    }
                    PageNodeContents::Content(content) => {
                        match e.kind {
                            PageNodeType::Text => {
                                view! {cx,
                                    <Text content />
                                }
                            }
                            _ => {
                                view! {cx,
                                    <div>"fail!!!"</div>
                                }
                            }
                        }
                    }
                }
            }}
        </For>
    }
}

#[component]
pub fn H1(cx: Scope, nodes: RwSignal<Vec<PageNode>>) -> Element {
    view! {cx,
        <div type="h1">
            <PageNodes nodes />
        </div>
    }
}

#[component]
pub fn Text(cx: Scope, content: RwSignal<HashMap<String, String>>) -> Element {
    view! {cx,
        <span type="text">
            {content.get().get("text").unwrap()}
        </span>
    }
}
