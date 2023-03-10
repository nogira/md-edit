use leptos::{log, Scope, RwSignal, create_rw_signal, UntrackedGettableSignal, UntrackedSettableSignal};
use web_sys::Element;

use super::{Page, PageNode, CreateElem, HashToLocation, IsFirstChild, IsLastChild, NextChild};

trait GetPageElem {
    fn get_page_elem(&self) -> Element;
}
impl GetPageElem for Element {
    /// get page elem (scroll window)
    fn get_page_elem(&self) -> Element {
        let mut elem = self.parent_element().unwrap();
        loop {
            if elem.get_attribute("type").unwrap() == "scroll-window" {
                return elem;
            }
            elem = elem.parent_element().unwrap();
        }
    }
}
pub trait ElemIsInView {
    fn elem_is_in_view(&self) -> bool;
}
impl ElemIsInView for Element {
    /// check this element is in view (should be rendered)
    fn elem_is_in_view(&self) -> bool {
        // get page elem
        let page_elem = self.get_page_elem();
        let page_top_edge = page_elem.get_bounding_client_rect().top();
        let elem_bot_edge = self.get_bounding_client_rect().bottom();
        let px_bot_above_top = page_top_edge - elem_bot_edge;
        if px_bot_above_top > REMOVE_DISTANCE {
            return false;
        }
        let page_bot_edge = page_elem.get_bounding_client_rect().bottom();
        let elem_top_edge = self.get_bounding_client_rect().top();
        let px_top_below_bot = elem_top_edge - page_bot_edge;
        if px_top_below_bot > REMOVE_DISTANCE {
            return false;
        }
        return true;
    }
}

const REMOVE_DISTANCE: f64 = 50.0;
const ADD_DISTANCE: f64 = 20.0;

pub fn update_dom_nodes_in_view(cx: Scope, page_data: RwSignal<Page>, page_elem: &Element) {

    let page_top_edge = page_elem.get_bounding_client_rect().top();
    let page_bot_edge = page_elem.get_bounding_client_rect().bottom();

    let top_elem_data = page_data.get().top_elem;
    let bot_elem_data = page_data.get().bot_elem;

    let mut top_done = false;
    let mut bot_done = false;

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

    ???   ??????   ???
    ??????????????????????????? --------       <- remove if bottom of top node is above
    ???????????????????????????
    ???   ??????   ???
    ???   ??????   ???
    ???   ??????   ??? --------       <- add if bottom of top node is below
    ???   ??????   ???
    ???????????????????????????

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
        if px_top_above_top < ADD_DISTANCE {
            // NOTE: THIS TRIGGERS AT TOP OF PAGE BC TOP NODE IS STILL UNDER 
            // THE TOP OF PAGE (but obv not the if statement below)
            if let Some(prev_node) = get_prev_block_node(&new_top_node.get().hash, page_data) {
                // add node to page
                let height = insert_new_node_before(cx, prev_node);
                // rm previously added padding
                // if total is negative (should never be) it is converted to 0 w/ `as u32`
                new_top_pad = (new_top_pad as f64 - height as f64) as u32;
                new_top_node = prev_node;
                update_top_padding(page_elem, new_top_pad);
            // if already first, we are done
            } else { top_done = true }
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
                update_top_padding(page_elem, new_top_pad);
            // if already last, we are done
            } else { top_done = true }
        } else { top_done = true }
        }
        let new_bot_elem = new_bot_node.get().elem_ref.unwrap();
        let bot_elem_top_edge = new_bot_elem.get_bounding_client_rect().top();
        let bot_elem_bot_edge = new_bot_elem.get_bounding_client_rect().bottom();
        let px_top_below_bot = bot_elem_top_edge - page_bot_edge;
        let px_bot_below_bot = bot_elem_bot_edge - page_bot_edge;
        // ==RENDER THE ELEMENT BELOW IT==
        if px_bot_below_bot < ADD_DISTANCE {
            if let Some(next_node) = get_next_block_node(&new_bot_node.get().hash, page_data) {
                // add node to page
                let height = insert_new_node_after(cx, next_node);
                // rm previously added padding
                // if total is negative (should never be) it is converted to 0 w/ `as u32`
                new_bot_pad = (new_bot_pad as f64 - height as f64) as u32;
                new_bot_node = next_node;
                update_bot_padding(page_elem, new_bot_pad);
            // if already last, we are done
            } else { bot_done = true }
        // ==REMOVE THE BOT ELEMENT==
        } else if px_top_below_bot > REMOVE_DISTANCE {
            if let Some(prev_node) = get_prev_block_node(&new_bot_node.get().hash, page_data) {
                // remove current bot node
                let height = remove_bot_elem_from_dom(new_bot_node);
                // set prev node to bot elem
                new_bot_pad = new_bot_pad + height;
                new_bot_node = prev_node;
                update_bot_padding(page_elem, new_bot_pad);
            // if already first, we are done
            } else { bot_done = true }
        } else { bot_done = true }
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

pub fn update_top_padding(page_elem: &Element, pad: u32) {
    page_elem.first_element_child().unwrap().set_attribute(
        "style", &format!("height: {}px", pad)).unwrap();
}
pub fn update_bot_padding(page_elem: &Element, pad: u32) {
    page_elem.last_element_child().unwrap().set_attribute(
        "style", &format!("height: {}px", pad)).unwrap();
}


// TODO: tbh it might be better to create a new implementation of insert that uses a slice of the nodes, as we'll already need something like this for e.g. copy and paste

/// 1. INSERT THE PREV NODE ALONG WITH ALL ITS PARENTS IF THEY DON'T EXIST
/// 2. ADD DOM_REF TO THE NODE(S)
/// 3. CALCULATE THE HEIGHT OF THE ELEMENT(S) BEING ADDED ITS USING AND, THEN 
/// UPDATE THE PADDING DIV BY REMOVING THE HEIGHT
fn insert_new_node_before(cx: Scope, new_node: RwSignal<PageNode>) -> u32 {
    let mut child_node = new_node;
    let mut child_elem: Element = child_node.create_elem();

    // if this is the last child, we must add its parent. if the parent is the 
    // last child of its parent, we must add its parent, etc
    loop {
        // need to update the ref of the parent node here bc while 
        // `create_elem()` updates all `.elem_ref` of each `PageNode`, we 
        // passed it the castrated_parent rather than the actual parent, so 
        // the parent node itself didnt' get updated
        child_node.update(|n| {
            n.elem_ref = Some(child_elem.clone()) // cloning refers to the same element since its just a ref (i have confirmed this)
        });
        let parent_node = child_node.get().parent.unwrap();
        if parent_node.is_last_child(&child_node) {
            // if child is last child, it means we have to add the parent node too
            let castrated_parent_elem = { // using parent elem with no children otherwise all of its children would be added to the top instead of only the top element
                let mut parent = parent_node.get();
                parent.children = Vec::new();
                create_rw_signal(cx, parent).create_elem()
            };
            castrated_parent_elem.append_child(&child_elem).unwrap();
            child_elem = castrated_parent_elem;
            child_node = parent_node;
        } else {
            // since we're inserting before the child to the right, we need the 
            // element of the next child so we know where exactly to insert
            let next_child = parent_node.next_child(&child_node).unwrap();
            let next_elem = next_child.get().elem_ref.unwrap();
            let parent_elem = parent_node.get().elem_ref.unwrap();
            parent_elem.insert_before(&child_elem, 
                Some(&next_elem)).unwrap();
            return child_elem.get_bounding_client_rect().height() as u32;
        }
    }
}

fn remove_all_children_elems(elem: &RwSignal<PageNode>) {
    let children = elem.get_untracked().children;
    for child in children {
        child.update_untracked(|n| n.elem_ref = None);
        remove_all_children_elems(&child);
    }
}

/// returns the total height of the nodes removed
fn remove_top_elem_from_dom(elem: RwSignal<PageNode>) -> u32 {
    // remove all span elements
    remove_all_children_elems(&elem);
    // 1. REMOVE ALL ITS PARENTS TOO IF IT'S THE ONLY CHILD CURRENTLY
    // 2. CALCULATE THE HEIGHT ITS USING AND, THEN UPDATE THE PADDING DIV
    // 3. REMOVE DOM_REF FROM THE NODE(S)
    let mut child_node = elem;
    let mut child_elem_ref = child_node.get().elem_ref.unwrap();
    let mut total_height = child_node.get().height - child_node.get().kind.innate_height();
    loop {
        total_height += child_node.get().kind.innate_height();
        child_node.update(|n| n.elem_ref = None);
        // if there is a parent node, we need to check if 
        // `child_node` is the last node, bc if it is, it means 
        // we need to remove the parent node too. then we check 
        // the parent node of the parent node, etc
        if let Some(parent_node) = child_node.get().parent {
            if parent_node.is_last_child(&child_node) {
                // if the child is the last child, it means we have to remove 
                // the parent node too
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
    total_height
}

fn insert_new_node_after(cx: Scope, new_node: RwSignal<PageNode>) -> u32 {
    // 1. INSERT THE NEXT NODE ALONG WITH ALL ITS PARENTS IF THEY DON'T EXIST
    // 2. ADD DOM_REF TO THE NODE
    // 3. CALCULATE THE HEIGHT ITS USING (INCLUDING ANY PARENT NODES, E.G. 
    // BLOCK PADDING) AND, THEN UPDATE THE PADDING DIV BY REMOVING THE HEIGHT
    let mut child_node = new_node;
    let mut child_elem: Element = child_node.create_elem();

    // if this is the first child, we must add its parent. if the parent is the 
    // first child of its parent, we must add its parent, etc
    loop {
        // need to update the ref of the parent node here bc while 
        // `create_elem()` updates all `.elem_ref` of each `PageNode`, we 
        // passed it the castrated_parent rather than the actual parent, so 
        // the parent node itself didnt' get updated
        child_node.update(|n| {
            n.elem_ref = Some(child_elem.clone()) // cloning refers to the same element since its just a ref (i have confirmed this)
        });
        let parent_node = child_node.get().parent.unwrap();
        if parent_node.is_first_child(&child_node) {
            // if child is first child, it means we have to add the parent node too
            let castrated_parent_elem = { // using parent elem with no children otherwise all of its children would be added to the bot instead of only the bot element
                let mut parent = parent_node.get();
                parent.children = Vec::new();
                create_rw_signal(cx, parent).create_elem()
            };
            castrated_parent_elem.append_child(&child_elem).unwrap();
            child_elem = castrated_parent_elem;
            child_node = parent_node;
        } else {
            // can just append to the parent
            let parent_elem = parent_node.get().elem_ref.unwrap();
            parent_elem.append_child(&child_elem).unwrap();
            return child_elem.get_bounding_client_rect().height() as u32;
        }
    }
}

/// returns the total height of the nodes removed
fn remove_bot_elem_from_dom(elem: RwSignal<PageNode>) -> u32 {
    // remove all span elements
    remove_all_children_elems(&elem);
    // 1. REMOVE ALL ITS PARENTS TOO IF IT'S THE ONLY CHILD CURRENTLY
    // 2. CALCULATE THE HEIGHT ITS USING AND, THEN UPDATE THE PADDING DIV
    // 3. REMOVE DOM_REF FROM THE NODE(S)
    let mut child_node = elem;
    let mut child_elem_ref = child_node.get().elem_ref.unwrap();
    let mut total_height = child_node.get().height - child_node.get().kind.innate_height();
    loop {
        total_height += child_node.get().kind.innate_height();
        child_node.update(|n| n.elem_ref = None);
        // if there is a parent node, we need to check if 
        // `child_node` is the first node, bc if it is, it means 
        // we need to remove the parent node too. then we check 
        // the parent node of the parent node, etc
        if let Some(parent_node) = child_node.get().parent {
            if parent_node.is_first_child(&child_node) {
                // if the child is the first child, it means we have to remove 
                // the parent node too
                child_node = parent_node;
                child_elem_ref = child_node.get().elem_ref.unwrap();
                continue;
            // if not the first child, we can now remove the node(s)
            // (only need to remove the root node)
            } else { }
        // if no parent (we're at a root node), just rm node bc 
        // no parents to check
        } else { }
        child_elem_ref.remove();
        break;
    }
    total_height
}

pub fn get_top_block_node(nodes: &Vec<RwSignal<PageNode>>) -> RwSignal<PageNode> {
    let mut top_node = nodes[0];
    loop {
        if let Some(first_child) = top_node.get().children.first() {
            // if children are not blocks, this is the final block node
            if !first_child.get().is_block() { break }
            top_node = *first_child;
        } else {
            break;
        }
    }
    top_node
}
pub fn get_bot_block_node(nodes: &Vec<RwSignal<PageNode>>) -> RwSignal<PageNode> {
    let mut bot_node = *nodes.last().unwrap();
    loop {
        let children = bot_node.get().children;
        if children.is_empty() { break }
        // if children are not blocks, this is the final block node
        let last_child = children.last().unwrap();
        if !last_child.get().is_block() { break }
        bot_node = *last_child;
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
    let start_idx = top_loc.first();
    let end_idx = bot_loc.first();
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
    let mut location = page_data.hash_to_location(hash);
    // console_log(&format!("location: {:?}", location));
    let nodes = page_data.get().nodes.get().children;
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
        if location.len() == 0 { return None }
    }
}

// TODO: INSTEAD OF THESE ALGOS TO FIND NEXT ITEM AND SUCH, I COULD JUST UPDATE 
// THE NODES DATAT STURTCURE TO STORE PARENTS, SIBLINGS, ETC IN EACH NODE. I'D 
// HAVE TO CONFIRM THE RWSIGNAL DATATYPE IS VERY SMALL THOUGH TO JUSTIFY ADDING 
// THEIR REFERENCES TO EACH NODE

pub fn get_prev_block_node(hash: &String, page_data: RwSignal<Page>
) -> Option<RwSignal<PageNode>> {
    let mut location = page_data.hash_to_location(hash);
    // log!("location: {:?}", location);
    let nodes = page_data.get().nodes.get().children;
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
        if location.len() == 0 { return None }
    }
}

pub fn get_node_from_location(location: &Vec<usize>, nodes: &Vec<RwSignal<PageNode>>
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
    get_node_from_location(
        &(page_data.hash_to_location(hash)),
        &page_data.get().nodes.get().children)
}
