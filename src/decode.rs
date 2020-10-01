use crate::tree::{Tree, TreeNode, TreeParentNode};
use std::hash::Hash;

pub struct Decoder<'a, T: Clone + Eq + Hash, I: Iterator<Item = bool>> {
    root_node: &'a TreeParentNode<T>,
    current_node: &'a TreeParentNode<T>,
    source: I,
}

impl<'a, T: Clone + Eq + Hash, I: Iterator<Item = bool>> Decoder<'a, T, I> {
    pub fn new(tree: &'a Tree<T>, source: I) -> Self {
        let root_node = tree.root();
        Decoder {
            root_node,
            current_node: root_node,
            source,
        }
    }
}

impl<'a, T: Clone + Eq + Hash, I: Iterator<Item = bool>> Iterator for Decoder<'a, T, I> {
    type Item = &'a T;
    fn next(&mut self) -> Option<Self::Item> {
        // Read bits from source
        while let Some(bit) = self.source.next() {
            // Traverse tree according to read bit
            let next_node = if bit {
                &self.current_node.1
            } else {
                &self.current_node.0
            };
            match next_node {
                TreeNode::Parent(nodes) => {
                    // Set current_node to next parent node
                    self.current_node = nodes.as_ref();
                }
                TreeNode::Leaf(value) => {
                    // Leaf node reached, reset current_node and yield decoded value
                    self.current_node = self.root_node;
                    return Some(value);
                }
            }
        }
        // End of source reached
        if std::ptr::eq(self.current_node, self.root_node) {
            None
        } else {
            // FIXME: Replace panic with something?
            panic!("Some trailing bits where partially processed, premature ending of the input stream?")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Decoder, Tree, TreeNode};
    #[test]
    fn decode_empty() {
        let tree = Tree::new((TreeNode::Leaf(0), TreeNode::Leaf(1)), 1, 2);
        let mut decoder = Decoder::new(&tree, [].iter().map(|e| *e));
        assert_eq!(decoder.next(), None);
    }
    #[test]
    fn decode_chars() {
        // "decoded"
        let encoded = "110_111_101_0_110_111_110".chars().filter_map(|c| match c {
            '0' => Some(false),
            '1' => Some(true),
            _ => None,
        });
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
        let decoder = Decoder::new(&tree, encoded);
        assert_eq!(decoder.collect::<String>(), "decoded");
    }
    #[test]
    fn decode_partial() {
        unimplemented!();
    }
}
