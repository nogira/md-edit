use leptos::{log, Scope, RwSignal, create_rw_signal, console_log, document};
use web_sys::Element;
use super::{Page, PageNode, create_elem};

pub fn update_dom_nodes_in_view(cx: Scope, page_data: RwSignal<Page>, page_elem: &Element) {

    let page_top_edge = page_elem.get_bounding_client_rect().top();
    let page_bot_edge = page_elem.get_bounding_client_rect().bottom();

    let top_elem_data = page_data.get().top_elem;
    let bot_elem_data = page_data.get().bot_elem;

    const REMOVE_DISTANCE: f64 = 50.0;
    const ADD_DISTANCE: f64 = 20.0;

    let mut top_done = false;
    let mut bot_done = true;

    let mut new_top_node = top_elem_data.get().node_sig;
    let mut new_bot_node = bot_elem_data.get().node_sig;
    let mut new_top_pad = top_elem_data.get().pad;
    let mut new_bot_pad = bot_elem_data.get().pad;

    // I'M LOOPING THE ENTIRE TOP AND BOTTOM INSTEAD OF MINILOOPS IN EACH 
    // IF/ELSE-IF SO IF E.G. I SKIP RIGHT TO THE BOTTOM OF THE PAGE, SINCE 
    // THE TOP IS CALLED FIRST, DERENDING ALL THE TOP NODES, IT WOULD 
    // EVENTUALLY CRASH (?) BC THE TOP_ELEM WOULD GET BELOW THE BOT_ELEM

    // OHHH, IF A REMOVE AN ELEM > 3, IT SHOUDL IMMEDIATELY TRIGGER A RE-ADD 
    // WHICH FUCKS EVERYTHING UP, SO I NEED TO INCORPORATE. wait no this is 
    // what the padding is for..

    /*
    if we use bottom edge detection for both adding and removing top elems, 
    this can happen:

    if a node is too big, when the top node above it is removed, and it becomes 
    the top node, it automatically triggers a re-render of the top node that 
    was just removed, leaving you in an infinite loop

    │   ⬆️   │
    └───────┘ --------       <- remove if bottom of top node is above
    ┌───────┐
    │   ⬆️   │
    │   ⬆️   │
    │   ⬆️   │ --------       <- add if bottom of top node is below
    │   ⬆️   │
    └───────┘

    thus we instead add if the TOP of the top node goes below the lower line
    */

    while !top_done || !bot_done {
        {
        let new_top_elem = new_top_node.get().elem_ref.unwrap();
        let top_elem_top_edge = new_top_elem.get_bounding_client_rect().top();
        let top_elem_bot_edge = new_top_elem.get_bounding_client_rect().bottom();
        let px_top_above_top = page_top_edge - top_elem_top_edge;
        let px_bot_above_top = page_top_edge - top_elem_bot_edge;
        // ==RENDER THE ELEMENT ABOVE IT==
        if true && px_top_above_top < ADD_DISTANCE {
            // NOTE: THIS TRIGGERS AT TOP OF PAGE BC TOP NODE IS STILL UNDER 
            // THE TOP OF PAGE (but obv not the if statement below)
            if let Some(prev_node) = get_prev_block_node(&new_top_node.get().hash, page_data) {
                // add node to page
                let height = insert_new_node_before(cx, &page_elem, prev_node);
                // rm previously added padding
                // if total is negative (should never be) it is converted to 0 w/ `as u32`
                new_top_pad = (new_top_pad as f64 - height as f64) as u32;
                new_top_node = prev_node;
                page_elem.first_element_child().unwrap().set_attribute(
                    "style", &format!("height: {}px", new_top_pad)).unwrap();
            } else {
                // if already first, we are done
                top_done = true;
            }
        // ==REMOVE THE TOP ELEMENT==
        } else if px_bot_above_top > REMOVE_DISTANCE {
            // can only remove the current top element if there is 
            // a next element to become the top
            if let Some(next_node) = get_next_block_node(&new_top_node.get().hash, page_data) {
                // remove current top node
                let height = remove_top_elem_from_dom(new_top_node);
                // set next node to top elem
                new_top_pad = new_top_pad + height;
                new_top_node = next_node;
                page_elem.first_element_child().unwrap().set_attribute(
                    "style", &format!("height: {}px", new_top_pad)).unwrap();
            } else {
                // if already last, we are done
                top_done = true;
            }
        } else {
            top_done = true;
        }
        }
        // log!("WHILE LOOP: bot node hash: {}", new_bot_node.get().hash);
        // let new_bot_elem = new_bot_node.get().elem_ref.unwrap();
        // let bot_elem_top_edge = new_bot_elem.get_bounding_client_rect().top();
        // let bot_elem_bot_edge = new_bot_elem.get_bounding_client_rect().bottom();
        // let px_top_below_bot = bot_elem_top_edge - page_bot_edge;
        // let px_bot_below_bot = bot_elem_bot_edge - page_bot_edge;
        // // log!("page-bot: {:?}, bot-elem-edge: {:?}", page_bot_edge, bot_elem_top_edge);
        // // log!("px_below_bot: {:?}", px_below_bot);
        // // ==RENDER THE ELEMENT BELOW IT==
        // if px_bot_below_bot < ADD_DISTANCE {
        //     log!("ADD");
        //     if let Some(next_node) = get_next_block_node(&new_bot_node.get().hash, page_data) {
        //         // new_bot_node = next_node;
        //     } else {
        //         // if already last, we are done
        //         bot_done = true;
        //     }
        // // ==REMOVE THE BOT ELEMENT==
        // } else if px_top_below_bot > REMOVE_DISTANCE {
        //     log!("REMOVE");
        //     if let Some(prev_node) = get_prev_block_node(&new_bot_node.get().hash, page_data) {
        //         // // remove current top node
        //         // new_bot_node.get().elem_ref.unwrap().remove();
        //         // // set prev node to top elem
        //         // new_bot_node = prev_node;
        //     } else {
        //         // if already first, we are done
        //         bot_done = true;
        //     }
        // } else {
        //     bot_done = true;
        // }
    }
    let new_top_hash = new_top_node.get().hash;
    if new_top_hash != top_elem_data.get().hash {
        top_elem_data.update(|e| {
            e.hash = new_top_hash;
            e.pad = new_top_pad;
            e.node_sig = new_top_node;
        });
    }
    let new_bot_hash = new_bot_node.get().hash;
    if new_bot_hash != bot_elem_data.get().hash {
        bot_elem_data.update(|e| {
            e.hash = new_bot_hash;
            e.pad = new_bot_pad;
            e.node_sig = new_bot_node;
        })
    }
}

fn insert_new_node_before(cx: Scope, page_elem: &Element,
new_node: RwSignal<PageNode>) -> u32 {
    // 1. INSERT THE PREV NODE ALONG WITH ALL ITS PARENTS IF THEY DON'T EXIST
    // 2. ADD DOM_REF TO THE NODE
    // 3. CALCULATE THE HEIGHT ITS USING (INCLUDING ANY PARENT NODES, E.G. 
    // BLOCK PADDING) AND, THEN UPDATE THE PADDING DIV BY REMOVING THE HEIGHT
    let mut child_node = new_node;
    let mut total_height = 0;
    let mut new_child_elem: Element = create_elem(child_node); // only using vec so ik when no elem present
    child_node.update(|n| {
        n.elem_ref = Some(new_child_elem.clone()) // cloning refers to the same element since its just a ref (i have confirmed this)
    });
    // if this is the last child, we must add its parent
    // if the parent is the last child of its parent, we must add its parent, etc
    loop {
        if let Some(parent_node) = child_node.get().parent {
            let children = parent_node.get().children;
            let is_last_child = {
                let num_children = children.len();
                let last_child = children[num_children - 1].clone();
                child_node.get().hash == last_child.get().hash
            };
            if is_last_child {
                // if child is last child, it means we have to add the parent node too
                let castrated_parent_elem = {
                    let mut parent = parent_node.get();
                    parent.children = Vec::new();
                    let node_sig = create_rw_signal(cx, parent);
                    create_elem(node_sig)
                };
                castrated_parent_elem.append_child(&new_child_elem).unwrap();
                // need to update the ref of the parent node bc while 
                // `create_elem()` updates all `.elem_ref` of each `PageNode`, 
                // we passed it the castrated_parent rather than the actual 
                // parent, so the parent node itself didnt' get updated
                parent_node.update(|n| {
                    n.elem_ref = Some(castrated_parent_elem.clone()) // cloning refers to the same element since its just a ref (i have confirmed this)
                });
                new_child_elem = castrated_parent_elem;
                child_node = parent_node;
            } else {
                for i in 0..children.len() {
                    // FIRST CHILD W/ .ELEM_REF = SOME() WILL BE THE child_node 
                    // ITSELF BC IF ITS THE FIRST child_node WE SET THE 
                    // .ELEM_REF BEFORE THE LOOP, AND IF IT'S A PARENT, WE SET 
                    // THE .ELEM_REF WHEN WE CHANGE child_node TO THE PARENT
                    if let Some(_) = children[i].get().elem_ref {
                        // get the first rendered (i.e. rendered to DOM) child, 
                        // and insert new elem before it
                        let first_rendered_child = children[i+1].get().elem_ref.unwrap();
                        // no need to update refs bc they have already been 
                        // update either before the loop or in prior loops
                        let parent_elem = parent_node.get().elem_ref.unwrap();
                        parent_elem.insert_before(
                            &new_child_elem, 
                            Some(&first_rendered_child)).unwrap();
                        total_height = new_child_elem.get_bounding_client_rect().height() as u32;
                        break;
                    }
                }
                break;
            }
        } else {
            // if no parent, we add from the page_elem
            let elem_to_add_before = page_elem.first_element_child().unwrap().next_element_sibling().unwrap();
            page_elem.insert_before(
                &new_child_elem, 
                Some(&elem_to_add_before)).unwrap();
            total_height = new_child_elem.get_bounding_client_rect().height() as u32;
            break;
        }
    }
    total_height
}


/// returns the total height of the nodes removed
fn remove_top_elem_from_dom(elem: RwSignal<PageNode>) -> u32 {
    // 1. REMOVE ALL ITS PARENTS TOO IF IT'S THE ONLY CHILD CURRENTLY
    // 2. CALCULATE THE HEIGHT ITS USING AND, THEN UPDATE THE PADDING DIV
    // 3. REMOVE DOM_REF FROM THE NODE(S)
    let mut child_node = elem;
    let mut child_elem_ref = child_node.get().elem_ref.unwrap();
    let mut total_height = child_node.get().height - child_node.get().kind.innate_height();
    loop {
        total_height += child_node.get().kind.innate_height();
        child_node.update(|n| n.elem_ref = None);
        // log!("REF::: {:?}", child_node.get().elem_ref);
        // if there is a parent node, we need to check if 
        // `child_node` is the last node, bc if it is, it means 
        // we need to remove the parent node too. then we check 
        // the parent node of the parent node, etc
        if let Some(parent_node) = child_node.get().parent {
            let children = parent_node.get().children;
            let num_children = children.len();
            let last_child = children[num_children - 1].clone();
            if child_node.get().hash == last_child.get().hash {
                // if the child is the last child, it means we 
                // have to remove the parent node too
                child_node = parent_node;
                child_elem_ref = child_node.get().elem_ref.unwrap();
                continue;
            // if not the last child, we can now remove the node(s)
            // (only need to remove the root node)
            } else { }
        // if no parent (we're at a root node), just rm node bc 
        // no parents to check
        } else { }
        child_elem_ref.remove();
        break;
    }
    // let mut temp = child_node;
    // log!("PARENT: {:?}", temp.get().elem_ref);
    // for child in temp.get().children {
    //     log!("CHILD: {:?}", child.get().elem_ref);
    // }

    total_height
}

pub fn get_top_block_node(nodes: &Vec<RwSignal<PageNode>>) -> RwSignal<PageNode> {
    let mut top_node = nodes[0];
    loop {
        if let Some(first_child) = top_node.get().children.get(0) {
                // if children are not blocks, this is the final block node
                if !first_child.get().kind.is_block() { break }
                top_node = *first_child;
        } else {
            break;
        }
    }
    top_node
}
pub fn get_bot_block_node(nodes: &Vec<RwSignal<PageNode>>) -> RwSignal<PageNode> {
    let mut last_idx = nodes.len() - 1;
    let mut bot_node = nodes[last_idx];
    loop {
        let children = bot_node.get().children;
        if children.is_empty() { break }
        // if children are not blocks, this is the final block node
        last_idx = children.len() - 1;
        let last_child = children[last_idx];
        if !last_child.get().kind.is_block() { break }
        bot_node = last_child;
    }
    bot_node
}
pub fn get_top_hash(nodes: &Vec<RwSignal<PageNode>>) -> String {
    get_top_block_node(nodes).get().hash
}
pub fn get_bot_hash(nodes: &Vec<RwSignal<PageNode>>) -> String {
    get_bot_block_node(nodes).get().hash
}

pub fn get_nodes_in_view(cx: Scope, nodes: Vec<RwSignal<PageNode>>, 
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

            let children = node.get().children;
            if !children.is_empty() {
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
                    cx, children.clone(), &top_loc, &bot_loc
                );
                // if this is not sliced (e.g. if top elem and all the 
                // children slice at 0), we check if it matches the 
                // children_signal so we don't have to create a new signal 
                // if we don't have to. if matches, its not actually 
                // sliced, so keep same signal

                // same ->  not a slice
                if child_nodes == children {
                    vec.push(*node);
                // different ->  a slice
                } else {
                    // create a new node with different children instead of 
                    // appending the current node with same children
                    let mut node = node.get();
                    node.children = child_nodes;
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

pub fn get_next_block_node(hash: &String, page_data: RwSignal<Page>
) -> Option<RwSignal<PageNode>> {
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
        if let Some(_) = get_node_from_location(&location, &nodes) {
            // get top nested child if exists of prev elem, bc that is the 
            // true next item
            // location -> NODE
            let nodes = vec![
                get_node_from_location(&location, &nodes).unwrap() // unwrap bc we know it exists
            ];
            let next_node = get_top_block_node(&nodes);
            return Some(next_node);
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

pub fn get_prev_block_node(hash: &String, page_data: RwSignal<Page>
) -> Option<RwSignal<PageNode>> {
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
            let prev_node = get_bot_block_node(&nodes);
            return Some(prev_node)
        }

        // jump one level down
        location.pop().unwrap();
        // if this is true, the input hash is already the first element
        if location.len() == 0 { console_log("ALREADY FIRST"); return None }
    }
}

fn get_node_from_location(location: &Vec<usize>, nodes: &Vec<RwSignal<PageNode>>
) -> Option<RwSignal<PageNode>> {
    let mut node = match nodes.get(location[0]) {
        Some(node) => node.clone(),
        None => return None,
    };
    for idx in &location[1..] {
        let children = node.get().children;
        match children.get(*idx) {
            Some(child) => {
                node = child.clone();
            },
            None => return None,
        }
    }
    Some(node)
}

fn get_hash_from_location(location: &Vec<usize>, nodes: &Vec<RwSignal<PageNode>>
) -> Option<String> {
    match get_node_from_location(location, nodes) {
        Some(node) => Some(node.get().hash),
        None => None,
    }
}

pub fn get_node_from_hash(hash: &String, page_data: RwSignal<Page>
) -> Option<RwSignal<PageNode>> {
    let locations = page_data.get().locations.get();
    get_node_from_location(
        locations.get(hash).unwrap(), 
        &page_data.get().nodes.get())
}
