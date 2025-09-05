// most of the code here is inspired by the helix editor
use std::io::Write;

use ratatui::TerminalOptions;
use ratatui::Viewport;
use ratatui::prelude::*;

use crate::views::View;

slotmap::new_key_type! {
    pub struct ViewId;
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Layout {
    Vertical,
    Horizontal,
}

struct Node {
    parent: ViewId,
    data: NodeData,
}

impl Node {
    fn view(view: Box<dyn View>) -> Node {
        Node {
            parent: ViewId::default(),
            data: NodeData::View(NodeViewData::new(view)),
        }
    }

    fn container(layout: Layout) -> Node {
        Node {
            parent: ViewId::default(),
            data: NodeData::Container(NodeContainerData::new(layout)),
        }
    }
}

enum NodeData {
    View(NodeViewData),
    Container(NodeContainerData),
}

struct NodeViewData {
    area: Rect,
    view: Box<dyn View>,
}

impl NodeViewData {
    pub fn new(view: Box<dyn View>) -> Self {
        Self {
            area: Rect::default(),
            view,
        }
    }
}

struct NodeContainerData {
    childs: Vec<ViewId>,
    area: Rect,
    layout: Layout,
}

impl NodeContainerData {
    fn new(layout: Layout) -> Self {
        Self {
            childs: Vec::new(),
            area: Rect::default(),
            layout,
        }
    }
}

struct Tree {
    nodes: slotmap::HopSlotMap<ViewId, Node>,
    root: ViewId,
    focuse: ViewId,
}

impl Tree {
    pub fn new(initial_size: (u16, u16)) -> Self {
        let mut nodes = slotmap::HopSlotMap::with_key();
        let ctr = NodeContainerData {
            childs: Vec::new(),
            area: Rect::new(0, 0, initial_size.0, initial_size.1),
            layout: Layout::Horizontal,
        };
        let root = nodes.insert(Node {
            parent: ViewId::default(),
            data: NodeData::Container(ctr),
        });
        nodes[root].parent = root;

        Tree {
            nodes,
            root,
            focuse: root,
        }
    }

    fn push(&mut self, view: Box<dyn View>) -> ViewId {
        let parent = self.nodes[self.focuse].parent;
        let node = self.nodes.insert(Node {
            parent,
            data: NodeData::View(NodeViewData::new(view)),
        });

        match self.nodes[parent] {
            Node {
                data: NodeData::Container(ref mut ctr),
                ..
            } => {
                // the pushed view need to be inserted below/after current
                // focuse view, if there are no childs it means the index is `0`, if there
                // are childs, we look for the focused view position and add `1`
                let position = if ctr.childs.is_empty() {
                    0
                } else {
                    ctr.childs
                        .iter()
                        .position(|view_id| *view_id == self.focuse)
                        .unwrap()
                        + 1
                };
                ctr.childs.insert(position, node);

                let area = ctr.area;
                self.recalculate(parent, area);
            }
            _ => panic!("unexpected, node parent is not a container"),
        };

        self.focuse = node;
        node
    }

    fn split(&mut self, view: Box<dyn View>, layout: Layout) {
        let parent = self.nodes[self.focuse].parent;
        let node = self.nodes.insert(Node::view(view));

        match self.nodes[parent] {
            Node {
                data: NodeData::Container(ref mut ctr),
                ..
            } => {
                // NOTE: area is defined here because borrow checker nonsense
                let area = ctr.area;

                if ctr.layout == layout {
                    let position = if ctr.childs.is_empty() {
                        0
                    } else {
                        ctr.childs
                            .iter()
                            .position(|view_id| *view_id == self.focuse)
                            .unwrap()
                            + 1
                    };
                    ctr.childs.insert(position, node);
                    self.nodes[node].parent = parent;
                } else {
                    // if the layout is different from the current
                    // container, we need to create a new container with the new layout
                    // and assign it as a child for the current container, the view will be a child
                    // of the new created container
                    let split_ctr = {
                        let mut ctr = NodeContainerData::new(layout);
                        ctr.childs.push(node);
                        ctr
                    };
                    let split = Node {
                        parent,
                        data: NodeData::Container(split_ctr),
                    };

                    let split = self.nodes.insert(split);
                    self.nodes[node].parent = split;
                };

                self.focuse = node;
                self.recalculate(parent, area);
            }
            _ => unreachable!(),
        };
    }

    /// remove the current focuse view
    fn remove(&mut self) {
        todo!();
    }

    #[inline]
    fn resize(&mut self, size: (u16, u16)) {
        self.recalculate(self.root, Rect::new(0, 0, size.0, size.1));
    }

    /// recalculate areas from the given view node with the new area
    fn recalculate(&mut self, root: ViewId, area: Rect) {
        let mut stack = vec![(root, area)];

        while let Some((node, area)) = stack.pop() {
            match &mut self.nodes[node] {
                Node {
                    data: NodeData::Container(ctr),
                    ..
                } => match ctr.layout {
                    Layout::Horizontal => {
                        ctr.area = area;
                        let height = ctr.area.height / ctr.childs.len() as u16;
                        let mut offset = ctr.area.y;

                        for (i, child) in ctr.childs.iter().enumerate() {
                            let area = Rect::new(0, offset, ctr.area.width, height);
                            offset += height;
                            stack.push((*child, area))
                        }
                    }
                    Layout::Vertical => {
                        todo!()
                    }
                },
                Node {
                    data: NodeData::View(view),
                    ..
                } => view.area = area,
            }
        }
    }
}

impl<'a> IntoIterator for &'a Tree {
    type IntoIter = TreeIter<'a>;
    type Item = <TreeIter<'a> as Iterator>::Item;

    fn into_iter(self) -> Self::IntoIter {
        TreeIter {
            tree: self,
            stack: vec![self.root],
        }
    }
}

struct TreeIter<'a> {
    tree: &'a Tree,
    stack: Vec<ViewId>,
}

impl<'a> Iterator for TreeIter<'a> {
    type Item = &'a NodeViewData;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let view = self.stack.pop()?;
            let node = &self.tree.nodes[view];

            match &node.data {
                NodeData::View(view) => return Some(view),
                NodeData::Container(ctr) => {
                    self.stack.extend(&ctr.childs);
                }
            }
        }
    }
}

/// the compositor is responsible to display and render requested
/// views to the terminal
pub struct Compositor<W: Write> {
    tree: Tree,
    terminal: Terminal<CrosstermBackend<W>>,
}

impl<W: Write> Compositor<W> {
    pub fn new(stdout: W, size: (u16, u16)) -> Self {
        let terminal = Terminal::with_options(
            CrosstermBackend::new(stdout),
            TerminalOptions {
                viewport: Viewport::Fixed(Rect::new(0, 0, size.0, size.1)),
            },
        )
        .unwrap();

        Self {
            terminal,
            tree: Tree::new(size),
        }
    }

    #[inline]
    pub fn split_view(&mut self, view: Box<dyn View>, layout: Layout) {
        self.tree.split(view, layout);
    }

    /// resize the compositor viewport
    #[inline]
    pub fn resize(&mut self, size: (u16, u16)) {
        self.terminal
            .resize(Rect::new(0, 0, size.0, size.1))
            .unwrap();
        self.tree.resize(size);
    }

    pub fn render(&mut self) {
        self.terminal
            .draw(|frame| {
                for view in &self.tree {
                    view.view.render(view.area, frame.buffer_mut());
                }
            })
            .unwrap();
    }
}
