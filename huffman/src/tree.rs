use std::cmp::Ordering;
use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap};
use std::hash::Hash;

#[derive(Clone, Copy)]
struct Count<T> {
    pub element: T,
    pub count: usize,
}

impl<T> Ord for Count<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.count.cmp(&other.count)
    }
}

impl<T> PartialOrd for Count<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.count.partial_cmp(&other.count)
    }
}

impl<T> PartialEq for Count<T> {
    fn eq(&self, other: &Self) -> bool {
        self.count == other.count
    }
}

impl<T> Eq for Count<T> {}

fn count_occurrences<T>(iter: impl Iterator<Item = T>) -> HashMap<T, usize>
where
    T: Eq + Hash,
{
    let mut counts = HashMap::with_capacity(256);
    for element in iter {
        counts.entry(element).and_modify(|c| *c += 1).or_insert(1);
    }
    counts
}

#[derive(Debug)]
pub enum TreeNode<T>
where
    T: Clone + Eq + Hash,
{
    Parent(Box<(TreeNode<T>, TreeNode<T>)>),
    Leaf(T),
}

impl<T> PartialEq for TreeNode<T>
where
    T: Clone + Eq + Hash,
{
    fn eq(&self, rhs: &Self) -> bool {
        match (self, rhs) {
            (TreeNode::Parent(nodes0), TreeNode::Parent(nodes1)) => nodes0.eq(nodes1),
            (TreeNode::Leaf(value0), TreeNode::Leaf(value1)) => value0.eq(value1),
            _ => false,
        }
    }
}

pub type TreeParentNode<T> = (TreeNode<T>, TreeNode<T>);

#[derive(Debug)]
pub struct Tree<T>
where
    T: Clone + Eq + Hash,
{
    root: TreeParentNode<T>,
    max_depth: usize,
    leaf_nodes: usize,
}

impl<T> PartialEq for Tree<T>
where
    T: Clone + Eq + Hash,
{
    fn eq(&self, rhs: &Self) -> bool {
        self.max_depth.eq(&rhs.max_depth)
            && self.leaf_nodes.eq(&rhs.leaf_nodes)
            && self.root.eq(&rhs.root)
    }
}

impl<T> Tree<T>
where
    T: Clone + Eq + Hash,
{
    pub fn new(root: TreeParentNode<T>, max_depth: usize, leaf_nodes: usize) -> Self {
        Tree {
            root,
            max_depth,
            leaf_nodes,
        }
    }
    pub fn build(iter: impl Iterator<Item = T>) -> Option<Self> {
        // Count occurrences of each word and store a tuple containing the tree node and the max_depth for child nodes in a min-heap
        let occurrences = count_occurrences(iter);
        let leaf_nodes = occurrences.len();
        let mut heap = BinaryHeap::with_capacity(occurrences.len());
        for (element, count) in occurrences {
            heap.push(Reverse(Count {
                element: (TreeNode::Leaf(element), 0usize),
                count,
            }))
        }
        // Combine nodes with the lowest counts until there is a single root node, increase max_depth for each combination
        while heap.len() > 1 {
            let (left, right) = (heap.pop().unwrap().0, heap.pop().unwrap().0);
            let new_node = TreeNode::Parent(Box::new((left.element.0, right.element.0)));
            heap.push(Reverse(Count {
                element: (new_node, left.element.1.max(right.element.1) + 1),
                count: left.count + right.count,
            }));
        }
        // Return resulting huffman tree
        if let Some(Reverse(root)) = heap.pop() {
            let root_node = root.element.0;
            let max_depth = root.element.1;
            Some(match root_node {
                TreeNode::Parent(nodes) => Tree {
                    root: *nodes,
                    max_depth,
                    leaf_nodes,
                },
                TreeNode::Leaf(value) => Tree {
                    root: (TreeNode::Leaf(value.clone()), TreeNode::Leaf(value)),
                    max_depth: 1,
                    leaf_nodes: 2,
                },
            })
        } else {
            None
        }
    }
    pub fn traverse<D: Clone>(&self, data: D, func: &mut dyn FnMut(bool, D, &TreeNode<T>) -> D) {
        Tree::traverse_nodes(&self.root.0, false, data.clone(), func);
        Tree::traverse_nodes(&self.root.1, true, data, func);
    }
    pub fn get_code_depths(&self) -> HashMap<T, usize> {
        let mut depths = HashMap::with_capacity(self.leaf_nodes);
        self.traverse(1usize, &mut |_, depth, node| {
            if let TreeNode::Leaf(value) = node {
                depths.insert(value.clone(), depth);
            }
            depth + 1
        });
        depths
    }
    pub fn build_codes_table(&self) -> HashMap<T, Box<[bool]>> {
        let mut table = HashMap::with_capacity(self.leaf_nodes);
        let initial_code = Vec::with_capacity(self.max_depth);
        self.traverse(initial_code, &mut |direction, code, node| {
            let mut code = code;
            code.push(direction);
            if let TreeNode::Leaf(value) = node {
                table.insert(value.clone(), code.into_boxed_slice());
                vec![]
            } else {
                code
            }
        });
        table
    }
    pub fn root(&self) -> &TreeParentNode<T> {
        &self.root
    }
    pub fn max_depth(&self) -> usize {
        self.max_depth
    }
    pub fn leaf_nodes(&self) -> usize {
        self.leaf_nodes
    }
    pub fn traverse_nodes<D: Clone>(
        root: &TreeNode<T>,
        direction: bool,
        data: D,
        func: &mut dyn FnMut(bool, D, &TreeNode<T>) -> D,
    ) {
        let data = func(direction, data, root);
        if let TreeNode::Parent(child) = root {
            Tree::traverse_nodes(&child.0, false, data.clone(), func);
            Tree::traverse_nodes(&child.1, true, data, func);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Tree, TreeNode};
    use std::collections::HashMap;
    use std::iter::FromIterator;
    #[test]
    fn build_tree_empty_iter() {
        let empty: [u8; 0] = [];
        assert!(Tree::build(empty.iter()).is_none());
    }
    #[test]
    fn build_tree_single_element() {
        let tree = Tree::build([0].iter().map(|e| *e)).unwrap();
        assert_eq!(tree.max_depth(), 1);
        assert_eq!(tree.leaf_nodes(), 2);
        assert_eq!(
            tree.get_code_depths(),
            HashMap::from_iter([(0, 1)].iter().map(|e| *e))
        )
    }
    #[test]
    fn build_tree_chars() {
        let text = "obocodoe";
        let mut depths = HashMap::with_capacity(5);
        depths.insert('o', 1);
        depths.insert('b', 3);
        depths.insert('c', 3);
        depths.insert('d', 3);
        depths.insert('e', 3);
        let tree = Tree::build(text.chars()).unwrap();
        assert_eq!(tree.max_depth(), 3);
        assert_eq!(tree.leaf_nodes(), 5);
        assert_eq!(tree.get_code_depths(), depths);
    }
    #[test]
    fn build_tree_ints() {
        let words = [0, 0, 1, 5, 4, 3, 3, 3, 3, 0, 2];
        let mut depths = HashMap::with_capacity(6);
        depths.insert(0, 2);
        depths.insert(1, 3);
        depths.insert(2, 3);
        depths.insert(3, 2);
        depths.insert(4, 3);
        depths.insert(5, 3);
        let tree = Tree::build(words.iter().map(|e| *e)).unwrap();
        assert_eq!(tree.max_depth(), 3);
        assert_eq!(tree.leaf_nodes(), 6);
        assert_eq!(
            tree.get_code_depths(),
            depths
        );
    }
    #[test]
    fn build_codes_table() {
        let tree = Tree::new(
            (
                TreeNode::Leaf('o'), // 0
                TreeNode::Parent(Box::new((
                    TreeNode::Parent(Box::new((
                        TreeNode::Leaf('b'), // 100
                        TreeNode::Leaf('c'), // 101
                    ))),
                    TreeNode::Parent(Box::new((
                        TreeNode::Leaf('d'), // 110
                        TreeNode::Leaf('e'), // 111
                    ))),
                ))),
            ),
            3,
            5,
        );
        let codes = tree.build_codes_table();
        assert_eq!(codes[&'o'].as_ref(), &[false]);
        assert_eq!(codes[&'b'].as_ref(), &[true, false, false]);
        assert_eq!(codes[&'c'].as_ref(), &[true, false, true]);
        assert_eq!(codes[&'d'].as_ref(), &[true, true, false]);
        assert_eq!(codes[&'e'].as_ref(), &[true, true, true]);
    }
}
