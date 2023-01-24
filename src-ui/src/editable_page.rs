use std::collections::HashMap;
use core::{cmp::max, fmt::Debug};
use leptos::{*, js_sys::{Math, Date}};
use serde::Serialize;
use tauri_sys::{event, tauri};
use web_sys::HtmlDivElement;
// use src_ui::*;
use super::{
    Page, PageNode, PageNodeType, PageNodeContents, EdgeElem, add_hashes,
    get_top_hash, get_nodes_in_view, get_hash_of_next_elem, get_hash_of_prev_elem,
    get_node_from_hash, 
};

// TODO: CUSTOMIZABLE MARKDOWN SYNTAX. E.G. IF YOU WANT `/` FOR ITALICS YOU CAN 
// JUST PICK IT

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

    let page_data: RwSignal<Page> = Page::signal_from(cx,
        create_rw_signal(cx, Vec::new()),
        EdgeElem::signal_from(cx, "".into(), 0, 0),
        EdgeElem::signal_from(cx, "".into(), 0, 0),
        create_rw_signal(cx, HashMap::new()),
    );
    page_data.update(|p| {
        p.nodes.update(|n| {
            for _ in 0..5 {
                n.push(PageNode::signal_from(cx,
                    "".into(), PageNodeType::H1,
                    PageNodeContents::signal_from_children(
                        cx, vec![
                            PageNode::signal_from(cx,
                                "".into(), PageNodeType::RawText,
                                PageNodeContents::signal_from_content(
                                    cx, HashMap::from([
                                        ("text".to_string(), "some text".to_string())
                                    ])
                                ), 0
                            )
                        ]
                    ), 0
                ));
                n.push(PageNode::signal_from(cx,
                    "".into(), PageNodeType::TextBlock,
                    PageNodeContents::signal_from_children(
                        cx, vec![
                            PageNode::signal_from(cx,
                                "".into(), PageNodeType::RawText,
                                PageNodeContents::signal_from_content(
                                    cx, HashMap::from([
                                        ("text".to_string(), "some text".to_string())
                                    ])
                                ), 0
                            )
                        ]
                    ), 0
                ));
            }
        })
    });
    add_hashes(page_data.get().nodes.get(), Vec::new(), page_data.get().locations);
    // ==init top/bot-elem==

    let top_hash = get_top_hash(&page_data.get().nodes.get());
    // console_log(&format!("top hash: {:?}", top_hash));
    page_data.get().top_elem.update(|e| e.hash = top_hash.clone());
    page_data.get().bot_elem.update(|e| e.hash = top_hash);
    // ==init nodes_in_view==
    // update_nodes_in_view(cx, page_data);

    // console_log(&format!("SIZE: {:?}", std::mem::size_of_val(&10_i32))); // 4 bytes
    // console_log(&format!("SIZE: {:?}", std::mem::size_of_val(&page_data))); // 20
    // console_log(&format!("SIZE: {:?}", std::mem::size_of_val(&page_data.get()))); // 100
    // console_log(&format!("SIZE: {:?}", std::mem::size_of_val(&page_data.get().nodes))); // 20
    // console_log(&format!("SIZE: {:?}", std::mem::size_of_val(&page_data.get().nodes.get()))); // 12

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

    // on-load render and get the height of 

    // on-render, update height. don't worry about bottom items. they will 
    // update once scrolled down to and then never have to have a weird scroll 
    // again
    // if an item changes it will re-render, and thus update its height
    //
    // only need to get height for leaf blocks, height of branches will not be 
    // set, instead relying on their set padding values to add to the total 
    // height calculation
    // thus, we would also need to make sure that branch blocks do not contain 
    // any raw text, instead must contain e.g. H1, but default text box

    let elem_ref: NodeRef<HtmlElement<Div>> = NodeRef::new(cx);

    // TODO: ⬇️⬇️⬇️⬇️

    // always render new page from the top

    // init first-node and last-node in view to zero
    // render node -> if node is not last, get next node, update last-node, 
    // trigger reactive re-render, 

    // on scroll/resize, update first/last-node, trigger reactive re-render

    let init_render_toggle = create_rw_signal(cx, true);

    // ==KEEP RERENDERING UNTIL EITHER BOT_ELEM IS ON THE BOTTOM EDGE OF THE 
    // VIEW, OR IT IS THE LAST ELEMENT==
    create_effect(cx, move |_| {
        // console_log("RUNNING");
        let refresh_toggle = init_render_toggle.get();
        if let Some(elem) = elem_ref.get() {
            request_animation_frame(move || {
                let editor_bottom_edge = elem.get_bounding_client_rect().bottom();
                let bot_elem_hash = page_data.get().bot_elem.get().hash;
                // console_log(&format!("hash for selector:: {:?}", bot_elem_hash));
                let bottom_elem = query_dom_page_node(&bot_elem_hash);
                let bot_top_edge = bottom_elem.get_bounding_client_rect().top();
                // console_log(&format!("editor:: {:?}", editor_bottom_edge));
                // console_log(&format!("bottom elem:: {:?}", bot_top_edge));
                if bot_top_edge < editor_bottom_edge {
                    console_log("ELEM NOT AT BOTTOM");
                    if let Some(next_hash) = get_hash_of_next_elem(&bot_elem_hash, page_data) {
                        // console_log(&format!("next hash: {:?}", next_hash));
                        // update bot_elem hash
                        page_data.get().bot_elem.update(|e| e.hash = next_hash);
                        // update nodes_in_view to trigger re-render
                        // update_nodes_in_view(cx, page_data);
                        // trigger refresh of this closure so bot_elem can 
                        // change again if this is not the bottom element
                        init_render_toggle.set(!refresh_toggle);
                    };
                }
            });
        } else {
            // this will be just the first run while the elem_ref is still 
            // empty bc haven't yet executed the `_ref=` thing in the view
        }
    });

    // FIXME: MIGHT NEED TO REMOVE THE `create_effect` SO IT DOESNT KEEP 
    // RUNNING EVERYTIME THE SIGNAL CHANGES
    // need to run this post-render to make sure to top-elem has its hash
    // create_effect(cx, move |_| { 
    //     let toggle = page_data.get().refresh_toggle;
    //     request_animation_frame(move || {
    //     page_data.update(|p| p.refresh_toggle = !p.refresh_toggle);
    //     console_log("SET TOP_ELEM IF MISSING");

    //     console_log(&format!("{:?}", page_data.get().refresh_toggle));

    // }); });


    // TODO: ALSO NEED TO SET TOP PADDING RIGHT AFTER INITIAL RENDER, AND SET SCROLL POSITION

    let scroll_throttle = store_value(cx, 0.0);
    let handle_scroll = move |event: web_sys::Event| {
        // this is firing WAAAAAAAAY too quickly. so much so that if an element 
        // from the top if removed, ALL elements immediately get removed before 
        // the padding div is able to save them. thus, we need to throttle this 
        // handler
        // FIXME: WOOOOOOOOOOOOOOOOW THIS IS JANKY. back to the drawing board? render whole page at once?
        let now = Date::now();
        if now > scroll_throttle.get() + 50.0 {
            scroll_throttle.set(now);
        } else {
            return;
        }

        // TODO: ONCE GET NEXT/PREV ELEM, QUERY ITS EDGE POSITION AND IF STILL 
        // REQUIRES ANOTHER NEXT/PREV ELEM, KEEP GETTING THEM UNTIL WE ARE IN 
        // RANGE !!!!!!!, AND ONLY THEN UPDATE THE TOP/BOT ELEM

        console_log("scroll");
        let page_elem: HtmlDivElement = event.target().unwrap().dyn_into().unwrap();
        // let scroll_top = page_elem.scroll_top();
        // console_log(&format!("SCROLL TOP: {:?}", scroll_top));
        let page_top_edge = page_elem.get_bounding_client_rect().top();
        let page_bot_edge = page_elem.get_bounding_client_rect().bottom();

        let top_elem = page_data.get().top_elem;
        let bot_elem = page_data.get().bot_elem;

        const REMOVE_DISTANCE: f64 = 50.0;
        const ADD_DISTANCE: f64 = 10.0;

        let top_elem_hash = top_elem.get().hash;
        let top_element = query_dom_page_node(&top_elem_hash);
        let top_elem_bot_edge = top_element.get_bounding_client_rect().bottom();
        let px_above_top = page_top_edge - top_elem_bot_edge;
        // if bot edge of top_elem is less than page_top_edge, we need to 
        // render the element above it
        if px_above_top < ADD_DISTANCE {
            console_log("RENDER ONE ABOVE !!!");
            // loop
            if let Some(prev_elem_hash) = get_hash_of_prev_elem(&top_elem_hash, page_data) {
                let height = get_node_from_hash(&prev_elem_hash, page_data)
                    .unwrap().get().height;
                // update top_elem to trigger re-render
                top_elem.update(|e| {
                    e.hash = prev_elem_hash;
                    // rm previously added padding
                    let total = e.pad as f64 - height as f64;
                    e.pad = total as u32; // if total is negative it is converted to 0
                });
            };
        // if bot edge of top_elem is greater than page_top_edge by 50px, we 
        // need to remove the element
        } else if px_above_top > REMOVE_DISTANCE {
            console_log("REMOVE TOP");
            // update height of node
            let height = top_element.get_bounding_client_rect().height();
            // console_log(&format!("HEIGHT ADDED: {:?}", height));
            let top_node = get_node_from_hash(&top_elem_hash, page_data).unwrap();
            top_node.update(|n| n.height = height as u32);

            if let Some(next_elem_hash) = get_hash_of_next_elem(&top_elem_hash, page_data) {
                // update top_elem to trigger re-render
                top_elem.update(|e| {
                    e.hash = next_elem_hash.clone();
                    let total = e.pad as f64 + height;
                    e.pad = total as u32;
                });
            };
        }

        // FIXME: I JUST REALIZED THIS WAY OF RENDERING MIGHT FUCK UP IF I 
        // SCROLL TOO FAST, OR MORE IMPORTANTLY IF I CLICK A SUBHEADING IN THE 
        // OUTLINE OR CLICK THE BOTTOM OF THE PAGE ON THE SCROLLBAR AND IT 
        // JUMPS STRAIGHT THERE WITHOUT GIVING IT TIME TO RENDER
        // prob some ways to patch this, but i wonder if there is a better was 
        // to render altogether 🤔
        // TODO: 🚨🚨🚨🚨🚨 hmm actually, what if the editable page element is simply a 
        // <div id="knfkd" />, and i give it a f-tonne of callbacks and such 
        // (avoiding leptos rendering) 🚨🚨🚨🚨🚨
        // MIGHT EVEN BE THE WHOLE "ONLY RENDER NEXT ELEM" THING THAT'S FUCKING 
        // EVERYTHING UP
        // TODO: on the other hand, maybe i'm just computing WAAAAAAAAAY too much 
        // shit (again with the recomputation every single next element). 
        // perhaps only update base blocks ??????? also check to see where i'm 
        // going overkill on the code
        // TODO: REFERENCING DATA STRUCTURE MAY SPEED THINGS UP A LOOOOOOOOOOOOT
        // TODO: PROBABLY ALSO DOESN'T HELP THAT BY USING <FOR>, THE WHOLE DATA 
        // STRUCTURE NEEDS TO BE RELOADED AND REPROCESSED WITH EVERY NEW RENDER. 
        // NOT TO MENTION ALL THE COPIES FOR ALL THE .GET() CALLS !!! ALL BC OF 
        // THE <FOR>. GO CUSTOM
        // TODO: I CAN USE NODEREF FOR DIRECT DOM MANIPULATION. COMPLETELY 
        // BYPASSING LEPTOS !!!!!!!!!!!!!!!
        // TODO: THEN AGAIN, ISNT LEPTOS FASTET BC IT SENDS HTML STRINGS 
        // INSTEAD OF USING MANY DOM CALLS ?? I MIGHT NEED TO SIMPLY FIND A WAY 
        // TO SYNC THE PADDING DIV UPDATE WITH THE PAGE NODES UPDATE

        // TODO: CALC TOTAL HEIGHT WHEN HIT BOTTOM AND ADD PADDING DIV ACCORDING TO THAT ???????? (making sure padding > 0)

        let bot_elem_hash = bot_elem.get().hash;
        let bot_element = query_dom_page_node(&bot_elem_hash);
        let bot_elem_top_edge = bot_element.get_bounding_client_rect().top();
        let px_below_bot = bot_elem_top_edge - page_bot_edge;
        // if top edge of bot_elem is less than page_bot_edge, we need to 
        // render the element below it
        if px_below_bot < ADD_DISTANCE {
            // console_log("RENDER ONE BELOW");
            if let Some(next_elem_hash) = get_hash_of_next_elem(&bot_elem_hash, page_data) {
                // update bot_elem to trigger re-render
                bot_elem.update(|e| e.hash = next_elem_hash);
            };
        // if top edge of bot_elem is greater than page_bot_edge by 50px, we 
        // need to remove the element
        } else if px_below_bot > REMOVE_DISTANCE {
            // console_log("REMOVE BOT");
            if let Some(prev_elem_hash) = get_hash_of_prev_elem(&bot_elem_hash, page_data) {
                // update bot_elem to trigger re-render
                bot_elem.update(|e| e.hash = prev_elem_hash);
            };
        }
    };

    // create_effect(cx, move |_| {
    //     request_animation_frame(move || {
    //         console_log(&format!("{:?}", document().query_selector("[type=t]").unwrap().unwrap().get_bounding_client_rect().top()));
    //     });
    // });

    // prob better to alter `padding` of top root elem instead of `top` (bc i dont think `top`/`bottom` would work)

    let nodes_in_view = create_rw_signal(cx, Vec::new());

    create_effect(cx, move |_| {
        let locations = page_data.get().locations.get();
        let top_elem = locations.get(
            &page_data.get().top_elem.get().hash).unwrap();
        let bot_elem = locations.get(
            &page_data.get().bot_elem.get().hash).unwrap();
        let new = get_nodes_in_view(cx, page_data.get().nodes.get(), top_elem, bot_elem);
        nodes_in_view.set(new);
    });

    view! {cx,
        <div contenteditable
        style="overflow-y: auto; height:150px;"
        type="page"
        on:scroll=handle_scroll
        // on:keydown=handle_keypress
        _ref=elem_ref
        >
            <div contenteditable="false" style=move ||{ format!("height: {}px", page_data.get().top_elem.get().pad)} />
            <PageNodes
            page_data=page_data
            nodes=nodes_in_view />
        </div>
    }
}

/// query a PageNode element from the DOM by its hash
fn query_dom_page_node(hash: &String) -> web_sys::Element {
    // HASH CANNOT START WITH NUMBER LIKE THIS "[hash=29w8r]" BC IT CRASHES 
    // QUERYSELECTOR. MUST USE "[hash=\"29w8r\"]" INSTEAD
    let selector = format!("[hash=\"{}\"]", hash);
    document().query_selector(&selector).unwrap().unwrap()
}

// TODO: if compare doesn't work to find the number of the child, use the hash after-all, purely for unique child-nodes. ohhhh wait this wouldn't work bc i'm not rendering all the nodes to the page. MAYBE BETTER TO JUST SAY FUCK REACTVITY AND RE-RENDER ALL VISIBLE (AT LEAST ALL THAT NEED TO UPDATE INDEXES) WHEN UPDATE OCCURS


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


// TODO: 
/// update all hash postions for all nodes in the level post-insert/delete
fn update_positions() {

}

#[component]
pub fn PageNodes(cx: Scope,
    page_data: RwSignal<Page>,
    nodes: RwSignal<Vec<RwSignal<PageNode>>>,
) -> impl IntoView {

    // TODO: COMMIT, THEN TEST CONVERSION TO <FOR />. IF THE UPDATING FROM 
    // SCROLLING IS NOT REACTIVE, REVERT MOST CHANGES AND DO MANUAL DOM 
    // UPDATES IN SCROLL HANDLER

    // TODO: COMMIT CHANGES, THE DECIDE ON EITHER PLACING ABOVE CODE IN THE 
    // view!{} so it executes on every update, OR, leave this function as an 
    // init component, and update the DOM directly in the scroll-handler

    view! {cx,
        <For each=nodes key=|n| n.get().hash view=move |node| {
            let node = node.get();
            let node_hash = node.hash.clone();
            match node.contents {
                PageNodeContents::Children(nodes) => {
                    match node.kind {
                        PageNodeType::H1 => view! {cx,
                            <div //contenteditable
                            type=PageNodeType::H1.value()
                            hash=node_hash>
                                <PageNodes page_data nodes  />
                            </div>
                        },
                        PageNodeType::TextBlock => view! {cx,
                            <div
                            type=PageNodeType::TextBlock.value()
                            hash=&node.hash>
                                <PageNodes page_data nodes  />
                            </div>
                        },
                        _ => view! {cx,
                            <div hash=node_hash>"‼️ block missing"</div>
                        },
                    }.into_any()
                }
                PageNodeContents::Content(content) => {
                    match node.kind {
                        PageNodeType::RawText => view! {cx,
                            <span
                            type=PageNodeType::RawText.value()
                            hash=node_hash>
                                {content.get().get("text").unwrap()}
                            </span>
                        },
                        _ => view! {cx,
                            <span
                            type=PageNodeType::RawText.value()
                            hash=node_hash>
                                "‼️ only raw text allowed"
                            </span>
                        },
                    }.into_any()
                }
            }
        }/>
    }
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
