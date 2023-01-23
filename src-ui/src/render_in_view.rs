use leptos::{Scope, RwSignal, create_rw_signal, console_log};
use super::{Page, PageNode, PageNodeContents};

pub fn get_top_hash(nodes: &Vec<RwSignal<PageNode>>) -> String {
    let mut top_hash = nodes[0].get().hash;
    // console_log(&format!("top hash: {:?}", top_hash));
    let mut node = nodes[0].get();
    loop {
        match node.contents {
            PageNodeContents::Children(children) => {
                // if children are not blocks, this is the final block node
                let first_child = children.get()[0].get();
                if !children.get()[0].get().kind.is_block() {
                    break;
                }
                top_hash = node.hash;
                node = first_child;
            }
            PageNodeContents::Content(_) => break,
        }
    }
    top_hash
}
fn get_bot_hash(nodes: &Vec<RwSignal<PageNode>>) -> String {
    let mut last_idx = nodes.len() - 1;
    let mut bot_hash = nodes[last_idx].get().hash;
    // console_log(&format!("top hash: {:?}", top_hash));
    let mut node = nodes[last_idx].get();
    loop {
        match node.contents {
            PageNodeContents::Children(children) => {
                // if children are not blocks, this is the final block node
                let child_nodes = children.get();
                last_idx = child_nodes.len() - 1;
                let last_child = child_nodes[last_idx].get();
                if !child_nodes[last_idx].get().kind.is_block() {
                    break;
                }
                bot_hash = node.hash;
                node = last_child;
            }
            PageNodeContents::Content(_) => break,
        }
    }
    bot_hash
}

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
        // console_log(&format!("LOOP IDX: {:?}", idx));
        // console_log(&format!("NODE KIND: {:?}", node.get().kind));
        // console_log(&format!("VEC: {:?}", vec));
        
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
pub fn update_nodes_in_view(cx: Scope, page_data: RwSignal<Page>) {
    let locations = page_data.get().locations.get();
    let top_elem = locations.get(
        &page_data.get().top_elem.get().hash
    ).unwrap();
    let bot_elem = locations.get(
        &page_data.get().bot_elem.get().hash
    ).unwrap();
    // console_log(&format!("LOCATIONS: {:?}", locations));
    // console_log(&format!("TOP ELEM: {:?}", top_elem));
    page_data.get().nodes_in_view.set(get_nodes_in_view(
        cx, page_data.get().nodes.get(), top_elem, bot_elem
    ));
}

pub fn get_hash_of_next_elem(hash: &String, page_data: RwSignal<Page>
) -> Option<String> {
    let mut location = page_data.get().locations.get()
        .get(hash).unwrap().to_owned();
    // console_log(&format!("location: {:?}", location));
    let nodes = page_data.get().nodes.get();
    // first check next child (e.g. input location is [0, 2], so check [0, 3])
    // then jump one level down and do same, etc until at base and still no 
    // next elems
    loop {
        // create theoretical location of next child on same level
        let loc_len = location.len();
        location[loc_len - 1] += 1;

        // * check if this location exists *
        if let Some(_) = get_hash_from_location(&location, &nodes) {
            // get top nested child if exists of prev elem, bc that is the 
            // true next item
            // location -> NODE
            let nodes = vec![
                get_node_from_location(&location, &nodes).unwrap() // unwrap bc we know it exists
            ];
            let next_hash = get_top_hash(&nodes);
            return Some(next_hash);
        };

        // jump one level down
        location.pop().unwrap();
        // if this is true, the input hash is already the last element
        if location.len() == 0 { console_log("ALREADY LAST"); return None }
    }
}

// TODO: INSTEAD OF THESE ALGOS TO FIND NEXT ITEM AND SUCH, I COULD JUST UPDATE 
// THE NODES DATAT STURTCURE TO STORE PARENTS, SIBLINGS, ETC IN EACH NODE. I'D 
// HAVE TO CONFIRM THE RWSIGNAL DATATYPE IS VERY SMALL THOUGH TO JUSTIFY ADDING 
// THEIR REFERENCES TO EACH NODE

pub fn get_hash_of_prev_elem(hash: &String, page_data: RwSignal<Page>
) -> Option<String> {
    let mut location = page_data.get().locations.get()
        .get(hash).unwrap().to_owned();
    // console_log(&format!("location: {:?}", location));
    let nodes = page_data.get().nodes.get();
    // first check prev child (e.g. input location is [0, 3], so check [0, 2])
    // then jump one level down and do same, etc until at base and still no 
    // prev elems
    loop {
        // create theoretical location of prev child on same level
        let loc_len = location.len();
        if location[loc_len - 1] != 0 { // if the location is idx 0, we can't decrement, and dont need to bc we know there is no prev child
            location[loc_len - 1] -= 1;

            // THIS NODE MUST EXIST BC TO HAVE E.G. AND INDEX = 1, YOU MUST 
            // HAVE AN IDX = 0
            // get last nested child if exists of prev elem, bc that is the 
            // true prev item
            // location -> NODE
            let nodes = vec![
                get_node_from_location(&location, &nodes).unwrap() // unwrap bc we know it exists
            ];
            let prev_hash = get_bot_hash(&nodes);
            return Some(prev_hash)
        }

        // jump one level down
        location.pop().unwrap();
        // if this is true, the input hash is already the first element
        if location.len() == 0 { console_log("ALREADY FIRST"); return None }
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
// FIXME: LMAO almost a duplicate of function above. should rly delete above 
// and do `get_node_from_location().get().hash`. or AT LEAST convert to wrapper 
// function around this one
fn get_node_from_location(location: &Vec<usize>, nodes: &Vec<RwSignal<PageNode>>
) -> Option<RwSignal<PageNode>> {
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
    Some(node)
}

pub fn get_node_from_hash(hash: &String, page_data: RwSignal<Page>
) -> Option<RwSignal<PageNode>> {
    let locations = page_data.get().locations.get();
    get_node_from_location(
        locations.get(hash).unwrap(), 
        &page_data.get().nodes.get())
}
