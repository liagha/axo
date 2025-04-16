use crate::axo_data::tree::{AvlNode, AvlTree, BinaryNode, BinarySearchTree, BinaryTree, BstNode, Node, Tree};
use core::fmt::{Display, Debug, Formatter, Result};

impl<T: Debug> Debug for Node<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {

        if !self.children.is_empty() {
            write!(f, "{:?} -> ", self.value)?;

            if self.children.len() == 1 {
                write!(f, "{:?}", self.children[0])
            } else {
                write!(f, "{:?}", self.children)
            }
        } else {
            write!(f, "{:?}", self.value)
        }
    }
}

impl<T: Display> Display for Node<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", self.value)?;
        if !self.children.is_empty() {
            write!(f, "(")?;
            for (i, child) in self.children.iter().enumerate() {
                if i > 0 {
                    write!(f, ", ")?;
                }
                write!(f, "{}", child)?;
            }
            write!(f, ")")?;
        }
        Ok(())
    }
}

impl<T: Display> Display for Tree<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match &self.root {
            Some(root) => write!(f, "{}", root),
            None => write!(f, "Empty"),
        }
    }
}

// Implement Debug if T is Debug
impl<T: Debug> Debug for Tree<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "Tree({:?})", self.root)
    }
}

// Implement Debug if T is Debug
impl<T: Debug> Debug for BinaryNode<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        f.debug_struct("BinaryNode")
            .field("value", &self.value)
            .field("left", &self.left)
            .field("right", &self.right)
            .finish()
    }
}

// Implement Debug if T is Debug
impl<T: Debug> Debug for BinaryTree<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        f.debug_struct("BinaryTree")
            .field("root", &self.root)
            .finish()
    }
}

// Implement Debug if T is Debug
impl<T: Ord + Debug> Debug for BstNode<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        f.debug_struct("BstNode")
            .field("value", &self.value)
            .field("left", &self.left)
            .field("right", &self.right)
            .finish()
    }
}

// Implement Debug for BinarySearchTree if T is Debug
impl<T: Ord + Debug> Debug for BinarySearchTree<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        f.debug_struct("BinarySearchTree")
            .field("root", &self.root)
            .finish()
    }
}

// Implement Debug if T is Debug
impl<T: Ord + Debug> Debug for AvlNode<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        f.debug_struct("AvlNode")
            .field("value", &self.value)
            .field("left", &self.left)
            .field("right", &self.right)
            .field("height", &self.height)
            .finish()
    }
}

// Implement Debug for AvlTree if T is Debug
impl<T: Ord + Debug> Debug for AvlTree<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        f.debug_struct("AvlTree")
            .field("root", &self.root)
            .finish()
    }
}
