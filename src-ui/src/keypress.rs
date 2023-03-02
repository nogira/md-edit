use std::{collections::HashMap, mem};
use leptos::{log, Scope, RwSignal, document, JsCast, UntrackedGettableSignal, 
    UntrackedSettableSignal};
use web_sys::{CharacterData, Range, Selection, Node};

use super::{Page, PageNode, PageNodeType, HashToNode, IsFirstChild, IsLastChild, 
    IsBlock, PrevChild, ChangeBlockKind, InsertNodes, RemoveChild, NextChild,
    RemoveThisBlockShell, InsertChar,RemoveChar, NextSibling, rand_utf8_hash, 
    get_prev_block_node, update_hash_locations};

pub enum Key {
    Shift, Tab, ArrowUp, ArrowDown, ArrowLeft, ArrowRight,
    Return, Delete, Space, // ForwardSlash, Three,
}
impl Key {
    fn key_code(&self) -> u32 {
        match self {
            Key::Shift => 16,
            Key::Tab => 9,
            Key::ArrowUp => 38,
            Key::ArrowDown => 40,
            Key::ArrowLeft => 37,
            Key::ArrowRight => 39,

            Key::Return => 13,
            Key::Delete => 8,
            Key::Space => 32,
            // Key::ForwardSlash => 191,
            // Key::Three => 51,
        }
    }
}

pub fn new_cursor_position(selection: &Selection, node: &Node, idx: u32) {
    selection.remove_all_ranges().unwrap();
    let range = Range::new().unwrap();
    range.set_start(node, idx).unwrap();
    selection.add_range(&range).unwrap();
}

pub fn process_keypress(cx: Scope, event: web_sys::KeyboardEvent, page_data: RwSignal<Page>) {
    let key = event.key();
    let key_code = event.key_code();
    log!("KEY: {:?}", key);
    log!("KEYCODE: {:?}", key_code);
    // TODO: i need to start thinking about recording edit events to enable undo
    // check how other programs handle undo

    // TODO: ALL I NEED TO HANDLE IS DELETE RETURN AND /
    // EVERYTHING ELSE IS ALREADY FINE
    // FIXME: WAIT BUT DATA NEEDS TO UPDATE AS WELL AS THE DOM SO ALL MUST BE UPDATED THROUGH THIS FUNCTION

    let selection = document().get_selection().unwrap().unwrap();

    // first check selection type bc if no selection, and not a special 
    // char, we don't need to do anything. if there is a selection though 
    // be need to handle the deletion
    let sel_type = selection.type_(); // "Range" or "Caret" (caret is 0 range)

    // SELECTION NODES ARE TEXT NODES
    let start_node: CharacterData = selection.anchor_node().unwrap().dyn_into().unwrap();
    log!("START NODE TEXT: {}", start_node.data());
    let start_offset = selection.anchor_offset();
    // parent span
    let start_span_elem = start_node.parent_element().unwrap();
    let hash = start_span_elem.get_attribute("hash").unwrap();
    log!("START NODE SPAN HASH: {}", hash);
    let start_span_node = page_data.hash_to_node(&hash).unwrap();

    // // DEBUG: is this element the same as the one stored?
    // let __span_node_elem = page_data.hash_to_node(&hash).unwrap().get_untracked().elem_ref.unwrap();
    // let __span_is_same = &start_span_elem == &__span_node_elem;
    // log!("SPAN ELEM IS SAME AS IN PAGE_DATA? {}", __span_is_same);
    // let __tb_elem = start_span_elem.parent_element().unwrap();
    // let __tb_hash = __tb_elem.get_attribute("hash").unwrap();
    // let __tb_node_elem = page_data.hash_to_node(&__tb_hash).unwrap().get_untracked().elem_ref.unwrap();
    // let __tb_is_same = &__tb_elem == &__tb_node_elem;
    // log!(" TEXT BLOCK ELEM IS SAME AS IN PAGE_DATA? {}", __tb_is_same);

    // FIXME: only need to prevent default if the delete if the first 
    // char in a block

    // TODO: given how hard it seems like it will be to merge and them 
    // unmerge  nodes for undo/redo history. a simple solution, at leasst 
    // for now, it to just make a copy of the entire range of all nodes 
    // changing at that instance

    if sel_type == "Caret"  {
        // this triggers both when pressed by itself and when another key is 
        // pressed while this is held-down
        if event.get_modifier_state("Meta") {
            return;
            // // # key pressed (cmd-3)
            // if event.key_code() == Key::Three.key_code() {
            //     log!("#")
            // }
        }
        // DELETE key pressed
        if key_code == Key::Delete.key_code() {
            if start_offset == 0 {
                let mut child_span_sig = start_span_node;
                loop {
                    let parent_sig = child_span_sig.get().parent.unwrap();
                    if !parent_sig.is_first_child(&child_span_sig) { break } // not start of line
                    if !parent_sig.is_block() {
                        child_span_sig = parent_sig;
                        continue;
                    }
                    // 1) get the first parent block
                    let parent = parent_sig.get();
                    // 2) to remove the current inner-most block type we 
                    // either a) if not a TextBlock convert it to a TextBlock, 
                    // as these blocks should always be leaf blocks so don't 
                    // need to worry about affecting any sibling blocks by 
                    // converting the parent block, or b) if a TextBlock, 
                    // remove the parent block from this block (just from this 
                    // block, not from any siblings if any), however if the 
                    // parent block is the Page block, we obviously don't want 
                    // to remove that, instead we check if there is a block 
                    // above to see if we need to append all the spans from 
                    // the current TextBlock to the block above
                    if parent.kind == PageNodeType::TextBlock {
                        let textblock_sig = parent_sig.clone();
                        let block_around_textblock_sig = parent.parent.unwrap();

                        if block_around_textblock_sig.is_first_child(&textblock_sig) {
                            // if is the first block child and parent block is 
                            // not a Page, remove the parent block and return
                            if block_around_textblock_sig.get().kind != PageNodeType::Page {
                                // log!("IS INDENT OR QUOTE");
                                block_around_textblock_sig.remove_this_block_shell();
                                update_hash_locations(&page_data);
                                new_cursor_position(&selection, &start_node, 0);
                            }
                            // if parent block is Page and there is no previous 
                            // block, return with no changes
                            return;
                        }

                        let block_around_textblock = block_around_textblock_sig.get_untracked();
                        if block_around_textblock.kind != PageNodeType::Page {
                            // remove the text block, and any blocks after 
                            // it, from this outer block, encasing the 
                            // blocks after it in a new block of the same type
                            let insertion_parent_sig = block_around_textblock.parent.unwrap();
                            let insertion_sig = block_around_textblock_sig.next_sibling();
                            // FIXME: SPAN ELEMENT IS BEING CHANGED CAUSING A BUG
                            block_around_textblock_sig.update_untracked(|block_around_textblock| {
                                let after_block_sig = PageNode::signal_from(
                                    // FIXME: THIS HASH MIGHT NOT BE UNIQUE
                                    cx, rand_utf8_hash(), block_around_textblock.kind.clone(), 
                                    HashMap::new(), vec![], None, 0);
                                let mut after = false;
                                let mut first_after = false;
                                let children = mem::take(
                                    &mut block_around_textblock.children);
                                for child in children {
                                    if child == textblock_sig {
                                        insertion_parent_sig.insert_nodes(&vec![child], (&insertion_sig).as_ref());
                                        after = true;
                                        first_after = true;
                                        continue
                                    }
                                    if after == true {
                                        // must insert the `after_block_sig` node pre-moving the child nodes so the child nodes are carried across
                                        if first_after == true {
                                            insertion_parent_sig.insert_nodes(
                                                &vec![after_block_sig], 
                                                (&insertion_sig).as_ref());
                                            first_after = false;
                                        }
                                        after_block_sig.insert_nodes(&vec![child], None);
                                        continue
                                    }
                                    // keep child in the block_around_textblock
                                    block_around_textblock.children.push(child);
                                    
                                }
                            });
                            // block_around_textblock_sig.insert()
                            update_hash_locations(&page_data);

                            // log!("{}", page_data.get_untracked().debug_nodes());

                            // new_cursor_position(&selection, &start_node, 0);
                            return

                        // parent around TextBlock is `Page` block, so we join to 
                        } else {
                            // get deepest branch block
                            // if last child is NOT a text leaf (e.g. table), 
                            // merge text into leaf block, else do nothing
                            let prev_block_sig = get_prev_block_node( // this is a leaf block
                                &textblock_sig.get().hash,
                                page_data
                            ).unwrap();
                            if prev_block_sig.get().kind == PageNodeType::Table {
                                return;
                            }
                            /*
                            prev block, last elem
                            <b>ooo<i>ooo</i></b>

                            text block, first elem
                            <b><i>ooo</i>ooo</b>

                            1. check last elem same as first elem. 
                            if true, append elems after first then 
                            check first elem of first elem, if 
                            false, append all elems
                            */
                            let mut node_1_sig = prev_block_sig; // prev block
                            let mut node_2_sig = textblock_sig; // current text block
                            'inner: loop {
                                let node_1 = node_1_sig.get_untracked();
                                let mut node_2 = node_2_sig.get_untracked();

                                let last_child_sig = node_1.children.last().unwrap().clone();
                                let first_child_sig = node_2.children.remove(0);
                                let second_child_sig = node_2.children.get(1);

                                // first append children after first bc will need to regardless
                                node_1_sig.insert_nodes(&node_2.children, None);

                                let first_child = first_child_sig.get_untracked();
                                let last_child = last_child_sig.get_untracked();
                                let first_child_kind = first_child.kind;
                                let last_child_kind = last_child.kind;
                                if last_child_kind == first_child_kind {
                                    // if no children, this is a raw text 
                                    // node, so we need to append the text
                                    if first_child_kind == PageNodeType::RawText {
                                        last_child_sig.update_untracked(|n| {
                                            // merge RawText nodes
                                            let txt = n.content.get("text").unwrap();
                                            let txt_len = txt.len();
                                            let apnd = first_child.content.get("text").unwrap();
                                            let joined = format!("{}{}", txt, apnd);
                                            n.content.insert("text".into(), joined.clone()).unwrap();
                                            // append text to DOM text node
                                            let last_child_elem = n.elem_ref.as_ref().unwrap();
                                            last_child_elem.set_text_content(Some(&joined));

                                            // fix cursor position bc the 
                                            // current cursor is still referencing 
                                            // the elements pre-modification, 
                                            // meaning it will cause errors if 
                                            // an arrow key is pressed
                                            new_cursor_position(&selection, 
                                                &last_child_elem.first_child().unwrap(),
                                                txt_len as u32);
                                        });
                                        break 'inner;
                                    }
                                    // else check children
                                    node_1_sig = last_child_sig;
                                    node_2_sig = first_child_sig;
                                // if different span types, simply append
                                } else {
                                    node_1_sig.insert_nodes(&vec![first_child_sig], second_child_sig);
                                    break 'inner;
                                }
                            }
                            // FIXME: this remove might create an error if the first last child is different to the first first child ?? but
                            block_around_textblock_sig.remove_child(&textblock_sig);
                            // update hashes
                            update_hash_locations(&page_data);
                        }
                        return;
                    }
                    // if first parent block is not a TextBlock (e.g. H1),
                    // change to textblock to act as a delete
                    parent_sig.change_block_kind(PageNodeType::TextBlock);
                    return;
                }
            }
            start_span_node.remove_char(&start_node, start_offset, &selection);
            return;
        }
        // RETURN key pressed
        else if key_code == Key::Return.key_code() {
            // split the spans of the block in two, add a new block below. 
            // transfer the right spans to the new block

            let txt_len = start_node.length();
            let left_txt = start_node.substring_data(0, start_offset).unwrap();
            log!("LEFT TXT: {:?}", left_txt);
            let right_txt = start_node.substring_data(start_offset, txt_len).unwrap();
            start_span_node.update_untracked(|n| {
                n.elem_ref.clone().unwrap().set_text_content(Some(&left_txt));
                n.content.insert("text".into(), left_txt);
            });
            let right_node_content = HashMap::from([
                ("text".to_string(), right_txt)
            ]);
            let right_node: RwSignal<PageNode> = PageNode::signal_from(cx,
                rand_utf8_hash(),
                start_span_node.get_untracked().kind,
                right_node_content, 
                vec![],
                None,
                0,
            ); // create element at end so can get heights

            let mut child_span_sig = start_span_node.clone();
            let mut new_node_sig: RwSignal<PageNode> = right_node.clone();

            // loop up parents until we hit the last span (i.e. whose parent is a block)
            loop {
                let parent_sig = child_span_sig.get_untracked().parent.unwrap();
                let parent_node = parent_sig.get_untracked();

                let new_parent_sig = PageNode::signal_from(cx,
                    rand_utf8_hash(), parent_node.kind.clone(), 
                    HashMap::new(), vec![new_node_sig], 
                    None, 0);
                new_node_sig.update(|n| {
                    n.parent = Some(new_parent_sig);
                });

                if parent_node.children.len() > 1 {
                    // leave children in front of `child_span_sig`, but add 
                    // children trailing
                    parent_sig.update_untracked(|pn| {
                        let mut can_add_child = false;

                        // move/append trailing children from `parent_sig` to 
                        // `new_parent_sig`
                        pn.children.retain(|child_sig| {
                            if can_add_child {
                                new_parent_sig.insert_nodes(
                                    &vec![child_sig.clone()], None);
                                return false // rm child
                            } else {
                                if child_sig == &child_span_sig { can_add_child = true }
                                return true // keep child
                            }
                        });
                    });
                }
                child_span_sig = parent_sig;
                new_node_sig = new_parent_sig;

                // if the parent node is a block, `new_node_sig` will be the 
                // block, and we can insert this block into the view and finish
                if parent_node.is_block() {
                    let parent_parent_sig = parent_sig.get_untracked().parent.unwrap();
                    let parent_next_child = parent_parent_sig.next_child(&parent_sig);
                    parent_parent_sig.insert_nodes(&vec![new_node_sig], 
                        (&parent_next_child).as_ref());
                    new_cursor_position(&selection, 
                        &right_node.get_untracked().elem_ref.unwrap(), 0);
                    break;
                }
            }
            return;
        }
        // SPACE key pressed
        else if key_code == Key::Space.key_code() {
            let txt_node_str = start_node.text_content().unwrap();
            if start_offset == 1 && &txt_node_str[0..1] == "#" {
                let mut child_sig = start_span_node;
                loop {
                    let parent_sig = child_sig.get().parent.unwrap();
                    if !parent_sig.is_first_child(&child_sig) { break } // is not at start of line
                    if !parent_sig.is_block() {
                        child_sig = parent_sig;
                        continue;
                    }
                    let parent = parent_sig.get();
                    if parent.kind != PageNodeType::TextBlock { break }
                    parent_sig.change_block_kind(PageNodeType::H1);
                    start_span_node.update_untracked(|e| {
                        e.content.insert("txt".into(), (&txt_node_str[1..]).clone().into());
                    });
                    new_cursor_position(&selection, &start_node, 0);
                    start_node.delete_data(0, 1).unwrap();
                    return;
                }
            } else if start_offset == 1 && &txt_node_str[0..1] == "-" {
                let mut child_sig = start_span_node;
                loop {
                    let parent_sig = child_sig.get().parent.unwrap();
                    if !parent_sig.is_first_child(&child_sig) { break }
                    if !parent_sig.is_block() {
                        child_sig = parent_sig;
                        continue;
                    }
                    let parent = parent_sig.get();
                    if parent.kind != PageNodeType::TextBlock { break }
                    parent_sig.change_block_kind(PageNodeType::Dot);
                    start_span_node.update_untracked(|e| {
                        e.content.insert("txt".into(), (&txt_node_str[1..]).clone().into());
                    });
                    start_node.delete_data(0, 1).unwrap();
                    return;
                }
            } else if start_offset == 1 && &txt_node_str[0..1] == ">" {
                let mut child_sig = start_span_node;
                loop {
                    let parent_sig = child_sig.get().parent.unwrap();
                    if !parent_sig.is_first_child(&child_sig) { break }
                    if !parent_sig.is_block() {
                        child_sig = parent_sig;
                        continue;
                    }
                    let parent = parent_sig.get();
                    if parent.kind != PageNodeType::TextBlock { break }
                    parent_sig.change_block_kind(PageNodeType::Quote);
                    
                    start_span_node.update_untracked(|e| {
                        let new_text = (&txt_node_str[1..]).clone();
                        log!("new_text: {}", new_text);
                        e.content.insert("txt".into(), new_text.into());
                    });
                    start_node.delete_data(0, 1).unwrap();
                    return;
                }
            }
        }
        else if key_code == Key::Shift.key_code()
        || key_code == Key::Tab.key_code()
        || key_code == 93 { // Meta/cmd
            // do nothing
            return;
        }
        else if key_code == Key::ArrowUp.key_code() {
            selection.modify("move", "backward", "line").unwrap();
            return;
        } else if  key_code == Key::ArrowDown.key_code() {
            selection.modify("move", "forward", "line").unwrap();
            return;
        } else if  key_code == Key::ArrowLeft.key_code() {
            selection.modify("move", "backward", "character").unwrap();
            return;
        } else if  key_code == Key::ArrowRight.key_code() {
            selection.modify("move", "forward", "character").unwrap();
            return;
        }

        // insert char
        start_span_node.insert_char(&key, &start_node, start_offset, &selection);
    }

    // let keys = [
    //     Key::Return.key_code(),
    //     Key::Delete.key_code(),
    //     Key::ForwardSlash.key_code(),
    // ];

    // if &sel_type == "Range" || keys.contains(&key_code) {
    //     let nodes = page_data.get();

        // FIXME: shit maybe i need hashing after-all w/ the hashes also 
        // stored in a signal along with their paths. looks like it will 
        // be very expensive to find the position of each child w/o it (if even possible)



        // if &sel_type == "Range" {

        // } else {

        // }

        // // RETURN key pressed
        // if event.key_code() == 13 {
        //     event.prevent_default();
        //     let parent = selection.anchor_node().unwrap().parent_element().unwrap();
        //     console_log(&format!("...: {:?}", parent));
        //     // console_log(&format!("{:?}", selection.anchor_node().unwrap().parent_element().unwrap().get_attribute("block").unwrap()));

        // // DELETE key pressed
        // } else if event.key_code() == 8 {

        //     event.prevent_default();


        //     // TODO: add capital letters to hash 

        //     let len = start_node.to_string().length();
        //     log!("SEL TYPE: {:?}", len);
            
        //     let kind = start_span_elem.get_attribute("type").unwrap();
        //     log!("TYPE: {:?}", kind);
        //     // step 1, check if block or span, bc need to get to parent block
        //     // if selection is at start of block, 
        //     match kind.as_str() {
        //         // span
        //         "span" => {},
        //         // block
        //         _ => {},
        //     }
        // }
    // }
}
