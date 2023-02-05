use leptos::{log, Scope, RwSignal, create_rw_signal, js_sys::Math, UntrackedSettableSignal, UntrackedGettableSignal, ev::scroll, JsCast, document};
use web_sys::{Node, Element};
use std::{hash::{Hash, Hasher, self}, collections::{HashMap, hash_map::DefaultHasher}};
use crate::editable_page::CreateElem;

use super::{get_top_block_node, get_bot_block_node, get_node_from_location, ElemIsInView};

// tried doing `struct PageSignal(RwSignal<Page>)` wrapper but it introduced 
// waaaaaaaaay too much complexity that i cbf solving
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Page {
    pub nodes: RwSignal<PageNode>,
    /// also use this to calculate scroll position
    pub top_elem: RwSignal<EdgeElem>,
    pub bot_elem: RwSignal<EdgeElem>,
    pub locations: RwSignal<HashMap<String, Vec<usize>>>,
    // pub undo_hist: RwSignal<Vec<UndoEvent>>,
}
// /// this also covers redo evvents
// struct UndoEvent {
//     start_pos: Vec<usize>,
//     end_pos: Vec<usize>,
//     kind: UndoEventKind,
//     content: String, // this might need to be `PageNode`
// }
// enum UndoEventKind {
//     Insert, Remove
// }
pub trait InsertHash {
    fn insert_hash(&self, hash: String, location: Vec<usize>);
}
impl InsertHash for RwSignal<HashMap<String, Vec<usize>>> {
    /// add/update hash
    fn insert_hash(&self, hash: String, location: Vec<usize>) {
        self.update_untracked(|ls| {
            ls.insert(hash, location);
        })
    }
}
pub trait RemoveHash {
    fn remove_hash(&self, hash: &String);
}
impl RemoveHash for RwSignal<HashMap<String, Vec<usize>>> {
    fn remove_hash(&self, hash: &String) {
        self.update_untracked(|ls| {
            ls.remove(hash);
        })
    }
}
pub trait ContainsHash {
    fn contains_hash(&self, hash: &String) -> bool;
}
impl ContainsHash for RwSignal<HashMap<String, Vec<usize>>> {
    fn contains_hash(&self, hash: &String) -> bool {
        self.update_returning_untracked(|ls| {
            ls.contains_key(hash)
        }).unwrap()
    }
}
pub trait HashToLocation {
    fn hash_to_location(&self, hash: &String) -> Vec<usize>;
}
impl HashToLocation for RwSignal<Page> {
    fn hash_to_location(&self, hash: &String) -> Vec<usize> {
        self.update_returning_untracked(|p| {
            p.locations.update_returning_untracked(|ls| {
                ls.get(hash).unwrap().clone()
            }).unwrap()
        }).unwrap()
    }
}
impl HashToLocation for RwSignal<HashMap<String, Vec<usize>>> {
    fn hash_to_location(&self, hash: &String) -> Vec<usize> {
        self.update_returning_untracked(|ls| {
            ls.get(hash).unwrap().clone()
        }).unwrap()
    }
}
// TODO: REIMPLEMENT THIS FUNCTION NATIVELY IN IMPL INSTEAD OF CALLING TO get_node_from_hash

pub trait HashToNode {
    fn hash_to_node(&self, hash: &String) -> Option<RwSignal<PageNode>>;
}
impl HashToNode for RwSignal<Page> {
    fn hash_to_node(&self, hash: &String) -> Option<RwSignal<PageNode>> {
        let loc = self.hash_to_location(hash);
        let nodes = self.update_returning_untracked(|p| {
            p.nodes.get().children
        }).unwrap();
        get_node_from_location(&loc, &nodes)
    }
}
impl Page {
    pub fn signal_from(cx: Scope, nodes: RwSignal<PageNode>, 
        top_elem: RwSignal<EdgeElem>, bot_elem: RwSignal<EdgeElem>, 
        locations: RwSignal<HashMap<String, Vec<usize>>>,
    ) -> RwSignal<Self> {
        create_rw_signal(cx, Self {nodes, top_elem, bot_elem, locations}) 
    }
}
impl Page {
    pub fn debug_nodes(&self) -> String {
        let nodes = &self.nodes.get();
        let slice = Vec::from([nodes.children[2]]);
        let lines = Self::debug_nodes_recursive(&slice);
        let mut string = String::new();
        for line in lines {
            string.push_str(&format!("{}\n", line));
        }
        string
    }
    pub fn debug_nodes_recursive(nodes: &Vec<RwSignal<PageNode>>) -> Vec<String> {
        let mut lines = Vec::new();
        for node in nodes {
            let node = node.get();

            lines.push("Node<".into());
            lines.push(format!("    hash: {},", node.hash));
            let has_elem_ref = match node.elem_ref {
                Some(_) => true,
                None => false,
            };
            lines.push(format!("    has_elem_ref: {:?},", has_elem_ref));
            let children = Self::debug_nodes_recursive(&node.children);
            lines.push(format!("    children: ["));
            for child in children {
                lines.push(format!("        {}", child));
            }
            lines.push(format!("    ]"));
            lines.push(">,".into());
        }
        lines
    }
}
/// the top or bottom element of the view
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EdgeElem {
    /// the hash/id of top element
    pub hash: String,
    /// signal to the node
    pub node_sig: RwSignal<PageNode>,
    // pub location: Vec<usize>, // location not needed bc can use hash to get it from hashset
    /// - if top elem: top = `padding-top` attribute, bot elem: `padding-bot`
    /// - padding is applied to the base node  (e.g. if node is `vec![1, 3, 2]`, 
    /// padding applied to base node of index 1)
    pub pad: u32,
    /// scroll offset of top_elem from top of scroll so we can get back to page 
    /// position on reload
    pub scroll_offset: i64,
}
impl EdgeElem {
    // pub fn from(hash: String, pad: u32, inner_edge_y: i32) -> Self {
    //     Self {hash, pad, inner_edge_y}
    // }
    pub fn signal_from(cx: Scope, hash: String, node_sig: RwSignal<PageNode>, pad: u32, scroll_offset: i64) -> RwSignal<Self> {
        create_rw_signal(cx, Self {hash, node_sig, pad, scroll_offset})
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PageNode {
    pub hash: String,
    pub kind: PageNodeType,
    pub content: HashMap<String, String>,
    pub children: Vec<RwSignal<PageNode>>,
    pub parent: Option<RwSignal<PageNode>>,
    pub elem_ref: Option<Element>,
    /// height of all elems is tracked so we can have an accurate scroll page 
    /// length without having to render the page down to the bottom
    pub height: u32,
    // /// the y-axis top of the element in pixels
    // pub top: usize,
    // /// the y-axis bottom of the element in pixels
    // pub bottom: usize,
}
impl PageNode {
    pub fn from(hash: String, kind: PageNodeType, 
        content: HashMap<String, String>, children: Vec<RwSignal<PageNode>>, 
        parent: Option<RwSignal<PageNode>>, height: u32,
    ) -> Self {
        Self {hash, kind, content, children, parent, elem_ref: None, height}
    }
    pub fn signal_from(cx: Scope, hash: String, kind: PageNodeType, 
        content: HashMap<String, String>, children: Vec<RwSignal<PageNode>>, 
        parent: Option<RwSignal<PageNode>>, height: u32,
    ) -> RwSignal<Self> {
        create_rw_signal(cx, Self {hash, kind, content, children, parent, elem_ref: None, height})
    }
    pub fn is_block(&self) -> bool {
        self.kind.is_block()
    }
    pub fn is_leaf_block(&self) -> bool {
        self.kind.is_block() && !self.children[0].get().is_block()
    }
}
pub trait IsBlock {
    fn is_block(&self) -> bool;
}
impl IsBlock for RwSignal<PageNode> {
    fn is_block(&self) -> bool {
        self.update_returning_untracked(|n| {
            n.kind.is_block()
        }).unwrap()
    }
}
pub trait IsLeafBlock {
    fn is_leaf_block(&self) -> bool;
}
impl IsLeafBlock for RwSignal<PageNode> {
    fn is_leaf_block(&self) -> bool {
        self.update_returning_untracked(|n| {
            n.kind.is_block() && !n.children[0].get().is_block()
        }).unwrap()
    }
}
pub trait IsFirstChild<T> {
    fn is_first_child(&self, child: &T) -> bool;
}
impl IsFirstChild<RwSignal<PageNode>> for RwSignal<PageNode> {
    fn is_first_child(&self, child: &RwSignal<PageNode>) -> bool {
        self.update_returning_untracked(|p| {
            let first_child = p.children[0].clone();
            &first_child == child
        }).unwrap()
    }
}
pub trait IsLastChild<T> {
    fn is_last_child(&self, child: &T) -> bool;
}
impl IsLastChild<RwSignal<PageNode>> for RwSignal<PageNode> {
    fn is_last_child(&self, child: &RwSignal<PageNode>) -> bool {
        self.update_returning_untracked(|p| {
            let last_child = p.children.last().unwrap().clone();
            &last_child == child
        }).unwrap()
    }
}
pub trait PrevChild<T> {
    fn prev_child(&self, child: &T) -> Option<RwSignal<PageNode>>;
}
impl PrevChild<RwSignal<PageNode>> for RwSignal<PageNode> {
    fn prev_child(&self, child: &RwSignal<PageNode>) -> Option<RwSignal<PageNode>> {
        self.update_returning_untracked(|p| {
            let len_children = p.children.len();
            for i in 0..len_children {
                if &p.children[i] == child {
                    if i == 0 { return None }
                    return match p.children.get(i-1) {
                        Some(child) => Some(child.clone()),
                        None => None,
                    };
                }
            }
            return None
        }).unwrap()
    }
}
pub trait NextChild<T> {
    fn next_child(&self, child: &T) -> Option<RwSignal<PageNode>>;
}
impl NextChild<RwSignal<PageNode>> for RwSignal<PageNode> {
    fn next_child(&self, child: &RwSignal<PageNode>) -> Option<RwSignal<PageNode>> {
        self.update_returning_untracked(|p| {
            let len_children = p.children.len();
            for i in 0..len_children {
                if &p.children[i] == child {
                    return match p.children.get(i+1) {
                        Some(child) => Some(child.clone()),
                        None => None,
                    };
                }
            }
            return None
        }).unwrap()
    }
}
pub trait RemoveChild<T> {
    fn remove_child(&self, child: &T);
}
impl RemoveChild<RwSignal<PageNode>> for RwSignal<PageNode> {
    /// remove child (page node & DOM elem)
    fn remove_child(&self, node: &RwSignal<PageNode>) {
        self.update_untracked(|n| {
            for (i, child) in n.children.clone().iter().enumerate() {
                if child == node {
                    n.children.remove(i);
                    if let Some(elem) = child.update_returning_untracked(|n| n.elem_ref.clone()).unwrap() {
                        elem.remove();
                    }
                    return
                }
            }
        });
    }
}
pub trait ChangeBlockKind {
    fn change_block_kind(&self, new_kind: PageNodeType);
}
impl ChangeBlockKind for RwSignal<PageNode> {
    fn change_block_kind(&self, new_kind: PageNodeType) {
        self.update_untracked(|p| {
            p.kind = new_kind.clone();
            if let Some(elem_ref) =  p.elem_ref.clone() {
                elem_ref.set_attribute("type", new_kind.value()).unwrap();
            }
        });
    }
}

pub trait InsertText {
    fn insert_text(&self, from: RwSignal<PageNode>);
}
impl InsertText for RwSignal<PageNode> {
    fn insert_text(&self, from: RwSignal<PageNode>) {
        // self.update_untracked(|p| {
        //     p.kind = new_kind.clone();
        //     if let Some(elem_ref) =  p.elem_ref.clone() {
        //         elem_ref.set_attribute("type", new_kind.value()).unwrap();
        //     }
        // });
    }
}
pub trait InsertNodes {
    fn insert_nodes(&self, nodes: &Vec<RwSignal<PageNode>>, before: Option<&RwSignal<PageNode>>);
}
impl InsertNodes for RwSignal<PageNode> {
    /// `before` is the node to insert before
    fn insert_nodes(&self, nodes: &Vec<RwSignal<PageNode>>, before: Option<&RwSignal<PageNode>>) {
        self.update_untracked(|parent| {
            let parent_elem = parent.elem_ref.clone().unwrap();
            let children = parent.children.clone();
            // if there is a before node, insert before, else insert as last child
            if let Some(before_sig) = before {
                // find idx of elem
                let mut idx = None;
                // children.ind
                for (i, node_sig) in children.iter().enumerate() {
                    if node_sig == before_sig {
                        idx = Some(i);
                        break;
                    }
                }
                if idx.is_none() { panic!("failed to insert nodes bc the node to insert before is not present in the parent block") }
                let idx = idx.unwrap();

                // BEFORE ELEM MIGHT NOT BE PRESENT ON THE SCREEN, BUT SOME OF 
                // THE ELEMENTS ON THE SCREEN MIGHT BE
                // i guess this can be handled by skipping element insertion 
                // if elem not present, but then, then immediately just 
                // re-rendering the page
                let optional_before_elem = match before_sig.get().elem_ref {
                    Some(elem) => Some(elem.dyn_into::<Node>().unwrap()),
                    None => None,
                };

                // must check EACH elem is in view to then remove if not bc 
                // even if you track the first elem that is not in view, the 
                // first elems might not be in view, but the later ones might be
                for (i, node_sig) in nodes.iter().enumerate() {
                    // if the node we're moving is already rendered, use it
                    let node_elem = node_sig.get().elem_ref
                        // must create a new node to insert if it's not already present
                        .unwrap_or(node_sig.create_elem());
                    // this defaults to `.add_child()` if `optional_before_elem` is `None`
                    parent_elem.insert_before(&node_elem, 
                        optional_before_elem.as_ref()).unwrap();
                    node_sig.update_untracked(|n| {
                        // remove if not in view
                        if !node_elem.elem_is_in_view() {
                            log!("NOT IN VIEW");
                            node_elem.remove();
                            n.elem_ref = None;
                        } else {
                            log!("IN VIEW");
                            n.elem_ref = Some(node_elem);
                        }
                        n.parent = Some(self.clone());
                        log!("NO CHILDREN? hash: {:?}", n.children[0].get_untracked().hash);
                    });
                    // log!("pre-len: {:?}", parent.children.len());
                    parent.children.insert(idx+i, node_sig.to_owned());
                    // log!("post-len: {:?}", parent.children.len());
                }
            // insert as last children
            } else {
                for node_sig in nodes {
                    // if the node we're moving is already rendered, use it
                    let node_elem = node_sig.get().elem_ref
                        // must create a new node to insert if it's not already present
                        .unwrap_or(node_sig.create_elem());
                    parent_elem.append_child(&node_elem).unwrap();
                    node_sig.update_untracked(|n| {
                        // remove if not in view
                        if !node_elem.elem_is_in_view() {
                            node_elem.remove();
                            n.elem_ref = None;
                        } else {
                            n.elem_ref = Some(node_elem);
                        }
                        n.parent = Some(self.clone())
                    });
                    parent.children.push(node_sig.to_owned());
                }
            }
        });
    }
}

pub trait RemoveThisBlockShell {
    fn remove_this_block_shell(&self);
}
impl RemoveThisBlockShell for RwSignal<PageNode> {
    /// remove this block, while placing the contents on the same level as 
    /// this block
    fn remove_this_block_shell(&self) {
        let curr_block = self.get_untracked();
        // remove each child, using this block as the node to insert before
        let parent_sig = curr_block.parent.unwrap();
        log!("pre-len: {:?}", parent_sig.get().children.len());
        parent_sig.insert_nodes(&curr_block.children, Some(self));
        log!("post-len: {:?}", parent_sig.get().children.len());
        // remove this block
        parent_sig.remove_child(self);
    }
}

// struct RwSignal<T>()
// impl Debug for RwSignal<PageNode> {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(f, "RwSignal PageNode({:?}, {:?})", self.hash, self.kind)
//     }
// }

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PageNodeType {
    // root
    Page, // using Page as root so able to attach a method to index node (as 
          // otherwise you would have to first index the root RwSignal<Vec<PageNode>> 
          // before runniing PageNode index method)
          //
          // also it allows you to insert root level elements without needing 
          // to have access to elem_ref, nor account for the spacers at top 
          // and bottom
    // block-branch
    Indent, Quote, 
    // block-leaf
    TextBlock, H1, H2, H3, H4, H5, CodeBlock, Dot, Num, Check, Table, // tbh table could be a branch too ???
    // text-branch
    Bold, Italic, Highlight, CodeInline, FileLink, UrlLink,
    // text-leaf
    RawText,
}
#[derive(Clone, Copy)]
struct PageNodeTypeInfo {
    val: &'static str,
    /// true = block, false = span
    block: bool,
    /// true = branch, false = leaf
    branch: bool,
    innate_height: u32,
}
const NUM_TYPES: usize = 21;
// NOTE: THIS MUST BE IN SAME ORDER AS THE ENUM  FOR THE INDEXING TO WORK
const PAGE_NODE_TYPE_INFO: [PageNodeTypeInfo; NUM_TYPES] = [
    // Page
    PageNodeTypeInfo { val: "p", block: true, branch: true, innate_height: 0 },

    // Indent
    PageNodeTypeInfo { val: "in", block: true, branch: true, innate_height: 0 },
    // Quote
    PageNodeTypeInfo { val: "q", block: true, branch: true, innate_height: 0 },

    // TextBlock
    PageNodeTypeInfo { val: "tb", block: true, branch: false, innate_height: 0 },
    // H1
    PageNodeTypeInfo { val: "h1", block: true, branch: false, innate_height: 0 },
    // H2
    PageNodeTypeInfo { val: "h2", block: true, branch: false, innate_height: 0 },
    // H3
    PageNodeTypeInfo { val: "h3", block: true, branch: false, innate_height: 0 },
    // H4
    PageNodeTypeInfo { val: "h4", block: true, branch: false, innate_height: 0 },
    // H5
    PageNodeTypeInfo { val: "h5", block: true, branch: false, innate_height: 0 },
    // CodeBlock
    PageNodeTypeInfo { val: "cd", block: true, branch: false, innate_height: 0 },
    // Dot
    PageNodeTypeInfo { val: "d", block: true, branch: false, innate_height: 0 },
    // Num
    PageNodeTypeInfo { val: "n", block: true, branch: false, innate_height: 0 },
    // Check
    PageNodeTypeInfo { val: "ch", block: true, branch: false, innate_height: 0 },
    // Table
    PageNodeTypeInfo { val: "tl", block: true, branch: false, innate_height: 0 },

    // Bold
    PageNodeTypeInfo { val: "b", block: false, branch: true, innate_height: 0 },
    // Italic
    PageNodeTypeInfo { val: "i", block: false, branch: true, innate_height: 0 },
    // Highlight
    PageNodeTypeInfo { val: "h", block: false, branch: true, innate_height: 0 },
    // CodeInline
    PageNodeTypeInfo { val: "ci", block: false, branch: true, innate_height: 0 },
    // FileLink
    PageNodeTypeInfo { val: "fl", block: false, branch: true, innate_height: 0 },
    // UrlLink
    PageNodeTypeInfo { val: "ul", block: false, branch: true, innate_height: 0 },

    // RawText
    PageNodeTypeInfo { val: "t", block: false, branch: false, innate_height: 0 },
];

impl PageNodeType {
    pub fn value(&self) -> &str {
        PAGE_NODE_TYPE_INFO[self.clone() as usize].val
    }
    pub fn is_block(&self) -> bool {
        PAGE_NODE_TYPE_INFO[self.clone() as usize].block
    }
    pub fn is_branch(&self) -> bool {
        PAGE_NODE_TYPE_INFO[self.clone() as usize].branch
    }
    /// get the height of the node without any contents (e.g. if the quote 
    /// block had top/bottom padding, get the sum height of the padding)
    pub fn innate_height(&self) -> u32 {
        PAGE_NODE_TYPE_INFO[self.clone() as usize].innate_height
    }
}

pub fn add_hashes(nodes: &Vec<RwSignal<PageNode>>, location: Vec<usize>, 
    locations: RwSignal<HashMap<String, Vec<usize>>>) {
    for (i, node) in nodes.iter().enumerate() {
        let mut location = location.clone();
        location.push(i);
        // create & add hash/location
        let mut hash = rand_utf8_hash();
        loop { // keep generating until find hash not already used
            if !locations.contains_hash(&hash) { break }
            hash = rand_utf8_hash();
        }
        locations.insert_hash(hash.clone(), location.clone());
        node.update_untracked(|e| e.hash = hash.clone());
        // if children present in node, update those too
        add_hashes(&node.get().children, location, locations);
    }
}
/// overwrite each hash location in `locations` with its current location
pub fn update_hash_locations(nodes: &Vec<RwSignal<PageNode>>, location: Vec<usize>, 
    locations: RwSignal<HashMap<String, Vec<usize>>>) {
    for (i, node) in nodes.iter().enumerate() {
        let mut location = location.clone();
        location.push(i);
        // update hash location
        let hash = node.update_returning_untracked(|n| n.hash.clone()).unwrap();
        locations.insert_hash(hash.clone(), location.clone());
        // if children present in node, update those too
        update_hash_locations(&node.get().children, location, locations);
    }
}

/// generate a utf-8 hash string of length 3
fn rand_utf8_hash() -> String {
    // chars used: 256 utf-8
    // generated random number: u64 = 2^64 = 18.4 quintillion
    // 256^3 = 16.777 million
    // 2^24 = 16.777 million <-- steel-manning max number of hashes needed lets assume each word has its own hash. at 17 million words, the text file would be approx 100mb. these seems extremely unlikely. https://www.quora.com/If-I-would-have-a-1GB-text-file-txt-approximately-how-many-words-would-it-include
    // 256^4 = 4.29 billion
    // 2^32 = 4.29 billion
    let gen_rand_num = || {
        let mut hasher = DefaultHasher::new();
        Math::random().to_bits().hash(&mut hasher);
        let bits_32 = hasher.finish() as u32;
        // 32 - 24 = 8
        let clipper = u32::MAX >> 8;
        let clipped = bits_32 & clipper;
        clipped
    };
    // we're treating each char in the hash as a number with base 256 (like 
    // how the decimal system uses a number with base 10)
    const BASE: u32 = 256_u32;
    let mut hash_str = String::new();
    let mut carry = gen_rand_num();
    loop { // loop to generate each of the 3 chars of the hash
        let rem = carry % BASE;
        hash_str.push(char::from_u32(rem).unwrap());
        if carry == rem { return hash_str }
        carry = (carry - rem) / BASE;
    }
}

pub fn init_demo_page_data(cx: Scope) -> RwSignal<Page> {
    let page = PageNode::signal_from(cx, 
        "".into(), PageNodeType::Page,
        HashMap::new(), Vec::new(), None, 0
    );
    let mut nodes = Vec::new();
    let raw_text_template = PageNode::from(
        "".into(), PageNodeType::RawText,
        HashMap::from([
            ("text".to_string(), "some text".to_string())
        ]), Vec::new(), None, 0
    );
    let h1_template = PageNode::from(
        "".into(), PageNodeType::H1,HashMap::new(),
        Vec::new(), None, 0
    );
    let dot_block_template = PageNode::from(
        "".into(), PageNodeType::Dot, HashMap::new(),
        Vec::new(), None, 0
    );
    let quote_block_template = PageNode::from(
        "".into(), PageNodeType::Quote, HashMap::new(),
        Vec::new(), None, 0
    );
    let text_block_template = PageNode::from(
        "".into(), PageNodeType::TextBlock, HashMap::new(),
        Vec::new(), None, 0
    );
    for i in 0..5 {
        {
            let h1_node_sig = create_rw_signal(cx, h1_template.clone());
            let mut new_child = raw_text_template.clone();
            new_child.content.get_mut("text").unwrap().push_str(&format!(" {}", i));
            new_child.parent = Some(h1_node_sig.clone());
            h1_node_sig.update(|n| {
                n.children.push(create_rw_signal(cx, new_child));
                n.parent = Some(page);
            });
            nodes.push(h1_node_sig);
        }

        {
            let parent = create_rw_signal(cx, dot_block_template.clone());
            let mut text_child = raw_text_template.clone();
            text_child.content.get_mut("text").unwrap().push_str(&format!(" {}", i));
            text_child.parent = Some(parent.clone());
            parent.update(|n| {
                n.children.push(create_rw_signal(cx, text_child));
                n.parent = Some(page);
            });
            nodes.push(parent);
        }
        {
            let parent = create_rw_signal(cx, quote_block_template.clone());
            let child_1 = create_rw_signal(cx, text_block_template.clone());
            let mut text_child_1 = raw_text_template.clone();
            text_child_1.content.get_mut("text").unwrap().push_str(&format!(" {}", i));
            text_child_1.parent = Some(child_1.clone());
            child_1.update(|n| {
                n.children.push(create_rw_signal(cx, text_child_1));
                n.parent = Some(parent.clone());
            });
            let child_2 = create_rw_signal(cx, text_block_template.clone());
            let mut text_child_2 = raw_text_template.clone();
            text_child_2.parent = Some(child_2.clone());
            child_2.update(|n| {
                n.children.push(create_rw_signal(cx, text_child_2));
                n.parent = Some(parent.clone());
            });
            parent.update(|n| {
                n.children = Vec::from([child_1, child_2]);
                n.parent = Some(page);
            });
            nodes.push(parent);
        }
    }
    page.update(|p| {
        p.children = nodes.clone();
    });
    let locations = create_rw_signal(cx, HashMap::new());
    add_hashes(&nodes, Vec::new(), locations);

    // BC SCREEN WIDTH IS VARIABLE, SET TOP AND BOTTOM ELEM TO THE TOP_ELEM, 
    // THEN TRIGGER THE IN-VIEW THING TO RENDER TO BOTTOM OF VIEW

    // let top_node = get_top_block_node(&nodes);
    // let top_hash = top_node.get().hash;
    // log!("{}", top_hash);
    // let bot_node = get_bot_block_node(&nodes);
    // let bot_hash = top_node.get().hash;
    // get lower top_node to test scroll to top_node on init
    let top_node = nodes[3];
    let top_hash = top_node.get().hash;

    Page::signal_from(cx,
        page,
        EdgeElem::signal_from(cx, top_hash.clone(), top_node.clone(), 0, 0),
        EdgeElem::signal_from(cx, top_hash, top_node, 0, 0),
        locations,
    )
}