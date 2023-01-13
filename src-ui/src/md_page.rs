use core::{cmp::max, fmt::Debug};

use leptos::*;
// use src_ui::*;

use super::text_conversion::{text_to_imd, MDBlock};

#[derive(Debug, Clone, PartialEq)]
enum MDNodeType {
    Block, Span, Text
}
#[derive(Debug, Clone, PartialEq)]
enum HideableMDType {
    None, H1, Bold, Italic
}
type MDNodeInfo = (MDNodeType, usize, HideableMDType);

#[component]
pub fn MarkdownPage(cx: Scope, name: String) -> Element {

    let (
        top_line_num,
        set_top_line_num
    ) = create_signal::<u32>(cx, 0);

    // e.g. vec![(Block, 2, None), (Block, 1, H1), (Span, 4, Bold), (Text, 3, None)]
    // meaning 3rd block, 2nd sub-block, 5th span elem, position 3 in text node
    let (
        selection_start,
        set_selection_start
    ) = create_signal::<Vec<MDNodeInfo>>(cx, Vec::new());

    create_effect(cx, |_| {console_log("effect")});

    console_log("RELOAD");

    let text = "#\n# hello\n- some point\n> this **is** cool\n> 🤡 ᾎ ᾯ y̆".to_string();
    let mut blocks: Vec<Element> = Vec::new();

    // text intermediary between raw text and html
    let (
        text_imd,
        set_text_imd
    ) = create_signal::<Vec<MDBlock>>(cx, Vec::new());

    // TODO: SWAP FROM BLOCK TYPE IN CLASS TO BLOCK TYPE IN type=""

    

    create_effect(cx, move |_| {
        console_log("setting text_imd");
        let imd = text_to_imd(&text);
        console_log(&format!("{:?}", imd));
        set_text_imd.update(|v| *v = imd);
    });

    /// text_idx is so we know where to start the text parsing from (return when hit \n)
    fn parse_md(cx: Scope, tag: &str, lvl_num: usize, text_idx: usize, text: &str) -> Vec<Element> {
        let mut block_num = 0;
        let mut blocks: Vec<Element> = Vec::new();

        let mut char_1_back = '\n';
        let mut idx_1_char_back: usize = 0;
        let mut char_2_back = '\n';
        let mut idx_2_char_back: usize = 0;
        // let mut iter = 0..(text.len() - 1);
        let mut iter = text.char_indices();
        while let Some((i, char)) = iter.next()  {
            // let char = &text[i..i+1];
            // console_log(&format!("{char}"));
            // console_log(&format!("{i}: {char}"));

            let next_char = match iter.clone().next() {
                Some((_, next_char)) => next_char,
                None => continue,
            };

            console_log(&format!("prev: {char_2_back} prev: {char_1_back} now: {char} next: {next_char}"));

            match char {
                '#' => {
                    let elem = view! {cx,
                        <div class="h1" block=block_num>
                        <md hidden>"# "</md>"hello"
                        </div>
                    };
                    blocks.push(elem);
                    block_num += 1;
                },
                '*' => {
                    console_log("here");
                    // ITALIC
                    if char_1_back == ' ' && next_char != '*' {
                        // check if it closes before a line-break, if not, ignore
                        console_log("italic");

                        // let elem = parse_md(cx, "i", line, i, text);
                        // blocks.push(elem)
                    // BOLD
                    } else if char_1_back == '*' && char_2_back == ' ' {
                        console_log("bold");
                        let mut prev_char = ' ';
                        let mut iter = (&text[i..]).char_indices();
                        while let Some((j, char)) = iter.next()  {
                            match char {
                                '*' => {
                                    if prev_char != ' ' {
                                        if let Some((_, next_char)) = iter.clone().next() {
                                            if next_char == '*' {
                                                console_log(&text[i+1..i+j])
                                            }
                                        }
                                    }
                                },
                                _ => {}
                            }
                            prev_char = char;
                        }
                        // let elem = parse_md(cx, "b", line, i, text);
                        // blocks.push(elem)
                    }
                }
                _ => {}
            }
            char_2_back = char_1_back;
            char_1_back = char;
        }

        // let parent = create_element(tag);
        // match tag {
        //     "div" | "h1" => parent.set_attribute("class", "block").unwrap(),
        //     _ => {},
        // }
        // view! {cx,
        //     <div class="block">
        //         {blocks}
        //     </div>
        // }

        blocks
    }

    // let div_e = create_element("div");
    // div_e.set_attribute("line", "1").unwrap();
    // div_e.set_inner_html("<b><span>**</span>xd<span>**</span></b>");

    // blocks.push(div_e);

    // let div_e = create_element("div");
    // div_e.set_attribute("line", "2").unwrap();
    // div_e.set_inner_html("<b><span>**</span>xd<span>**</span></b>");

    // blocks.push(div_e);

    // let div_s = create_element("span");
    // div_s.set_inner_html("**");
    // div_s.set_attribute("hidden", "true").unwrap();
    // // div_s.add_event_listener_with_callback(type_, listener);

    // let div_e = view! {cx, <div line="3">"more "<b>"hello"{div_s}</b>" text"</div>};
    // // div_e.set_inner_html("<b><span>**</span>xd<span>**</span></b>");

    fn handle_keypress(event: web_sys::KeyboardEvent) {
        console_log(&format!("KEY: {:?}", event.key_code()));

        // if return key pressed. 
        if event.key_code() == 13 {
            console_log("return");
            event.prevent_default();
            let selection = document().get_selection().unwrap().unwrap();
            // console_log(&format!("{:?}", selection.anchor_node().unwrap().parent_element().unwrap().get_attribute("block").unwrap()));
        // if delete key pressed
        } else if event.key_code() == 8 {
            // let selection = document().get_selection().unwrap().unwrap();
            // let parent = selection.anchor_node().unwrap().parent_element().unwrap();
            // if parent.tag_name() == "MD" {
            //     let md_elem = parent.parent_element().unwrap();
            // }
            // let full_page = 
            console_log(&format!("KEY: {:?}", event.key_code()));
        }
    }

    let out = view! {cx,
        // scroll view
        <div id="md-page" contenteditable on:scroll=|_| console_log("test") on:keydown=handle_keypress>
            {parse_md(cx, "div", 0, 0, &text)}
            // {blocks}
            // // <For each=lines_vec key=|e| e.num>
            // //     {|cx: Scope, e: &GenericEventRes| {
            // //         // view! {
            // //         //     cx,
            // //         //     <{move {e.elem}} />
            // //         // }
            // //         (e.elem)(cx)
            // //     }}
            // // </For>
            // "hello"
            // // <div html="<b>xd</b>" />.set_inner_html()
            // {div_e}
        </div>
    };
    use crate::wasm_bindgen::closure::Closure;


    // let closure = Closure::<dyn FnMut(_)>::new(move |event: web_sys::Event| {
    //     event.prevent_default()

    //     // find the line clicked on, then run ".innerText" on that div so can update my vec on line strings
    // });
    // document().add_event_listener_with_callback("", closure.as_ref().unchecked_ref()).unwrap();
    // closure.forget();

    let closure = Closure::<dyn FnMut(_)>::new(move |event: web_sys::Event| {
        // console_log("PRESELECTION");
        let selection = document().get_selection().unwrap().unwrap();
        // console_log(&format!("selection: {:?}", selection.to_string()));
        // console_log("POSTSELECTION");
        // console_log(&format!("{:?}", selection.anchor_node()));
        // console_log(&format!("{:?}", selection.anchor_offset()));
        // console_log(&format!("{:?}", selection.to_string()));

        let mut new_selection_start = Vec::new();

        // if there is no selection, this is where we catch it (rather than 
        // `let selection =`). we still want to run it to deselect prev
        if let Some(anchor) = selection.anchor_node() {

            new_selection_start.push((
                MDNodeType::Text,
                selection.anchor_offset() as usize,
                HideableMDType::None));

            // PARENT ELEM
            let parent_elem = anchor.parent_element().unwrap();
            // get hideable md type from class (if not hideable, val is just None)
            let hideable_md_type = get_hideable_md_type(&parent_elem);
            // get node type & idx
            let (node_type, idx) = get_span_or_block_info(&parent_elem);
            new_selection_start.insert(0, (node_type, idx, hideable_md_type.clone()));

            // get info of all parent elems
            loop {
                let parent_elem = parent_elem.parent_element().unwrap();
                // end if we reach the root node
                if parent_elem.id() == "md-page" { break }

                // get hideable md type from class (if not hideable, val is just None)
                let hideable_md_type = get_hideable_md_type(&parent_elem);
                // get node type & idx
                let (node_type, idx) = get_span_or_block_info(&parent_elem);
                new_selection_start.insert(0, (node_type, idx, hideable_md_type.clone()));
            }
        };

        // loop through new_selection_start and compare to selection_start to 
        // know if need to unhide or hide elems

        let prev_vec = selection_start.get();
        let prev_len = prev_vec.len();
        let curr_len = new_selection_start.len();
        let max_len = max(prev_len, curr_len);

        let root = document().query_selector("#md-page").unwrap().unwrap();
        let mut prev_elem = root.clone();
        let mut curr_elem = root;

        for i in 0..max_len {
            // NEED TO TRACK WHEN THEY DIVERGE bc e.g. if the first elem is 
            // diff, but next is same, its absolutely not the same element, 
            // but only way to know that is to know if all previous 
            // `MDNodeInfo`s are the same or not
            let diff = false;
            // if prev item present
            if let Some(prev_item) = prev_vec.get(i) {
                if prev_item.0 == MDNodeType::Text { continue }
                // get nth child according to prev_item
                prev_elem = prev_elem.first_element_child().unwrap();
                for _ in 0..prev_item.1 {
                    prev_elem = prev_elem.next_element_sibling().unwrap();
                }
                if let Some(curr_item) = new_selection_start.get(i) {
                    if curr_item.0 == MDNodeType::Text { continue }
                    // get nth child according to curr_item
                    curr_elem = curr_elem.first_element_child().unwrap();
                    for _ in 0..curr_item.1 {
                        curr_elem = curr_elem.next_element_sibling().unwrap();
                    }
                    // need to update both if the item is different (either by 
                    // the item value itself, or by previous divergence)
                    if diff || prev_item != curr_item {
                        // check if old needs md hidden
                        hide_md(prev_item, &prev_elem);
                        // check if new needs md unhidden
                        unhide_md(curr_item, &curr_elem);
                    } // if both the same, don't need to do anything
                // if no current item, safe to hide md
                } else {
                    // check if old needs md hidden
                    hide_md(prev_item, &prev_elem);
                }
            // if no prev item, safe to unhide
            } else if let Some(curr_item) = new_selection_start.get(i) {
                if curr_item.0 == MDNodeType::Text { continue }
                // get nth child according to curr_item
                curr_elem = curr_elem.first_element_child().unwrap();
                for _ in 0..curr_item.1 {
                    curr_elem = curr_elem.next_element_sibling().unwrap();
                }
                // check if new needs md unhidden
                unhide_md(curr_item, &curr_elem);
            }
        }

        set_selection_start.update(|v| *v = new_selection_start)

        // TODO: find the line clicked on, then run ".innerText" on that div so can update my vec on line strings

    });
    document().add_event_listener_with_callback("selectionchange", closure.as_ref().unchecked_ref()).unwrap();
    closure.forget();
    out
}



fn get_hideable_md_type(elem: &Element) -> HideableMDType {
    let parent_class = elem.class_list().get(0)
        .unwrap_or("".to_string());
    match parent_class.as_str() {
        "h1" => HideableMDType::H1,
        _ => HideableMDType::None,
    }
}
fn get_span_or_block_info(elem: &Element) -> (MDNodeType, usize) {
    console_log(&format!("{:?}", elem.tag_name()));
    // get node type
    if let Some(block_num) = elem.get_attribute("block") {
        (MDNodeType::Block, block_num.parse().unwrap())
    } else {
        let span_num = elem.get_attribute("span").unwrap();
        (MDNodeType::Span, span_num.parse().unwrap())
    }
}

fn hide_md(node_info: &MDNodeInfo, elem: &Element) {
    // FIXME: just realized if the structure of the file changes, 
    // these unwraps could fail
    match node_info.2 {
        HideableMDType::H1 => {
            let hidden_md = elem.first_element_child().unwrap();
            hidden_md.set_attribute("hidden", "").unwrap();
        },
        _ => {},
    }
}
fn unhide_md(node_info: &MDNodeInfo, elem: &Element) {
    match node_info.2 {
        HideableMDType::H1 => {
            let hidden_md = elem.first_element_child().unwrap();
            hidden_md.remove_attribute("hidden").unwrap();
        },
        _ => {},
    }
}

// #[component]
// pub fn H1(cx: Scope, contents: String) -> Element {
//     // go through contents to check if anything here 
//     view! {cx,
//         <h1>"hi"</h1>
//     }
// }
