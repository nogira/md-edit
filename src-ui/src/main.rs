#![feature(box_syntax)]
#![feature(iter_advance_by)]

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
            // <li>"hi"</li>
            // <li>"hi"<ul><li>"hi"</li></ul></li>
            // <SimpleCounter name="Michal".to_string() />
        </div>
    })
}

// use serde::Deserialize;

// // #[derive(Clone)]
// // struct GenericEventRes {
// //     num: usize,
// //     elem: Box<dyn Fn(Scope) -> Element>,
// // }

// fn toggle_bit(idx: u8, val: u8) -> u8 {
//     let bit = 1 << idx;
//     // toggle bit in val
//     val ^ bit
// }

// // struct LineTrack {
// //     start_idx: usize,
// //     /// element type
// //     e_t: LineType,

// // }
// // #[derive(PartialEq)]
// // enum LineType {
// //     Null, Normal, H1
// // }
// // impl LineTrack {
// //     pub fn new() -> Self {
// //         Self { start_idx: 0, e_t: LineType::Null }
// //     }
// //     pub fn waiting_for_line_break(&self) -> bool {
// //         self.e_t != LineType::Null
// //     }
// //     pub fn reset(&mut self, start_idx: usize) {
// //         self.start_idx = start_idx;
// //         self.e_t = LineType::Null;
// //     }
// //     /// track line w/ no special element
// //     pub fn track_normal(&mut self, start_idx: usize) {
// //         self.start_idx = start_idx;
// //         self.e_t = LineType::Normal;
// //     }
// //     /// this uses second bit
// //     pub fn track_h1(&mut self, start_idx: usize) {
// //         self.start_idx = start_idx;
// //         self.e_t = LineType::H1;
// //     }
// //     pub fn get_elem<F>(
// //         &mut self, end_idx: usize, str: &String, key: usize
// //     ) -> GenericEventRes {
// //         let s = self.start_idx;
// //         let e = end_idx;
// //         let elem: GenericEventRes;
// //         match self.e_t {
// //             LineType::Null => panic!(""),
// //             LineType::Normal => {
// //                 elem = GenericEventRes { num: key, elem: box |cx: Scope| view! {cx, <div>"normal"</div>}}
// //             },
// //             LineType::H1 => {
// //                 elem = GenericEventRes{ num: key, elem: box |cx: Scope| view! {cx, <H1 contents=(&str[s..e]).to_string() />}}
// //             },
// //         }
// //         self.reset(end_idx);
// //         elem
// //     }
// // }

// #[component]
// pub fn MarkdownPage(cx: Scope, name: String) -> Element {
//     // let text = "# hello\n this **is** cool".to_string();
//     // let (
//     //     lines_vec,
//     //     set_lines_vec
//     // ) = create_signal::<Vec<GenericEventRes>>(cx, vec![]);
//     // // let mut lines: Vec<ListElem> = Vec::new();
//     // let mut is_h1 = false; // TODO: convert all these bools to a flag !!!
//     //                             // since i am skipping contents, and letting subcomponeent handle contents, just check if flag thing equals 0b0000000 bc if true i can completely skip the char until we reach a \n

//     // let mut line_track = LineTrack::new();

//     // for (i, char) in text.chars().enumerate() {
//     //     if line_track.waiting_for_line_break() {
//     //         if char == '\n' {
//     //             set_lines_vec.update(|v| v.push(
//     //                 line_track.get_elem(i, &text, lines_vec.get().len())
//     //             ))
//     //         }
//     //     } else {
//     //         match char {
//     //             '#' => line_track.track_h1(i),
//     //             _ => line_track.track_normal(i),
//     //         }
//     //     }

//     // }
//     // let (lines_vec, set_lines_vec) = create_signal::<Vec<GenericEventRes>>(cx, vec![]);

//     let (
//         top_line_num,
//         set_top_line_num
//     ) = create_signal::<u32>(cx, 0);

//     create_effect(cx, |_| {console_log("effect")});

//     console_log("RELOAD");

//     let text = "# hello\nthis **is** cool 🤡 ᾎ ᾯ y̆".to_string();
//     let mut blocks: Vec<Element> = Vec::new();

//     /// text intermediary between raw text and html
//     let text_imd: Vec<&str> = Vec::new();

//     /// if a block is edited, only that block updates in the text_imd
//     /// if instead a block is deleted, we need a new way to update text_imd to 
//     /// prevent it from getting inner text of the ENTIRE file (this is 
//     /// especially important for when we add table support, but the innerText 
//     /// of which will not likely be the text we want to save in the file)

//     // i think the easiest way of knowing where selection is is giving each 
//     // span and div a level number so i know how far to go back to get parent, 
//     // and each line should have a line number so i can easisly navigate to the 
//     // line of the original text which will be how i store it (vector of lines (Vec<&str>))

//     /// text_idx is so we know where to start the text parsing from (return when hit \n)
//     fn parse_md(cx: Scope, tag: &str, lvl_num: usize, text_idx: usize, text: &str) -> Vec<Element> {
//         let mut block_num = 0;
//         let mut blocks: Vec<Element> = Vec::new();

//         // BLOCK FOUNDATION INSTEAD OF LINE FOUNDATION IS CLEARLY NEED TO 
//         // HANDLE SHIT LIKE TABLES. then again, how tf do i handle tables 
//         // inside quotes??? HMMMM, then again, maybe i just track the outer 
//         // blocks and rm all preceding shit in the line of blocks im already in

//         // ITS POSSIBLE I CAN CONVERT STRAIGHT FROM DOM TO SINGLE LINE OF TEXT 
//         // W/ `innerText`, BUT THIS IS ONLY USEFUL FOR FILE SAVING, NOT 
//         // UPDATING RENDER
//         //
//         // ==ON THE OTHER HAND, IT MIGHT BE POSSIBLE TO HAVE NO INTERMEDIATE 
//         // DATA STRUCTURE AND JUST UPDATE THE DOM FROM KEY PRESSES==

//         // WOULD CERTAINLY BE WAY FASTER

//         let mut char_1_back = '\n';
//         let mut idx_1_char_back: usize = 0;
//         let mut char_2_back = '\n';
//         let mut idx_2_char_back: usize = 0;
//         // let mut iter = 0..(text.len() - 1);
//         let mut iter = text.char_indices();
//         while let Some((i, char)) = iter.next()  {
//             // let char = &text[i..i+1];
//             // console_log(&format!("{char}"));
//             // console_log(&format!("{i}: {char}"));

//             let next_char = match iter.clone().next() {
//                 Some((_, next_char)) => next_char,
//                 None => continue,
//             };

//             console_log(&format!("prev: {char_2_back} prev: {char_1_back} now: {char} next: {next_char}"));

//             match char {
//                 '#' => {


//                     let elem = view! {cx,
//                         <div class="h1" block=block_num>
//                         "hello"
//                         </div>
//                     };
//                     blocks.push(elem);
//                     block_num += 1;
//                 },
//                 '*' => {
//                     console_log("here");
//                     // ITALIC
//                     if char_1_back == ' ' && next_char != '*' {
//                         // check if it closes before a line-break, if not, ignore
//                         console_log("italic");

//                         // let elem = parse_md(cx, "i", line, i, text);
//                         // blocks.push(elem)
//                     // BOLD
//                     } else if char_1_back == '*' && char_2_back == ' ' {
//                         console_log("bold");
//                         let mut prev_char = ' ';
//                         let mut iter = (&text[i..]).char_indices();
//                         while let Some((j, char)) = iter.next()  {
//                             match char {
//                                 '*' => {
//                                     if prev_char != ' ' {
//                                         if let Some((_, next_char)) = iter.clone().next() {
//                                             if next_char == '*' {
//                                                 console_log(&text[i+1..i+j])
//                                             }
//                                         }
//                                     }
//                                 },
//                                 _ => {}
//                             }
//                             prev_char = char;
//                         }
//                         // let elem = parse_md(cx, "b", line, i, text);
//                         // blocks.push(elem)
//                     }
//                 }
//                 _ => {}
//             }
//             char_2_back = char_1_back;
//             char_1_back = char;
//         }

//         // let parent = create_element(tag);
//         // match tag {
//         //     "div" | "h1" => parent.set_attribute("class", "block").unwrap(),
//         //     _ => {},
//         // }
//         // view! {cx,
//         //     <div class="block">
//         //         {blocks}
//         //     </div>
//         // }

//         blocks
//     }

//     // let div_e = create_element("div");
//     // div_e.set_attribute("line", "1").unwrap();
//     // div_e.set_inner_html("<b><span>**</span>xd<span>**</span></b>");

//     // blocks.push(div_e);

//     // let div_e = create_element("div");
//     // div_e.set_attribute("line", "2").unwrap();
//     // div_e.set_inner_html("<b><span>**</span>xd<span>**</span></b>");

//     // blocks.push(div_e);

//     // let div_s = create_element("span");
//     // div_s.set_inner_html("**");
//     // div_s.set_attribute("hidden", "true").unwrap();
//     // // div_s.add_event_listener_with_callback(type_, listener);

//     // let div_e = view! {cx, <div line="3">"more "<b>"hello"{div_s}</b>" text"</div>};
//     // // div_e.set_inner_html("<b><span>**</span>xd<span>**</span></b>");

//     // structure as blocks and lines. blocks can have sub-blocks. lines have the number of the line OVERALL in the text


//     fn handle_keypress(event: web_sys::KeyboardEvent) {
        
//         // if return key pressed. 
//         if event.char_code() == 13 {
//             console_log("return");
//             event.prevent_default();
//             let selection = document().get_selection().unwrap().unwrap();
//             console_log(&format!("{:?}", selection.anchor_node().unwrap().parent_element().unwrap().get_attribute("block").unwrap()));
//         }
        
//     }

//     let out = view! {cx,
//         // scroll view
//         <div contenteditable on:scroll=|_| console_log("test") on:keypress=handle_keypress>
//             {parse_md(cx, "div", 0, 0, &text)}
//             // {blocks}
//             // // <For each=lines_vec key=|e| e.num>
//             // //     {|cx: Scope, e: &GenericEventRes| {
//             // //         // view! {
//             // //         //     cx,
//             // //         //     <{move {e.elem}} />
//             // //         // }
//             // //         (e.elem)(cx)
//             // //     }}
//             // // </For>
//             // "hello"
//             // // <div html="<b>xd</b>" />.set_inner_html()
//             // {div_e}
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
//         let selection = document().get_selection().unwrap().unwrap();
//         console_log(&format!("{:?}", selection.anchor_node()));
//         console_log(&format!("{:?}", selection.anchor_offset()));
//         console_log(&format!("{:?}", selection.to_string()));

//         // find the line clicked on, then run ".innerText" on that div so can update my vec on line strings
//     });
//     document().add_event_listener_with_callback("selectionchange", closure.as_ref().unchecked_ref()).unwrap();
//     closure.forget();
//     out
// }

// #[component]
// pub fn H1(cx: Scope, contents: String) -> Element {
//     // go through contents to check if anything here 
//     view! {cx,
//         <h1>"hi"</h1>
//     }
// }