use leptos::{Scope, RwSignal, create_rw_signal, js_sys::Math};
use web_sys::Element;
use std::{hash::{Hash, Hasher}, collections::{HashMap, hash_map::DefaultHasher}};
use super::{get_top_block_node, get_bot_block_node};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Page {
    pub nodes: RwSignal<PageNode>,
    /// also use this to calculate scroll position
    pub top_elem: RwSignal<EdgeElem>,
    pub bot_elem: RwSignal<EdgeElem>,
    pub locations: RwSignal<HashMap<String, Vec<usize>>>,
}
impl Page {
    pub fn signal_from(cx: Scope, nodes: RwSignal<PageNode>, 
        top_elem: RwSignal<EdgeElem>, bot_elem: RwSignal<EdgeElem>, 
        locations: RwSignal<HashMap<String, Vec<usize>>>
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
    /// - if top elem: bottom edge of the elem. once it passes over the top of 
    /// the page + some px, it signals the element should be unrendered, and 
    /// new top-elem chosen
    pub inner_edge_y: i32, // FIXME: think i can delete this bc saving not useful ??
}
impl EdgeElem {
    // pub fn from(hash: String, pad: u32, inner_edge_y: i32) -> Self {
    //     Self {hash, pad, inner_edge_y}
    // }
    pub fn signal_from(cx: Scope, hash: String, node_sig: RwSignal<PageNode>, pad: u32, inner_edge_y: i32) -> RwSignal<Self> {
        create_rw_signal(cx, Self {hash, node_sig, pad, inner_edge_y})
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
            PageNodeType::Page => "p",
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
            PageNodeType::Page => true,
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
    /// get the height of the node without any contents (e.g. if the quote 
    /// block had top/bottom padding, get the sum height of the padding)
    pub fn innate_height(&self) -> u32 {
        match *self {
            // PageNodeType::Quote => 0,
            // PageNodeType::TextBlock => 0,
            // PageNodeType::H1 => 0,
            // PageNodeType::H2 => 0,
            // PageNodeType::H3 => 0,
            // PageNodeType::CodeBlock => 0,
            // PageNodeType::Bold => 0,
            // PageNodeType::Italic => 0,
            // PageNodeType::Highlight => 0,
            // PageNodeType::CodeInline => 0,
            // PageNodeType::FileLink => 0,
            // PageNodeType::UrlLink => 0,
            // PageNodeType::RawText => 0,
            _ => 0,
        }
    }
}

pub fn add_hashes(nodes: &Vec<RwSignal<PageNode>>, location: Vec<usize>, 
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
        let children = node.get().children;
        if !children.is_empty()  {
            add_hashes(&children, location, locations);
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

pub fn init_demo_page_data(cx: Scope) -> RwSignal<Page> {
    let mut page = PageNode::signal_from(cx, 
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
    let text_block_template = PageNode::from(
        "".into(), PageNodeType::TextBlock, HashMap::new(),
        Vec::new(), None, 0
    );
    for _ in 0..5 {
        {
            let h1_node_sig = create_rw_signal(cx, h1_template.clone());
            let mut new_child = raw_text_template.clone();
            new_child.parent = Some(h1_node_sig.clone());
            h1_node_sig.update(|n| {
                n.children.push(create_rw_signal(cx, new_child));
                n.parent = Some(page);
            });
            nodes.push(h1_node_sig);
        }

        {
            let parent = create_rw_signal(cx, text_block_template.clone());
            let child = create_rw_signal(cx, text_block_template.clone());
            let mut text_child = raw_text_template.clone();
            text_child.parent = Some(child.clone());
            child.update(|n| {
                n.children.push(create_rw_signal(cx, text_child));
                n.parent = Some(parent.clone());
            });
            parent.update(|n| {
                n.children.push(child);
                n.parent = Some(page);
            });
            nodes.push(parent);
        }
        {
            let parent = create_rw_signal(cx, text_block_template.clone());
            let child_1 = create_rw_signal(cx, text_block_template.clone());
            let mut text_child_1 = raw_text_template.clone();
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
    // ==init top/bot-elem==

    let top_node = get_top_block_node(&nodes);
    let top_hash = top_node.get().hash;
    let bot_node = get_bot_block_node(&nodes);
    let bot_hash = top_node.get().hash;

    Page::signal_from(cx,
        page,
        EdgeElem::signal_from(cx, top_hash, top_node, 0, 0),
        EdgeElem::signal_from(cx, bot_hash, bot_node, 0, 0),
        locations,
    )
}