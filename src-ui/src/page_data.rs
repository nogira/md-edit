use leptos::{Scope, RwSignal, create_rw_signal, js_sys::Math};
use std::{hash::{Hash, Hasher}, collections::{HashMap, hash_map::DefaultHasher}};

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
    pub fn signal_from(cx: Scope, nodes: RwSignal<Vec<RwSignal<PageNode>>>, 
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
    pub inner_edge_y: i32, // FIXME: think i can delete this bc saving not useful ??
}
impl EdgeElem {
    pub fn from(hash: String, pad: u32, inner_edge_y: i32) -> Self {
        Self {hash, pad, inner_edge_y}
    }
    pub fn signal_from(cx: Scope, hash: String, pad: u32, inner_edge_y: i32) -> RwSignal<Self> {
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
    pub height: u32,
    // /// the y-axis top of the element in pixels
    // pub top: usize,
    // /// the y-axis bottom of the element in pixels
    // pub bottom: usize,
}
impl PageNode {
    pub fn from(hash: String, kind: PageNodeType, 
        contents: PageNodeContents, height: u32,
    ) -> Self {
        Self {hash, kind, contents, height}
    }
    pub fn signal_from(cx: Scope, hash: String, kind: PageNodeType, 
        contents: PageNodeContents, height: u32,
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
    pub fn signal_from_children(cx: Scope, children: Vec<RwSignal<PageNode>>) -> Self {
        Self::Children(create_rw_signal(cx, children))
    }
    pub fn signal_from_content(cx: Scope, content: HashMap<String, String>) -> Self {
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
    pub fn value(&self) -> &str {
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
    pub fn is_block(&self) -> bool {
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

pub fn add_hashes(nodes: Vec<RwSignal<PageNode>>, location: Vec<usize>, 
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
