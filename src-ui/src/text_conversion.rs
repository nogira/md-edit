use core::fmt::Debug;
use leptos::*;
// use src_ui::*;

/// a block is either a full string or a collection of sub-blocks. e.g.
/// ```md
/// > some text           <- block 0 (quote) sub-block 0 (none)
/// > some more text      <- block 0 (quote) sub-block 1 (none)
/// ```
#[derive(Clone)]
pub enum MDBlock {
    /// block w/ no children blocks
    /// 
    /// (block type, content)
    Leaf(MDLeafBlock), // (MDBlockType, String)
    /// block w/ children blocks
    /// 
    /// (block type, pre-string, child blocks)
    Branch(MDBranchBlock), // (MDBlockType, String, Vec<MDBlock>)
}
#[derive(Clone)]
pub struct MDLeafBlock {
    pub kind: MDBlockType,
    pub content: String,
}
#[derive(Clone)]
pub struct MDBranchBlock {
    pub kind: MDBlockType,
    pub children: Vec<MDBlock>,
}
impl MDBlock {
    fn new_leaf(kind: MDBlockType, content: String) -> MDBlock {
        MDBlock::Leaf(MDLeafBlock {
            kind, content
        })
    }
    fn new_branch(kind: MDBlockType, children: Vec<MDBlock>) -> MDBlock {
        MDBlock::Branch(MDBranchBlock {
            kind, children
        })
    }
}
impl Debug for MDBlock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MDBlock::Leaf(leaf) => {
                write!(f, "MDBlock::Leaf({:?}, {:?})", leaf.kind, leaf.content)
            },
            MDBlock::Branch(branch) => {
                write!(f, "MDBlock::Branch({:?}, {:?})", branch.kind, branch.children)
            },
        }
    }
}

#[derive(Debug, Clone)]
pub enum MDBlockType {
    Text, Tab, Quote, H1, H2, H3, Table, Dot, Num, Code
}

/// `pre` is the preceding text (if any) that we're using to detect 
/// continuation of the block
/// 
/// the usize returned is the end idx so we know how many chars to skip
pub fn text_to_imd(text: &str) -> Vec<MDBlock> {
    let mut blocks = Vec::new();
    // let mut block_start_idx = 0;
    let block_start = true;
    let mut iter = text.chars().enumerate();
    while let Some((i, char)) = iter.next() {
        console_log(&format!("CHAR: {:?}", char));
        // parse char
        match char {
            // ===LEAF ONLY BLOCKS===
            '#' => {
                let next_char = iter.clone().next().unwrap().1;
                if next_char == ' ' {
                    let start_idx = i+2;
                    let (txt_len, contents
                    ) = get_leaf_end_index_and_content(&text[start_idx..]);
                    blocks.push(MDBlock::new_leaf(MDBlockType::H1, contents));
                    // + 2 bc added 2 to start index
                    iter.advance_by(txt_len + 2).unwrap();
                    continue;
                }
            },
            '-' => {
                let next_char = iter.clone().next().unwrap().1;
                if next_char == ' ' {
                    let start_idx = i+2;
                    let (txt_len, contents
                    ) = get_leaf_end_index_and_content(&text[start_idx..]);
                    blocks.push(MDBlock::new_leaf(MDBlockType::Dot, contents));
                    // + 2 bc added 2 to start index
                    iter.advance_by(txt_len + 2).unwrap();
                    continue;
                }
            },
            // ===BRANCH BLOCKS=== (recursive)
            '>' => {
                // skip if no ' ' after
                let next_char = iter.clone().next().unwrap().1;
                if next_char == ' ' {
                    // TODO: reimplement text_to_imd as A to see is better
                    //
                    // A) pass the whole string sliced from i and let the 
                    //    recursive function detect when to end. (seems it 
                    //    would need a lot more logic, and requires passing a 
                    //    whole bunch of vars there (pre, block) and back 
                    //    (end_idx), but it also avoids making any copies of 
                    //    the string until leaf node)
                    //
                    // B) parse the string right here, detecting when it ends 
                    //    and removing all pre shid (but again, this req 
                    //    more copies, and more iterations over the string)

                    // get string until end of "> " quotes, then recurse
                    let (txt_len, contents
                    ) = get_block_end_index_and_content(&text[i+2..], "> ");
                    // recurse
                    let sub_blocks = text_to_imd(&contents);
                    blocks.push(MDBlock::new_branch(MDBlockType::Quote, sub_blocks));
                    // + 2 bc added 2 to start index
                    iter.advance_by(txt_len + 2)
                        // if end of line
                        .unwrap_or(());
                    continue;
                }
            }
            '\t' => {
                // get string until end of "\t" prepending, then recurse
                let (txt_len, contents
                ) = get_block_end_index_and_content(&text[i+1..], "\t");
                // recurse
                let sub_blocks = text_to_imd(&contents);
                blocks.push(MDBlock::new_branch(MDBlockType::Tab, sub_blocks));
                // + 2 bc added 2 to start index
                iter.advance_by(txt_len + 1)
                    // if end of line
                    .unwrap_or(());
                continue;
            }
            _ => {},
        }

        // if matches didn't work out, parse as normal leaf

        let (txt_len, contents) = get_leaf_end_index_and_content(&text[i..]);
        // console_log(&format!("none slice: {:?}", &text[i..]));
        // console_log(&format!("h1 txt: {:?}", contents));
        blocks.push(MDBlock::new_leaf(MDBlockType::Text, contents));
        iter.advance_by(txt_len)
            // if at end of text, will no be able to advance by the +1, but 
            // no need to do `iter.advance_by(end_idx)` bc the 
            // `iter.advance_by(end_idx + 1)` already did this
            .unwrap_or(()); 

    }
    blocks
}

pub fn imd_to_text(imd: Vec<MDBlock>) -> &'static str {
    todo!()
}

fn get_leaf_end_index_and_content(text: &str) -> (usize, String) {
    let mut leaf_str = String::new();
    let mut end_idx = text.len() - 1;
    let mut sub_iter = text.chars().enumerate();
    while let Some((i, char)) = sub_iter.next() {
        if char == '\n' {
            end_idx = i;
            break;
        } else {
            leaf_str.push(char);
        }
    }
    (end_idx, leaf_str)
}
fn get_block_end_index_and_content(text: &str, pre: &str) -> (usize, String) {
    let mut trimmed_str = String::new();
    let pre_len = pre.len();
    let mut end_idx = text.len() - 1;
    let mut iter = text.chars().enumerate();
    while let Some((i, char)) = iter.next() {
        // console_log(&format!("h1 char: {:?}", char));
        if char == '\n' {
            let next_chars = &text[i+1..i+1+pre_len]; // FIXME: THIS COULD CRASH IF END OF TEXT
            // console_log(&format!("h1 idx: {:?}", j));
            if next_chars != pre {
                end_idx = i;
                break;
            // if does equal, skip the chars so they're not added to trimmed_str, 
            // but keep the `\n`
            } else {
                trimmed_str.push(char);
                iter.advance_by(pre_len).unwrap();
            }
        } else {
            trimmed_str.push(char);
        }
    }
    console_log(&format!("trimmed str: {:?}", trimmed_str));
    (end_idx, trimmed_str)
}