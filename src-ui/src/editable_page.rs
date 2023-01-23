use std::{collections::{HashMap, hash_map::DefaultHasher}, hash::{Hash, Hasher, self}};
use core::{cmp::max, fmt::Debug};
use leptos::{*, js_sys::Math};
use serde::Serialize;
use tauri_sys::{event, tauri};
use web_sys::HtmlDivElement;
// use src_ui::*;

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
    pub nodes: RwSignal<Vec<RwSignal<PageNode>>>,
    pub nodes_in_view:  RwSignal<Vec<RwSignal<PageNode>>>,
    /// (hash, location, top)
    /// also use this to calculate scroll position
    pub top_elem: RwSignal<EdgeElem>,
    pub bot_elem: RwSignal<EdgeElem>,
    pub locations: RwSignal<HashMap<String, Vec<usize>>>,
}
impl Page {
    fn signal_from(cx: Scope, nodes: RwSignal<Vec<RwSignal<PageNode>>>, 
        top_elem: RwSignal<EdgeElem>, bot_elem: RwSignal<EdgeElem>, 
        locations: RwSignal<HashMap<String, Vec<usize>>>
    ) -> RwSignal<Self> {
        let nodes_in_view = create_rw_signal(cx, Vec::new());
        create_rw_signal(cx, Self {nodes, nodes_in_view, top_elem, 
            bot_elem, locations}) 
    }
}
/// the top or bottom element of the view
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EdgeElem {
    // the hash/id of top element
    pub hash: String,
    // pub location: Vec<usize>, // location not needed bc can use hash to get it from hashset
    /// - if top elem: top = `padding-top` attribute, bot elem: `padding-bot`
    /// - padding is applied to the base node  (e.g. if node is `vec![1, 3, 2]`, 
    /// padding applied to base node of index 1)
    pub pad: u32,
    /// - if top elem: bottom edge of the elem. once it passes over the top of 
    /// the page + some px, it signals the element should be unrendered, and 
    /// new top-elem chosen
    pub inner_edge_y: i32,
}
impl EdgeElem {
    fn from(hash: String, pad: u32, inner_edge_y: i32) -> Self {
        Self {hash, pad, inner_edge_y}
    }
    fn signal_from(cx: Scope, hash: String, pad: u32, inner_edge_y: i32) -> RwSignal<Self> {
        create_rw_signal(cx, Self {hash, pad, inner_edge_y})
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PageNode {
    pub hash: String,
    pub kind: PageNodeType,
    pub contents: PageNodeContents,
    /// height of all elems is tracked so we can have an accurate scroll page 
    /// length without having to render the page down to the bottom
    pub height: usize,
    // /// the y-axis top of the element in pixels
    // pub top: usize,
    // /// the y-axis bottom of the element in pixels
    // pub bottom: usize,
}
impl PageNode {
    fn from(hash: String, kind: PageNodeType, 
        contents: PageNodeContents, height: usize,
    ) -> Self {
        Self {hash, kind, contents, height}
    }
    fn signal_from(cx: Scope, hash: String, kind: PageNodeType, 
        contents: PageNodeContents, height: usize,
    ) -> RwSignal<Self> {
        create_rw_signal(cx, Self {hash, kind, contents, height})
    }
}
// struct RwSignal<T>()
// impl Debug for RwSignal<PageNode> {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(f, "RwSignal PageNode({:?}, {:?})", self.hash, self.kind)
//     }
// }
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PageNodeContents {
    Children(RwSignal<Vec<RwSignal<PageNode>>>), Content(RwSignal<HashMap<String, String>>)
}
impl PageNodeContents {
    fn signal_from_children(cx: Scope, children: Vec<RwSignal<PageNode>>) -> Self {
        Self::Children(create_rw_signal(cx, children))
    }
    fn signal_from_content(cx: Scope, content: HashMap<String, String>) -> Self {
        Self::Content(create_rw_signal(cx, content))
    }
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
    fn is_block(&self) -> bool {
        match *self {
            PageNodeType::Quote => true,
            PageNodeType::TextBlock => true,
            PageNodeType::H1 => true,
            PageNodeType::H2 => true,
            PageNodeType::H3 => true,
            PageNodeType::CodeBlock => true,
            PageNodeType::Bold => false,
            PageNodeType::Italic => false,
            PageNodeType::Highlight => false,
            PageNodeType::CodeInline => false,
            PageNodeType::FileLink => false,
            PageNodeType::UrlLink => false,
            PageNodeType::RawText => false,
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
    fn add_hashes(nodes: Vec<RwSignal<PageNode>>, location: Vec<usize>, 
        locations: RwSignal<HashMap<String, Vec<usize>>>) {
        for (i, node) in nodes.iter().enumerate() {
            let mut location = location.clone();
            location.push(i);
            // create & add hash/location if not present
            if node.get().hash == "".to_string() {
                let mut hash = rand_alphanumerecimal_hash();
                loop {
                    if !locations.get().contains_key(&hash) { break }
                    hash = rand_alphanumerecimal_hash();
                }
                locations.update(|h| {
                    h.insert(hash.clone(), location.clone());
                });
                nodes[i].update(|e| e.hash = hash.clone())
            }
            // if children present in node, update those too
            match node.get().contents {
                PageNodeContents::Children(children) => {
                    add_hashes(children.get(), location, locations)
                },
                _ => {},
            }
        };
    }
    add_hashes(page_data.get().nodes.get(), Vec::new(), page_data.get().locations);

    // ==init top/bot-elem==
    let mut top_hash = page_data.get().nodes.get()[0].get().hash;
    console_log(&format!("top hash: {:?}", top_hash));
    let mut node = page_data.get().nodes.get()[0].get();
    loop {
        match node.contents {
            PageNodeContents::Children(children) => {
                // if children are not blocks, this is the final block node
                let first_child = children.get()[0].get();
                if !children.get()[0].get().kind.is_block() {
                    break;
                } else {
                    top_hash = node.hash;
                    node = first_child;
                }
            }
            PageNodeContents::Content(_) => break,
        }
    }
    console_log(&format!("top hash: {:?}", top_hash));
    page_data.get().top_elem.update(|e| {
        e.hash = top_hash.clone()
    });
    page_data.get().bot_elem.update(|e| {
        e.hash = top_hash
    });

    // ==init nodes_in_view==
    fn get_nodes_in_view(cx: Scope, nodes: Vec<RwSignal<PageNode>>, 
        top_loc: &Vec<usize>, bot_loc: &Vec<usize>,
    ) -> Vec<RwSignal<PageNode>> {
        let mut vec = Vec::new();
        let mut iter = nodes.iter();

        // if `top_loc[1]` = 0, this of course wouldn't be sliced, but can't 
        // add that to this var bc if len is 3 you would need to check all 3, 
        // if len was 4 you need to check all 4, etc, etc, which seems like 
        // wasted effort when we can just let the loop check. this is simply a 
        // quick preliminary check
        let start_node_may_be_sliced = top_loc.len() > 1;
        let end_node_may_be_sliced = top_loc.len() > 1;
        // let a_node_may_be_sliced = start_node_may_be_sliced || end_node_may_be_sliced;

        // if no index, it means e.g. start from beginning, or e.g. end at end
        let start_idx = top_loc.get(0);
        let end_idx = bot_loc.get(0);
        if let Some(idx) = start_idx {
            iter.advance_by(*idx).unwrap();
        }
        let mut idx = match start_idx {
            Some(idx) => *idx,
            None => 0,
        };
        for node in iter {
            console_log(&format!("LOOP IDX: {:?}", idx));
            console_log(&format!("NODE KIND: {:?}", node.get().kind));
            console_log(&format!("VEC: {:?}", vec));
            
            let is_start_elem = idx == *start_idx.unwrap_or(&usize::MAX);
            let is_end_elem = idx == *end_idx.unwrap_or(&usize::MAX);

            let start_elem_and_may_be_sliced = is_start_elem && start_node_may_be_sliced;
            let end_elem_and_may_be_sliced = is_end_elem && end_node_may_be_sliced;

            // only start and end nodes are able to be sliced
            if start_elem_and_may_be_sliced || end_elem_and_may_be_sliced {
                let mut top_loc = top_loc.clone();
                let mut bot_loc = bot_loc.clone();

                if let PageNodeContents::Children(
                    children_signal
                ) = node.get().contents {

                    if start_elem_and_may_be_sliced {
                        top_loc.splice(0..1, []); // rm first elem so children in recursion get location in relation to their position
                    // if this is not start elem, we must pass top_loc = &Vec::new() 
                    // so the recursion doesn't get a start index
                    } else { top_loc.clear() }
                    if end_elem_and_may_be_sliced {
                        bot_loc.splice(0..1, []); // rm first elem so children in recursion get location in relation to their position
                    // if this is not end elem, we must pass bot_loc = &Vec::new()
                    // so the recursion doesn't get an end index
                    } else { bot_loc.clear() }
    
                    let child_nodes = get_nodes_in_view(
                        cx, children_signal.get(), &top_loc, &bot_loc
                    );
                    // if this is not sliced (e.g. if top elem and all the 
                    // children slice at 0), we check if it matches the 
                    // children_signal so we don't have to create a new signal 
                    // if we don't have to. if matches, its not actually 
                    // sliced, so keep same signal

                    // same ->  not a slice
                    if child_nodes == children_signal.get() {
                        vec.push(*node);
                    // different ->  a slice
                    } else {
                        let mut node = node.get();
                        node.contents = PageNodeContents::Children(
                            create_rw_signal(cx, child_nodes)
                        );
                        vec.push(create_rw_signal(cx, node));
                    }
                } else {
                    vec.push(*node);
                }
            // for nodes that aren't start node or end node, or if no nodes 
            // are sliced, in both cases we can simply pass the entire node 
            // (no need to create a new signal)
            } else {
                vec.push(*node);
            }
            if is_end_elem { break }
            idx += 1;
        }
        vec
    }
    fn update_nodes_in_view(cx: Scope, page_data: RwSignal<Page>) {
        let locations = page_data.get().locations.get();
        let top_elem = locations.get(
            &page_data.get().top_elem.get().hash
        ).unwrap();
        let bot_elem = locations.get(
            &page_data.get().bot_elem.get().hash
        ).unwrap();
        console_log(&format!("LOCATIONS: {:?}", locations));
        console_log(&format!("TOP ELEM: {:?}", top_elem));
        page_data.get().nodes_in_view.set(get_nodes_in_view(
            cx, page_data.get().nodes.get(), top_elem, bot_elem
        ));
    }
    // ==UPDATE NODES IN VIEW==
    update_nodes_in_view(cx, page_data);

    console_log(&format!("NUM NODES IN VIEW: {:?}", page_data.get().nodes_in_view.get().len()));

    // FIXME: THIS WILL NOT WORK BC IF I'M GETTING A SIGNAL TO NODE Vec::new(), 
    // IT ISN'T POSSIBLE TO THEN MODIFY THE CONTENTS (I.E. MAKE A SLICE OF ITS 
    // CHILD NODES) BC THEN YOU'RE EITHER MODIFYING THE ORIGINAL DATA, OR 
    // YOU'RE MAKING A COPY WHICH WE DON'T WANT TO DO
    // i guess i must just make a copy of at least the block nodes for in-view 
    // if i want to do this

    // ok so lets start by doing rendering conditional upon being within view, 
    // then appending hash to hashset if item is rendered
    // hashset could either contain the path to the item, or a closure that contains an item.update(|i| *i = x)

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
        console_log("RUNNING");
        let refresh_toggle = init_render_toggle.get();
        if let Some(elem) = elem_ref.get() {
            request_animation_frame(move || {
                let editor_bottom_edge = elem.get_bounding_client_rect().bottom();
                let bot_elem_hash = page_data.get().bot_elem.get().hash;
                console_log(&format!("hash for selector:: {:?}", bot_elem_hash));
                // HASH CANNOT START WITH NUMBER LIKE THIS "[hash=29w8r]" BC 
                // IT CRASHES QUERYSELECTOR. MUST USE "[hash=\"29w8r\"]" INSTEAD
                let selector = format!("[hash=\"{}\"]", bot_elem_hash);
                let bottom_elem = document().query_selector(&selector).unwrap().unwrap();
                let bot_top_edge = bottom_elem.get_bounding_client_rect().top();
                console_log(&format!("editor:: {:?}", editor_bottom_edge));
                console_log(&format!("bottom elem:: {:?}", bot_top_edge));
                if bot_top_edge < editor_bottom_edge {
                    console_log("ELEM NOT AT BOTTOM");
                    if let Some(next_hash) = get_hash_of_next_elem(&bot_elem_hash, page_data) {
                        console_log(&format!("next hash: {:?}", next_hash));
                        // update bot_hash, then update nodes_in_view to 
                        // trigger re-render
                        page_data.get().bot_elem.update(|e| e.hash = next_hash);
                        update_nodes_in_view(cx, page_data);
                        init_render_toggle.set(!refresh_toggle);
                    };
                }
            });
        }
    });

    fn get_hash_of_next_elem(hash: &String, page_data: RwSignal<Page>
    ) -> Option<String> {
        let mut location = page_data.get().locations.get()
            .get(hash).unwrap().to_owned();
        console_log(&format!("location: {:?}", location));
        let nodes = page_data.get().nodes.get();
        // first check next child (e.g. input location is [0, 2], so check [0, 3])
        // then jump one level down and do same, etc until at base and still no 
        // next elems
        loop {
            // create theoretical location of next child on same level
            let loc_len = location.len();
            location[loc_len - 1] += 1;

            // * check if this location exists *
            if let Some(next_hash) = get_hash_from_location(&location, &nodes) {
                return Some(next_hash);
            };

            // jump one level down
            location.pop().unwrap();
            // if this is true, the input hash is already the last element
            if location.len() == 0 { console_log("ALREADY LAST"); return None }
        }
    }
    fn get_hash_from_location(location: &Vec<usize>, nodes: &Vec<RwSignal<PageNode>>
    ) -> Option<String> {
        let mut node = match nodes.get(location[0]) {
            Some(node) => node.clone(),
            None => return None,
        };
        for idx in &location[1..] {
            match node.get().contents {
                PageNodeContents::Children(children) => {
                    match children.get().get(*idx) {
                        Some(child) => {
                            node = child.clone();
                        },
                        None => return None,
                    }
                },
                _ => { panic!("should never get here") },
            }
        }
        Some(node.get().hash)
    }

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


    // ALSO NEED TO SET TOP PADDING RIGHT AFTER INITIAL RENDER, AND SET SCROLL POSITION



    let handle_scroll = move |event: web_sys::Event| {
        console_log("scroll");
        let top_elem = page_data.get().top_elem.get();
        let bot_elem = page_data.get().bot_elem.get();
        let elem: HtmlDivElement = event.target().unwrap().dyn_into().unwrap();
        let scroll_top = elem.scroll_top();
        console_log(&format!("SCROLL TOP: {:?}", scroll_top));

        console_log(&format!("{:?}", document().query_selector("[type=t]").unwrap().unwrap().get_bounding_client_rect().top()));
    };

    // create_effect(cx, move |_| {
    //     request_animation_frame(move || {
    //         console_log(&format!("{:?}", document().query_selector("[type=t]").unwrap().unwrap().get_bounding_client_rect().top()));
    //     });
    // });

    // prob better to alter `padding` of top root elem instead of `top` (bc i dont think `top`/`bottom` would work)

    view! {cx,
        <div contenteditable
        style="overflow-y: auto; height:150px;"
        type="page"
        on:scroll=handle_scroll
        // on:keydown=handle_keypress
        _ref=elem_ref
        >
            <PageNodes
            page_data=page_data
            nodes=page_data.get().nodes_in_view />
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
                // `'a' == 97 as char`
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
