use std::{collections::{HashMap, hash_map::DefaultHasher}, hash::{Hash, Hasher, self}};
use core::{cmp::max, fmt::Debug};
use leptos::{*, js_sys::Math};
use serde::Serialize;
use tauri_sys::{event, tauri};
// use web_sys::Element;
// use src_ui::*;

// FIXME: CURRENTLY BOTTLENECKED BY NOT BEING ABLE TO GET THE SIZE OF AN 
// ELEMENT ==AFTER== IT HAS BEEN RENDERED TO THE DOM

/// pass in the element that has been updated, and update all elements IN_VIEW 
/// below. we don't need to update ones out of view bc they will update when 
/// we scroll down to them
fn update_dims(start_elem: Vec<usize>) {

}

// seems EXTREMELY complex/janky to get the top/bottom/height of each element 
// bc it must be rendered to the DOM to have those attributes, and there is no 
// easy way to trigger a callback as soon as the element loads

// way to avoid this is prob to simply keep track of the height of the top 
// elem, as knowing the metrics of the other elems is pretty irrelevent
// 
// when top elem moves out of view, add top="x" attribute to elem below, then 
// disappear the out-of-view elem. in addition, it seems like this would be 
// much harder w/ nested elems bc you would have to add top="x" to all of them, 
// and seems v complex to add and remove children while scrolling up/down
// SOLUTION: ONLY DO IN-VIEW RENDERING FOR THE BASE ELEMENTS, NOT THE CHILDREN.
// this is quite feasible bc long blocks are not extremely common
//
// actually on second thought it might not be that hard to update them. once 
// you have to top child all you need to do is go though the `location` list 
// to change all the `top` attrs
// 
// SOMETHING ELSE IS I PREF WANT

// ok so i need to store the hash + path of all elements. this is to enable 
// both selections, edits, and in-view rendering/rerendering

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Page {
    pub nodes: RwSignal<Vec<PageNode>>,
    /// (hash, location, top)
    /// also use this to calculate scroll position
    pub top_elem: RwSignal<(String, Vec<usize>, usize)>,
    pub locations: RwSignal<HashMap<String, Vec<usize>>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PageNode {
    pub hash: String,
    pub kind: PageNodeType,
    pub contents: PageNodeContents,
    // /// the y-axis top of the element in pixels
    // pub top: usize,
    // /// the y-axis bottom of the element in pixels
    // pub bottom: usize,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PageNodeContents {
    Children(RwSignal<Vec<PageNode>>), Content(RwSignal<HashMap<String, String>>)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PageNodeType {
    // root
    // Page, // using Page as root so able to attach a method to index node (as 
          // otherwise you would have to first index the root RwSignal<Vec<PageNode>> 
          // before runniing PageNode index method)
    // block-branch
    Quote,
    // block-leaf
    TextBlock, H1, H2, H3, CodeBlock,
    // text-branch
    Bold, Italic, Highlight, CodeInline, FileLink, UrlLink,
    // text-leaf
    RawText,
}
impl PageNodeType {
    fn value(&self) -> &str {
        match *self {
            PageNodeType::Quote => "q",
            PageNodeType::TextBlock => "tb",
            PageNodeType::H1 => "h1",
            PageNodeType::H2 => "h2",
            PageNodeType::H3 => "h3",
            PageNodeType::CodeBlock => "cb",
            PageNodeType::Bold => "b",
            PageNodeType::Italic => "i",
            PageNodeType::Highlight => "h",
            PageNodeType::CodeInline => "ci",
            PageNodeType::FileLink => "fl",
            PageNodeType::UrlLink => "ul",
            PageNodeType::RawText => "t",
        }
    }
}

// #[derive(Serialize)]
// struct RetreiveFileCmdArgs {
//     hash: String,
// }
// async fn retrieve_file(hash: String) -> String {
//     tauri::invoke("retrieve_file", &RetreiveFileCmdArgs { hash })
//         .await
//         .unwrap()
// }

// pub enum Key {
//     Return, Delete, ForwardSlash
// }
// impl Key {
//     fn key_code(&self) -> u32 {
//         match self {
//             Key::Return => 13,
//             Key::Delete => 8,
//             Key::ForwardSlash => 191,
//         }
//     }
// }

#[component]
pub fn EditablePage(cx: Scope) -> impl IntoView {

    // TODO: * load in a Vec<PageNode> from file (same structure but w/o the RwSignal) *

    let page_data: RwSignal<Page> = create_rw_signal(cx,
        Page { nodes: create_rw_signal(cx, vec![
            PageNode {
                hash: "".into(), kind: PageNodeType::H1, contents: PageNodeContents::Children(
                    create_rw_signal(cx,vec![
                        PageNode {
                            hash: "".into(), kind: PageNodeType::RawText, contents: PageNodeContents::Content(
                                create_rw_signal(cx,HashMap::from([("text".to_string(), "some text".to_string())]))
                            )
                        }
                    ])
                )
            },
            PageNode {
                hash: "".into(), kind: PageNodeType::TextBlock, contents: PageNodeContents::Children(
                    create_rw_signal(cx,vec![
                        PageNode {
                            hash: "".into(), kind: PageNodeType::RawText, contents: PageNodeContents::Content(
                                create_rw_signal(cx,HashMap::from([("text".to_string(), "some text".to_string())]))
                            )
                        }
                    ])
                )
            }
        ]), top_elem: create_rw_signal(cx, ("".into(), Vec::new(), 0))
        , locations: create_rw_signal(cx, HashMap::new())}
    );

    // ok so lets start by doing rendering conditional upon being within view, 
    // then appending hash to hashset if item is rendered
    // hashet could either contain the path to the item, or a closure that contains an item.update(|i| *i = x)

    // if i'm able to attach a message event to each node, i could send a 
    // message by posting to the element looked up by hash (i.e. querySelector())

    // let handle_keypress = move |event: web_sys::KeyboardEvent| {

    //     // TODO: ALL I NEED TO HANDLE IS DELETE RETURN AND /
    //     // EVERYTHING ELSE IS ALREADY FINE

    //     // FIXME: WAIT BUT DATA NEEDS TO UPDATE AS WELL AS THE DOM SO ALL MUST BE UPDATED THROUGH THIS FUNCTION

    //     let selection = document().get_selection().unwrap().unwrap();
    //     let key_code = event.key_code();
    //     console_log(&format!("KEY: {:?}", key_code));
    //     // first check selection type bc if no selection, and not a special 
    //     // char, we don't need to do anything. if there is a selection though 
    //     // be need to handle the deletion
    //     let sel_type = selection.type_(); // "Range" or "Caret" (caret is 0 range)

    //     let keys = [
    //         Key::Return.key_code(),
    //         Key::Delete.key_code(),
    //         Key::ForwardSlash.key_code(),
    //     ];
    //     if &sel_type == "Range" || keys.contains(&key_code) {
    //         let nodes = page_data.get();

    //         // FIXME: shit maybe i need hashing after-all w/ the hashes also 
    //         // stored in a signal along with their paths. looks like it will 
    //         // be very expensive to find the position of each child w/o it (if even possible)
    //         let start_node = selection.anchor_node().unwrap();
    //         let start_offset = selection.anchor_offset();

    //         if &sel_type == "Range" {

    //         } else {

    //         }



    //         // RETURN key pressed
    //         if event.key_code() == 13 {
    //             event.prevent_default();
    //             let parent = selection.anchor_node().unwrap().parent_element().unwrap();
    //             console_log(&format!("...: {:?}", parent));
    //             // console_log(&format!("{:?}", selection.anchor_node().unwrap().parent_element().unwrap().get_attribute("block").unwrap()));

    //         // DELETE key pressed
    //         } else if event.key_code() == 8 {
    //             // FIXME: only need to prevent default if the delete if the first 
    //             // char in a block
    //             event.prevent_default();

    //             // TODO: i wonder if its better to immediately find the offset in 
    //             // string, and current node, then edit the data for re-render 
    //             // rather than any parsing of the DOM itself

    //             // can't store location of each node in the element itself bc e.g. 
    //             // when a node is removed you would have to update all elements in 
    //             // the vec with their new indexes which defeats the purpose of 
    //             // reactivity

    //             let selection = document().get_selection().unwrap().unwrap();
    //             let sel_type = selection.type_(); // "Range" or "Caret" (caret is 0 range)
                
    //             // let node locatio

    //             let offset = selection.anchor_offset();
    //             let selected = selection.anchor_node().unwrap();
    //             let len = selected.to_string().length();
    //             console_log(&format!("SEL TYPE: {:?}", len));
    //             let parent = selection.anchor_node().unwrap().parent_element().unwrap();
    //             let kind = parent.get_attribute("node").unwrap();
    //             console_log(&format!("TYPE: {:?}", kind));
    //             // step 1, check if block or span, bc need to get to parent block
    //             // if selection is at start of block, 
    //             match kind.as_str() {
    //                 // span
    //                 "span" => {},
    //                 // block
    //                 _ => {},
    //             }
    //             // if parent.tag_name() == "MD" {
    //             //     let md_elem = parent.parent_element().unwrap();
    //             // }
    //             // let full_page = 
    //             console_log(&format!("KEY: {:?}", event.key_code()));
    //         }
    //     }
    // };

    rand_alphanumerecimal_hash();
    // console_log(rand_alphanumerecimal_hash().as_str());

    // console_log(&format!("HIEGHT: {:?}", page.get_box_quads()));

    // on_mounted

    // create_effect(cx, move |_| {
    //     console_log("hello");
    //     if let Some(elem) = document().query_selector("[type=page]").unwrap() {
    //         console_log(&format!("{:?}", elem.get_box_quads()));
    //     };
    // });
    // create_effect

    view! {cx,
        <div contenteditable
        type="page"
        on:scroll=|_| console_log("scroll")
        // on:keydown=handle_keypress
        >
            <PageNodes
            nodes=page_data.get().nodes
            location=Vec::new()
            hashes=page_data.get().locations />
        </div>
    }
}

// TODO: if compare doesn't work to find the number of the child, use the hash after-all, purely for unique child-nodes. ohhhh wait this wouldn't work bc i'm not rendering all the nodes to the page. MAYBE BETTER TO JUST SAY FUCK REACTVITY AND RE-RENDER ALL VISIBLE (AT LEAST ALL THAT NEED TO UPDATE INDEXES) WHEN UPDATE OCCURS

/// generate an alphanumeric hash string of length 5
fn rand_alphanumerecimal_hash() -> String {
    // chars used: 26 a-z, 10 0-9 -> 36
    // 36^4 = 1.67 million (1.67 million perhaps too small ?)
    // 36^5 = 60.4 million <--
    // 
    // generated random number: u64 = 2^64 = 18.4 quintillion
    // 2^32 = 4.29 billion
    // 2^16 = 65k
    // 2^26 = 67.1 million <--
    let gen_rand_num = || {
        let mut hasher = DefaultHasher::new();
        Math::random().to_bits().hash(&mut hasher);
        let bits_32 = hasher.finish() as u32;
        // 32 - 26 = 6
        let clipper = u32::MAX >> 6;
        let clipped = bits_32 & clipper;
        clipped
    };
    const MAX: u32 = 36_u32.pow(5) - 1;
    const BASE: u32 = 36_u32;
    loop {
        let mut hash_str = String::new();
        let mut carry = gen_rand_num();
        if carry <= MAX {
            loop {
                let rem = carry % BASE;
                if rem < 10 { hash_str.push_str(&format!("{}", rem)) }
                // 'a' == 97 as char
                else { hash_str.push(char::from_u32(rem + 87).unwrap()) }

                if carry == rem {
                    return hash_str
                } else {
                    carry = (carry - rem) / BASE;
                }
            }
        }
    }
}

// TODO: make the key/id a microhash string (like 4 char long?) and store the 
// paths to each microhash, so e.g. if click an elem, the click registers and 
// microhash is used to edit nodes
// 36^4 = ! 1.7 million
// even assuming each word has a diff style, you would never reach a page w/ 
// 1.7 million words, so seems fine as long as check for collisions during 
// each generation
// TODO: check the num nodes in my max-length doc. potentially increase length 
// to 5. note that at lets say 500k words you have a reasonable chance of 
// sequential collisions which is undesirable


// const text = 'An obscure body in the S-K System, your majesty. The inhabitants refer to it as the planet Earth.';

// async function digestMessage(message) {
//   const msgUint8 = new TextEncoder().encode(message);                           // encode as (utf-8) Uint8Array
//   const hashBuffer = await crypto.subtle.digest('SHA-256', msgUint8);           // hash the message
//   const hashArray = Array.from(new Uint8Array(hashBuffer));                     // convert buffer to byte array
//   const hashHex = hashArray.map((b) => b.toString(16).padStart(2, '0')).join(''); // convert bytes to hex string
//   return hashHex;
// }

// digestMessage(text)
//   .then((digestHex) => console.log(digestHex));

#[component]
pub fn PageNodes(cx: Scope,
    nodes: RwSignal<Vec<PageNode>>,
    location: Vec<usize>,
    hashes: RwSignal<HashMap<String, Vec<usize>>>
) -> impl IntoView {
    let mut elems: Vec<HtmlElement<AnyElement>> = Vec::new();

    for (i, node) in nodes.get().iter().enumerate() {
        let mut location = location.clone();
        location.push(i);
        let elem = match node.contents {
            PageNodeContents::Children(nodes) => {
                match node.kind {
                    PageNodeType::H1 => view! {cx,
                        <div contenteditable
                        type=PageNodeType::H1.value()>
                            <PageNodes nodes location hashes />
                        </div>
                    },
                    PageNodeType::TextBlock => view! {cx,
                        <div type=PageNodeType::TextBlock.value()>
                            <PageNodes nodes location hashes />
                        </div>
                    },
                    _ => view! {cx,
                        <div>"‼️ block missing"</div>
                    },
                }.into_any()
            }
            PageNodeContents::Content(content) => {
                match node.kind {
                    PageNodeType::RawText => view! {cx,
                        <span type=PageNodeType::RawText.value()>
                            {content.get().get("text").unwrap()}
                        </span>
                    },
                    _ => view! {cx,
                        <span type=PageNodeType::RawText.value()>
                            "‼️ only raw text allowed"
                        </span>
                    },
                }.into_any()
            }
        };
        elems.push(elem);
    }
    elems
}

// #[component]
// pub fn InlineStyle(cx: Scope, nodes: RwSignal<Vec<PageNode>>, kind: &str) -> Element {
//     // FIXME: this is giving an error. when i remove `kind: &str` and `type=kind` it stops
//     view! {cx,
//         <span type=kind>
//             <PageNodes nodes />
//         </span>
//     }
// }

// not sure these need to be specialized. do above instead??

// #[component]
// pub fn UrlLink(cx: Scope, nodes: RwSignal<Vec<PageNode>>) -> Element {
//     view! {cx,
//         <span
//         type=PageNodeType::UrlLink.value()>
//             <PageNodes nodes />
//         </span>
//     }
// }

// #[component]
// pub fn FileLink(cx: Scope, nodes: RwSignal<Vec<PageNode>>) -> Element {
//     view! {cx,
//         <span
//         type=PageNodeType::FileLink.value()>
//             <PageNodes nodes />
//         </span>
//     }
// }

// #[component]
// pub fn Highlight(cx: Scope, nodes: RwSignal<Vec<PageNode>>) -> Element {
//     view! {cx,
//         <span
//         type=PageNodeType::Highlight.value()>
//             <PageNodes nodes />
//         </span>
//     }
// }

// #[component]
// pub fn Italic(cx: Scope, nodes: RwSignal<Vec<PageNode>>) -> Element {
//     view! {cx,
//         <span
//         type=PageNodeType::Italic.value()>
//             <PageNodes nodes />
//         </span>
//     }
// }

// #[component]
// pub fn Bold(cx: Scope, nodes: RwSignal<Vec<PageNode>>) -> Element {
//     view! {cx,
//         <span
//         type=PageNodeType::Bold.value()>
//             <PageNodes nodes />
//         </span>
//     }
// }

// #[derive(Debug, Clone, PartialEq)]
// enum MDNodeType {
//     Block, Span, Text
// }
// #[derive(Debug, Clone, PartialEq)]
// enum HideableMDType {
//     None, H1, Bold, Italic
// }
// type MDNodeInfo = (MDNodeType, usize, HideableMDType);

// #[component]
// pub fn MarkdownPage(cx: Scope, name: String) -> Element {

//     let (
//         top_line_num,
//         set_top_line_num
//     ) = create_signal::<u32>(cx, 0);

//     // e.g. vec![(Block, 2, None), (Block, 1, H1), (Span, 4, Bold), (Text, 3, None)]
//     // meaning 3rd block, 2nd sub-block, 5th span elem, position 3 in text node
//     let (
//         selection_start,
//         set_selection_start
//     ) = create_signal::<Vec<MDNodeInfo>>(cx, Vec::new());

//     create_effect(cx, |_| {console_log("effect")});

//     console_log("RELOAD");

//     let text = "#\n# hello     .<!-- hi -->\n- some point\n> this **is** cool\n> 🤡 ᾎ ᾯ y̆".to_string();
//     let mut blocks: Vec<Element> = Vec::new();

//     // text intermediary between raw text and html
//     let (
//         text_imd,
//         set_text_imd
//     ) = create_signal::<Vec<MDBlock>>(cx, Vec::new());

//     // TODO: SWAP FROM BLOCK TYPE IN CLASS TO BLOCK TYPE IN type=""

//     create_effect(cx, move |_| {
//         console_log("setting text_imd");
//         let imd = text_to_imd_blocks(&text);
//         console_log(&format!("{:?}", imd));
//         set_text_imd.update(|v| *v = imd);
//     });

//     // let div_s = create_element("span");
//     // div_s.set_inner_html("**");
//     // div_s.set_attribute("hidden", "true").unwrap();
//     // // div_s.add_event_listener_with_callback(type_, listener);

//     // let div_e = view! {cx, <div line="3">"more "<b>"hello"{div_s}</b>" text"</div>};
//     // // div_e.set_inner_html("<b><span>**</span>xd<span>**</span></b>");

//     fn handle_keypress(event: web_sys::KeyboardEvent) {
//         console_log(&format!("KEY: {:?}", event.key_code()));

//         // if return key pressed. 
//         if event.key_code() == 13 {
//             console_log("return");
//             event.prevent_default();
//             let selection = document().get_selection().unwrap().unwrap();
//             // console_log(&format!("{:?}", selection.anchor_node().unwrap().parent_element().unwrap().get_attribute("block").unwrap()));
//         // if delete key pressed
//         } else if event.key_code() == 8 {
//             // let selection = document().get_selection().unwrap().unwrap();
//             // let parent = selection.anchor_node().unwrap().parent_element().unwrap();
//             // if parent.tag_name() == "MD" {
//             //     let md_elem = parent.parent_element().unwrap();
//             // }
//             // let full_page = 
//             console_log(&format!("KEY: {:?}", event.key_code()));
//         }
//     }

//     let out = view! {cx,
//         // scroll view
//         <div id="md-page" contenteditable on:scroll=|_| console_log("test") on:keydown=handle_keypress>
//             {imd_blocks_to_dom(&text_imd.get())}
//         </div>
//     };
//     use crate::wasm_bindgen::closure::Closure;


//     // let closure = Closure::<dyn FnMut(_)>::new(move |event: web_sys::Event| {
//     //     event.prevent_default()

//     //     // find the line clicked on, then run ".innerText" on that div so can update my vec on line strings
//     // });
//     // document().add_event_listener_with_callback("", closure.as_ref().unchecked_ref()).unwrap();
//     // closure.forget();

//     let closure = Closure::<dyn FnMut(_)>::new(move |event: web_sys::Event| {
//         // console_log("PRESELECTION");
//         let selection = document().get_selection().unwrap().unwrap();
//         // console_log(&format!("selection: {:?}", selection.to_string()));
//         // console_log("POSTSELECTION");
//         // console_log(&format!("{:?}", selection.anchor_node()));
//         // console_log(&format!("{:?}", selection.anchor_offset()));
//         // console_log(&format!("{:?}", selection.to_string()));

//         let mut new_selection_start = Vec::new();

//         // if there is no selection, this is where we catch it (rather than 
//         // `let selection =`). we still want to run it to deselect prev
//         if let Some(anchor) = selection.anchor_node() {

//             new_selection_start.push((
//                 MDNodeType::Text,
//                 selection.anchor_offset() as usize,
//                 HideableMDType::None));

//             // PARENT ELEM
//             let mut parent_elem = anchor.parent_element().unwrap();
//             // get hideable md type from class (if not hideable, val is just None)
//             let hideable_md_type = get_hideable_md_type(&parent_elem);
//             // get node type & idx
//             let (node_type, idx) = get_span_or_block_info(&parent_elem);
//             new_selection_start.insert(0, (node_type, idx, hideable_md_type.clone()));

//             // get info of all parent elems
//             loop {
//                 parent_elem = parent_elem.parent_element().unwrap();
//                 // end if we reach the root node
//                 if &parent_elem.id() == "md-page" { break }

//                 // get hideable md type from class (if not hideable, val is just None)
//                 let hideable_md_type = get_hideable_md_type(&parent_elem);
//                 // get node type & idx
//                 let (node_type, idx) = get_span_or_block_info(&parent_elem);
//                 new_selection_start.insert(0, (node_type, idx, hideable_md_type.clone()));
//             }
//         };

//         // loop through new_selection_start and compare to selection_start to 
//         // know if need to unhide or hide elems

//         let prev_vec = selection_start.get();
//         let prev_len = prev_vec.len();
//         let curr_len = new_selection_start.len();
//         let max_len = max(prev_len, curr_len);

//         let root = document().query_selector("#md-page").unwrap().unwrap();
//         let mut prev_elem = root.clone();
//         let mut curr_elem = root;

//         // FIXME: WAIT I DON'T REALLY NEED TO GO BACK ALL THE WAY BC THE ONLY 
//         // MARKDOWN THAT WILL BECOME INHIDDEN IS REALLY  IN THE TEXT/SPANS, OR 
//         // H1 HEADING

//         for i in 0..max_len {
//             // NEED TO TRACK WHEN THEY DIVERGE bc e.g. if the first elem is 
//             // diff, but next is same, its absolutely not the same element, 
//             // but only way to know that is to know if all previous 
//             // `MDNodeInfo`s are the same or not
//             let diff = false;
//             // if prev item present
//             if let Some(prev_item) = prev_vec.get(i) {
//                 if prev_item.0 == MDNodeType::Text { continue }
//                 // get nth child according to prev_item
//                 prev_elem = prev_elem.first_element_child().unwrap();
//                 for _ in 0..prev_item.1 {
//                     prev_elem = prev_elem.next_element_sibling().unwrap();
//                 }
//                 if let Some(curr_item) = new_selection_start.get(i) {
//                     if curr_item.0 == MDNodeType::Text { continue }
//                     // get nth child according to curr_item
//                     curr_elem = curr_elem.first_element_child().unwrap();
//                     for _ in 0..curr_item.1 {
//                         curr_elem = curr_elem.next_element_sibling().unwrap();
//                     }
//                     // need to update both if the item is different (either by 
//                     // the item value itself, or by previous divergence)
//                     if diff || prev_item != curr_item {
//                         // check if old needs md hidden
//                         hide_md(prev_item, &prev_elem);
//                         // check if new needs md unhidden
//                         unhide_md(curr_item, &curr_elem);
//                     } // if both the same, don't need to do anything
//                 // if no current item, safe to hide md
//                 } else {
//                     // check if old needs md hidden
//                     hide_md(prev_item, &prev_elem);
//                 }
//             // if no prev item, safe to unhide
//             } else if let Some(curr_item) = new_selection_start.get(i) {
//                 if curr_item.0 == MDNodeType::Text { continue }
//                 // get nth child according to curr_item
//                 curr_elem = curr_elem.first_element_child().unwrap();
//                 for _ in 0..curr_item.1 {
//                     curr_elem = curr_elem.next_element_sibling().unwrap();
//                 }
//                 // check if new needs md unhidden
//                 unhide_md(curr_item, &curr_elem);
//             }
//         }

//         set_selection_start.update(|v| *v = new_selection_start)

//         // TODO: find the line clicked on, then run ".innerText" on that div so can update my vec on line strings

//     });
//     document().add_event_listener_with_callback("selectionchange", closure.as_ref().unchecked_ref()).unwrap();
//     closure.forget();
//     out
// }

// fn imd_blocks_to_dom(imd: &Vec<MDBlock>) -> Vec<Element> {
//     // block num is so i can access `selection_start` vec from just the DOM info
//     let mut elem_num = 0;
//     let mut elem_blocks: Vec<Element> = Vec::new();

//     // NOTE: THE REASON I'M USING `block` and `text` ATTRIBUTE NAMES 
//     // INSTEAD OF E.G. `elem-num` IS SO I CAN DETECT IF BLOCK OR SPAN AT 
//     // THE SAME TIME I GET IT'S NUMBER

//     for block in imd {
//         match block {
//             MDBlock::Leaf(leaf) => {
//                 match &leaf.kind {
//                     MDBlockType::H1 => {
//                         let elem = create_element("div");
//                         elem.set_attribute("type", "h1").unwrap();
//                         elem.set_attribute("block", &format!("{}", elem_num)).unwrap();
//                         let md = create_element("md");
//                         md.set_inner_html("#&nbsp;");
//                         md.set_attribute("hidden", "").unwrap();
//                         elem.append_child(&md).unwrap();
//                         let text = imd_text_to_dom(&leaf.text);
//                         for t in text {
//                             elem.append_child(&t).unwrap();
//                         }
//                         elem_blocks.push(elem);
//                         elem_num += 1;
//                     },
//                     MDBlockType::Text => {
//                         let elem = create_element("div");
//                         elem.set_attribute("type", "text").unwrap();
//                         elem.set_attribute("block", &format!("{}", elem_num)).unwrap();
//                         let text = imd_text_to_dom(&leaf.text);
//                         for t in text {
//                             elem.append_child(&t).unwrap();
//                         }
//                         elem_blocks.push(elem);
//                         elem_num += 1;
//                     },
//                     _ => {},
//                 }
//             },
//             MDBlock::Branch(branch) => {
//                 match &branch.kind {
//                     MDBlockType::Quote => {
//                         let elem = create_element("div");
//                         elem.set_attribute("type", "quote").unwrap();
//                         elem.set_attribute("block", &format!("{}", elem_num)).unwrap();
//                         let children = imd_blocks_to_dom(&branch.children);
//                         for child in &children {
//                             elem.append_child(&child).unwrap();
//                         }
//                         elem_blocks.push(elem);
//                         elem_num += 1;
//                     }
//                     _ => {},
//                 }
//             }
//         }
//     }
//     elem_blocks
// }

// fn imd_text_to_dom(imd: &Vec<MDText>) -> Vec<Element> {
//     // block num is so i can access `selection_start` vec from just the DOM info
//     let mut elem_num = 0;
//     let mut elem_spans: Vec<Element> = Vec::new();

//     for span in imd {
//         match span {
//             MDText::Leaf(leaf) => {
//                 match &leaf.kind {
//                     MDTextType::Raw => {
//                         let elem = create_element("span");
//                         elem.set_attribute("type", "raw").unwrap();
//                         elem.set_attribute("text", &format!("{}", elem_num)).unwrap();
//                         elem.set_inner_html(&spaces_to_nbsp(&leaf.text));
//                         elem_spans.push(elem);
//                         elem_num += 1;
//                     },
//                     _ => {},
//                 }
//             },
//             MDText::Branch(branch) => {
//                 match &branch.kind {
//                     MDTextType::Bold => {
//                         let elem = create_element("span");
//                         elem.set_attribute("type", "b").unwrap();
//                         elem.set_attribute("text", &format!("{}", elem_num)).unwrap();
//                         let md = create_element("md");
//                         md.set_inner_html("**");
//                         md.set_attribute("hidden", "").unwrap();
//                         elem.append_child(&md).unwrap();
//                         let children = imd_text_to_dom(&branch.children);
//                         for child in &children {
//                             elem.append_child(&child).unwrap();
//                         }
//                         let md = create_element("md");
//                         md.set_inner_html("**");
//                         md.set_attribute("hidden", "").unwrap();
//                         elem.append_child(&md).unwrap();
//                         elem_spans.push(elem);
//                         elem_num += 1;
//                     }
//                     MDTextType::Italic => {

//                     },
//                     _ => {},
//                 }
//             }
//         }
//     }
//     elem_spans
// }


// fn spaces_to_nbsp(text: &str) -> String {
//     let mut content = String::new();
//     for char in text.chars() {
//         if char == ' ' {
//             content.push_str("&nbsp;");
//             // no need to do same for tabs bc 1) doesn't 
//             // seem like you can, and 2) tabs converted to blocks
//         } else {
//             content.push(char);
//         }
//     }
//     content
// }

// fn get_hideable_md_type(elem: &Element) -> HideableMDType {
//     let kind = elem.get_attribute("type")
//         .unwrap_or("".to_string());
//     match kind.as_str() {
//         "h1" => HideableMDType::H1,
//         "b" => HideableMDType::Bold,
//         _ => HideableMDType::None,
//     }
// }
// fn get_span_or_block_info(elem: &Element) -> (MDNodeType, usize) {
//     // get node type
//     if let Some(block_num) = elem.get_attribute("block") {
//         (MDNodeType::Block, block_num.parse().unwrap())
//     } else {
//         let span_num = elem.get_attribute("text").unwrap();
//         (MDNodeType::Span, span_num.parse().unwrap())
//     }
// }

// fn hide_md(node_info: &MDNodeInfo, elem: &Element) {
//     // FIXME: just realized if the structure of the file changes, 
//     // these unwraps could fail
//     match node_info.2 {
//         HideableMDType::H1 => {
//             let hidden_md = elem.first_element_child().unwrap();
//             hidden_md.set_attribute("hidden", "").unwrap();
//         },
//         HideableMDType::Bold => {
//             let hidden_md = elem.first_element_child().unwrap();
//             hidden_md.set_attribute("hidden", "").unwrap();
//             let hidden_md = elem.last_element_child().unwrap();
//             hidden_md.set_attribute("hidden", "").unwrap();
//         },
//         _ => {},
//     }
// }
// fn unhide_md(node_info: &MDNodeInfo, elem: &Element) {
//     match node_info.2 {
//         HideableMDType::H1 => {
//             let hidden_md = elem.first_element_child().unwrap();
//             hidden_md.remove_attribute("hidden").unwrap();
//         },
//         HideableMDType::Bold => {
//             let hidden_md = elem.first_element_child().unwrap();
//             hidden_md.remove_attribute("hidden").unwrap();
//             let hidden_md = elem.last_element_child().unwrap();
//             hidden_md.remove_attribute("hidden").unwrap();
//         },
//         _ => {},
//     }
// }

// // this is some *very* cool /text/

// // #[component]
// // pub fn H1(cx: Scope, contents: String) -> Element {
// //     // go through contents to check if anything here 
// //     view! {cx,
// //         <h1>"hi"</h1>
// //     }
// // }
