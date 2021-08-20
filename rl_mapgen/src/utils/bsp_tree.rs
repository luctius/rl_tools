use std::fmt;

type BspId = usize;

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
pub enum NodeDir {
    Left,
    Right,
}
impl NodeDir {
    fn to_idx(self) -> usize {
        match self {
            NodeDir::Left => 0,
            NodeDir::Right => 1,
        }
    }
}

pub trait Split
    where Self: Sized, {
    type Context;
    fn split(&mut self, context: &Self::Context) -> Option<(Self, Self)>;
}

#[derive(Debug)]
pub struct BspTree<S>
    where S: Split, {
    pub data:  Vec<S>,
    pub nodes: Vec<BspNode>,
    pub depth: usize,
}
impl<S> BspTree<S> where S: Split, {
    pub fn new(data: S) -> BspTree<S> {
        BspTree { depth: 0,
                  data:  vec![data],
                  nodes: vec![BspNode { parent:   0,
                                        id:       0,
                                        sibbling: None,
                                        depth:    0,
                                        children: [None, None], }], }
    }

    pub fn add_node(&mut self, node: BspNode, data: S) -> Option<BspNode> {
        for (idx, n) in self.nodes[node.id].children.iter().enumerate() {
            if n.is_some() {
                continue;
            } else {
                let new_idx = self.nodes.len();
                let new_node = BspNode { parent:   node.id,
                                         id:       new_idx,
                                         sibbling: None,
                                         depth:    node.depth + 1,
                                         children: [None, None], };

                if node.depth + 1 > self.depth {
                    self.depth = node.depth + 1;
                }
                self.nodes.push(new_node);
                self.data.push(data);
                self.nodes[node.id].children[idx] = Some(new_idx);
                return Some(new_node);
            }
        }
        None
    }

    pub fn link_sibblings(&mut self, node1: BspNode, node2: BspNode) {
        if node1.parent == node2.parent && node1.id != node2.id {
            self.nodes[node1.id].sibbling = Some(node2.id);
            self.nodes[node2.id].sibbling = Some(node1.id);
        }
    }

    pub fn get_parent_data(&self, node: BspNode) -> Option<&S> {
        self.data.get(node.parent)
    }

    pub fn get_root(&self) -> BspNode {
        self.nodes[0]
    }

    pub fn get_data(&self, node: BspNode) -> Option<&S> {
        self.data.get(node.id)
    }

    pub fn get_data_mut(&mut self, node: BspNode) -> Option<&mut S> {
        self.data.get_mut(node.id)
    }

    pub fn get_tree_depth(&self) -> usize {
        self.depth
    }

    pub fn iter(&self, depth: usize) -> BspIter<S> {
        BspIter { pos: 0, tree: self, depth }
    }

    pub fn leaf_iter(&self) -> BspLeafIter<S> {
        BspLeafIter { pos: 0, tree: self, }
    }
}

#[derive(Debug)]
pub struct BspIter<'a, S>
    where S: Split, {
    pos:   usize,
    depth: usize,
    tree:  &'a BspTree<S>,
}
impl<'a, S> Iterator for BspIter<'a, S> where S: Split, {
    type Item = BspNode;

    fn next(&mut self) -> Option<Self::Item> {
        while self.pos < self.tree.nodes.len() {
            self.pos += 1;
            if self.tree.nodes[self.pos - 1].depth == self.depth {
                return Some(self.tree.nodes[self.pos - 1]);
            }
        }
        None
    }
}

#[derive(Debug)]
pub struct BspLeafIter<'a, S>
    where S: Split, {
    pos:  usize,
    tree: &'a BspTree<S>,
}
impl<'a, S> Iterator for BspLeafIter<'a, S> where S: Split, {
    type Item = BspNode;

    fn next(&mut self) -> Option<Self::Item> {
        'outer: while self.pos < self.tree.nodes.len() {
            self.pos += 1;

            for n in self.tree.nodes[self.pos - 1].children.iter() {
                if n.is_some() {
                    continue 'outer;
                }
            }
            return Some(self.tree.nodes[self.pos - 1]);
        }
        None
    }
}

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
pub struct BspNode {
    parent:   BspId,
    pub id:   BspId,
    sibbling: Option<BspId>,
    depth:    usize,
    children: [Option<BspId>; 2],
}
impl BspNode {
    pub fn get_sibbling<S>(&self, tree: &BspTree<S>) -> Option<BspNode>
        where S: Split, {
        if let Some(n) = self.sibbling {
            tree.nodes.get(n).cloned()
        } else {
            None
        }
    }

    pub fn get_child<S>(&self, dir: NodeDir, tree: &BspTree<S>) -> Option<BspNode>
        where S: Split, {
        if let Some(n) = self.children[dir.to_idx()] {
            tree.nodes.get(n).cloned()
        } else {
            None
        }
    }

    pub fn split<S>(&mut self, context: &<S as Split>::Context, tree: &mut BspTree<S>) -> Option<(BspNode, BspNode)>
        where S: Split, {
        for i in &self.children {
            if i.is_some() {
                return None;
            }
        }

        if let Some((c1, c2)) = if let Some(d) = tree.get_data_mut(*self) {
            d.split(context)
        } else {
            return None;
        } {
            let kids = tree.add_node(*self, c1).and_then(|n1| tree.add_node(*self, c2).map(|n2| (n1, n2)));

            if let Some((n1, n2)) = kids {
                tree.link_sibblings(n1, n2);
                self.children[NodeDir::Left.to_idx()] = Some(n1.id);
                self.children[NodeDir::Right.to_idx()] = Some(n2.id);
            }
            kids
        } else {
            None
        }
    }
}
impl fmt::Display for BspNode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({})", self.id)
    }
}
