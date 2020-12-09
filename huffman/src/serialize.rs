/* ********************************************************** *\

HF {
    magic: "hf00",
    encoding: u32, // 0=Raw, 1=DynamicTree, 2=FixedTree
    tree_nodes: vec![1,0,0,0,1,1,1,1], // 0=Parent, 1=Leaf
    tree_values: "obcde", // In nodes order
    data_size: u64, // Bytes
    data: vec![] // Encoded data, align 32?
    checksum?,
}

\* ********************************************************** */

use std::collections::VecDeque;
use std::hash::Hash;
use std::io;

use crate::traits::Serialize;
use crate::tree::{Tree, TreeNode};

const MAGIC: [u8; 4] = ['h' as u8, 'f' as u8, '0' as u8, '0' as u8];
const ALIGN_TREE: usize = 4;
const ALIGN_VALUES: usize = 4;
const ALIGN_DATA: usize = 4;

pub enum Encoding {
    Raw,
    DynamicTree,
    FixedTree,
}

fn align_diff(size: usize, align: usize) -> usize {
    (align - (size % align)) % align
}

pub fn write_magic<W: io::Write>(mut dst: W) -> io::Result<usize> {
    dst.write_all(&MAGIC).map(|_| MAGIC.len())
}

pub fn write_encoding<W: io::Write>(mut dst: W, encoding: Encoding) -> io::Result<usize> {
    let data: [u8; 4] = u32::to_le_bytes(match encoding {
        Encoding::Raw => 0,
        Encoding::DynamicTree => 1,
        Encoding::FixedTree => 2,
    });
    dst.write_all(&data).map(|_| data.len())
}

pub fn write_tree<W: io::Write, T: Serialize>(mut dst: W, tree: &Tree<T>) -> io::Result<usize>
where
    T: Clone + Eq + Hash,
{
    let mut values = Vec::with_capacity(tree.leaf_nodes());
    let nodes_size = write_tree_nodes(&mut dst, tree, &mut values)?;
    let values_size = write_tree_values(dst, &values)?;
    Ok(nodes_size + values_size)
}

pub fn write_tree_nodes<W: io::Write, T>(
    mut dst: W,
    tree: &Tree<T>,
    values: &mut Vec<T>,
) -> io::Result<usize>
where
    T: Clone + Eq + Hash,
{
    let mut nodes = VecDeque::with_capacity(tree.leaf_nodes());
    let mut data = [0u8; ALIGN_TREE];
    let mut data_byte = &mut data[0];
    let mut data_byte_index = 0usize;
    let mut data_bit_index = 0;
    nodes.push_back(&tree.root().0);
    nodes.push_back(&tree.root().1);
    // Write all nodes as bits in ALIGN_TREE sized chunks
    // Parent = 0, Leaf = 1
    while let Some(node) = nodes.pop_front() {
        match node {
            TreeNode::Parent(children) => {
                *data_byte &= !(1 << data_bit_index);
                nodes.push_back(&children.0);
                nodes.push_back(&children.1);
            }
            TreeNode::Leaf(value) => {
                *data_byte |= 1 << data_bit_index;
                values.push(value.clone());
            }
        }
        data_bit_index = (data_bit_index + 1) % 8;
        if data_bit_index == 0 {
            data_byte_index += 1;
            if data_byte_index % ALIGN_TREE == 0 {
                dst.write_all(&data)?;
            }
            data_byte = &mut data[data_byte_index % ALIGN_TREE];
        }
    }
    // Write bytes for ALIGN_TREE alignment
    if data_byte_index % ALIGN_TREE != 0 || data_bit_index != 0 {
        dst.write_all(&data)?;
        data_byte_index += align_diff(data_byte_index, ALIGN_TREE);
    }
    Ok(data_byte_index)
}

pub fn write_tree_values<W: io::Write, T: Serialize>(
    mut dst: W,
    values: &[T],
) -> io::Result<usize> {
    let mut written = 0usize;
    for v in values {
        written += v.serialize(&mut dst)?;
    }
    Ok(written)
}

pub fn write_data<W: io::Write>(mut dst: W, data: &[u8]) -> io::Result<usize> {
    if data.len() > u64::MAX as usize {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Data length cannot be represented with 64 bits",
        ));
    }
    let data_size = u64::to_le_bytes(data.len() as u64);
    let padding: Vec<u8> = (0..align_diff(data.len(), ALIGN_DATA))
        .map(|_| 0u8)
        .collect();
    dst.write_all(&data_size)?;
    dst.write_all(data)?;
    dst.write_all(&padding)?;
    Ok(data_size.len() + data.len() + padding.len())
}
