#![allow(dead_code)]

extern crate alloc;

use alloc::vec::Vec;
use alloc::boxed::Box;
use core::cmp::Ordering;
use alloc::string::String;

/// A tree data structure.
#[derive(Eq, Hash, PartialEq)]
pub struct Tree<T> {
    /// The root node of the tree
    pub root: Option<Node<T>>,
}

/// A node in a tree.
#[derive(Clone, Eq, Hash, PartialEq)]
pub struct Node<T> {
    /// The value stored in this node
    pub value: T,
    /// Child nodes
    pub children: Vec<Node<T>>,
}

impl<T> Node<T> {
    /// Creates a new node with the given value and no children.
    pub fn new(value: T) -> Self {
        Node {
            value,
            children: Vec::new(),
        }
    }

    /// Creates a new node with the given value and children.
    pub fn with_children(value: T, children: Vec<Node<T>>) -> Self {
        Node { value, children }
    }

    /// Adds a child node to this node.
    pub fn add_child(&mut self, child: Node<T>) {
        self.children.push(child);
    }

    /// Creates and adds a child node with the given value to this node.
    pub fn add_value(&mut self, value: T) {
        self.children.push(Node::new(value));
    }

    /// Returns the number of direct children this node has.
    pub fn child_count(&self) -> usize {
        self.children.len()
    }

    /// Returns whether this node has any children.
    pub fn has_children(&self) -> bool {
        !self.children.is_empty()
    }

    /// Returns a reference to a child at the given index, or None if out of bounds.
    pub fn get_child(&self, index: usize) -> Option<&Node<T>> {
        self.children.get(index)
    }

    /// Returns a mutable reference to a child at the given index, or None if out of bounds.
    pub fn get_child_mut(&mut self, index: usize) -> Option<&mut Node<T>> {
        self.children.get_mut(index)
    }

    /// Removes and returns the child at the specified index.
    pub fn remove_child(&mut self, index: usize) -> Option<Node<T>> {
        if index < self.children.len() {
            Some(self.children.remove(index))
        } else {
            None
        }
    }

    /// Traverses the tree in pre-order (current node, then children from left to right).
    pub fn traverse_pre_order<F>(&self, f: &mut F)
    where
        F: FnMut(&T),
    {
        f(&self.value);
        for child in &self.children {
            child.traverse_pre_order(f);
        }
    }

    /// Traverses the tree in post-order (children from left to right, then current node).
    pub fn traverse_post_order<F>(&self, f: &mut F)
    where
        F: FnMut(&T),
    {
        for child in &self.children {
            child.traverse_post_order(f);
        }
        f(&self.value);
    }

    /// Traverses the tree level by level (breadth-first).
    pub fn traverse_breadth_first<F>(&self, mut f: F)
    where
        F: FnMut(&T),
    {
        // Create a queue for BFS
        let mut queue = Vec::new();
        queue.push(self);

        while !queue.is_empty() {
            // Dequeue a node
            let node = queue.remove(0);

            // Process this node
            f(&node.value);

            // Enqueue all children
            for child in &node.children {
                queue.push(child);
            }
        }
    }

    /// Returns the total number of nodes in the tree (this node plus all descendants).
    pub fn size(&self) -> usize {
        let mut count = 1; // Count this node
        for child in &self.children {
            count += child.size();
        }
        count
    }

    /// Returns the height of the tree (longest path from this node to a leaf).
    pub fn height(&self) -> usize {
        if self.children.is_empty() {
            0
        } else {
            let mut max_height = 0;
            for child in &self.children {
                let height = child.height();
                if height > max_height {
                    max_height = height;
                }
            }
            max_height + 1
        }
    }

    /// Maps each node's value to a new value using the provided function.
    pub fn map<U, F>(&self, f: F) -> Node<U>
    where
        F: Fn(&T) -> U + Copy,
    {
        let mapped_children: Vec<Node<U>> = self.children
            .iter()
            .map(|child| child.map(f))
            .collect();

        Node {
            value: f(&self.value),
            children: mapped_children,
        }
    }

    /// Returns true if any node in the tree satisfies the predicate.
    pub fn any<F>(&self, f: &F) -> bool
    where
        F: Fn(&T) -> bool,
    {
        if f(&self.value) {
            return true;
        }

        for child in &self.children {
            if child.any(f) {
                return true;
            }
        }

        false
    }

    /// Returns true if all nodes in the tree satisfy the predicate.
    pub fn all<F>(&self, f: &F) -> bool
    where
        F: Fn(&T) -> bool,
    {
        if !f(&self.value) {
            return false;
        }

        for child in &self.children {
            if !child.all(f) {
                return false;
            }
        }

        true
    }

    /// Finds the first node that matches the predicate in a pre-order traversal.
    pub fn find<F>(&self, predicate: F) -> Option<&Node<T>>
    where
        F: Fn(&T) -> bool,
    {
        if predicate(&self.value) {
            return Some(self);
        }

        for child in &self.children {
            if let Some(found) = child.find(&predicate) {
                return Some(found);
            }
        }

        None
    }

    /// Finds the first node that matches the predicate in a pre-order traversal, returning mutable reference.
    pub fn find_mut<F>(&mut self, predicate: F) -> Option<&mut Node<T>>
    where
        F: Fn(&T) -> bool,
    {
        if predicate(&self.value) {
            return Some(self);
        }

        for child in &mut self.children {
            if let Some(found) = child.find_mut(&predicate) {
                return Some(found);
            }
        }

        None
    }

    /// Performs a depth-first search for a path to a node that satisfies the given predicate.
    pub fn find_path<F>(&self, predicate: F) -> Option<Vec<usize>>
    where
        F: Fn(&T) -> bool + Copy,
    {
        if predicate(&self.value) {
            return Some(Vec::new());
        }

        for (i, child) in self.children.iter().enumerate() {
            if let Some(mut path) = child.find_path(predicate) {
                path.insert(0, i);
                return Some(path);
            }
        }

        None
    }

    /// Returns a reference to the node at the specified path.
    pub fn node_at_path(&self, path: &[usize]) -> Option<&Node<T>> {
        let mut current = self;

        for &index in path {
            match current.children.get(index) {
                Some(child) => current = child,
                None => return None,
            }
        }

        Some(current)
    }

    /// Returns a mutable reference to the node at the specified path.
    pub fn node_at_path_mut(&mut self, path: &[usize]) -> Option<&mut Node<T>> {
        let mut current = self;

        for &index in path {
            match current.children.get_mut(index) {
                Some(child) => current = child,
                None => return None,
            }
        }

        Some(current)
    }

    /// Fold the tree into a single value, starting from the leaves and working up.
    pub fn fold<B: Clone, F>(&self, init: B, f: F) -> B
    where
        F: Fn(B, &T, Vec<B>) -> B,
    {
        let child_results: Vec<B> = self.children
            .iter()
            .map(|child| child.fold(init.clone(), &f))
            .collect();

        f(init, &self.value, child_results)
    }
}

impl<T> Tree<T> {
    /// Creates a new empty tree
    pub fn new() -> Self {
        Tree { root: None }
    }

    /// Creates a new tree with the given root value
    pub fn with_root(value: T) -> Self {
        Tree {
            root: Some(Node::new(value)),
        }
    }

    /// Creates a new tree with an existing node as the root
    pub fn with_root_node(node: Node<T>) -> Self {
        Tree { root: Some(node) }
    }

    /// Returns true if the tree is empty (has no root)
    pub fn is_empty(&self) -> bool {
        self.root.is_none()
    }

    /// Returns a reference to the root node if it exists
    pub fn root(&self) -> Option<&Node<T>> {
        self.root.as_ref()
    }

    /// Returns a mutable reference to the root node if it exists
    pub fn root_mut(&mut self) -> Option<&mut Node<T>> {
        self.root.as_mut()
    }

    /// Sets the root of the tree to the given node
    pub fn set_root(&mut self, node: Node<T>) {
        self.root = Some(node);
    }

    /// Adds a child with the given value to the root node
    pub fn add_child(&mut self, value: T) -> Result<(), &'static str> {
        match self.root.as_mut() {
            Some(root) => {
                root.add_value(value);
                Ok(())
            }
            None => Err("Cannot add child to an empty tree"),
        }
    }

    /// Adds a child node to the root node
    pub fn add_child_node(&mut self, node: Node<T>) -> Result<(), &'static str> {
        match self.root.as_mut() {
            Some(root) => {
                root.add_child(node);
                Ok(())
            }
            None => Err("Cannot add child to an empty tree"),
        }
    }

    /// Returns the size of the tree (total number of nodes)
    pub fn size(&self) -> usize {
        match &self.root {
            Some(root) => root.size(),
            None => 0,
        }
    }

    /// Returns the height of the tree
    pub fn height(&self) -> usize {
        match &self.root {
            Some(root) => root.height(),
            None => 0,
        }
    }

    /// Maps each node's value to a new value using the provided function
    pub fn map<U, F>(&self, f: F) -> Tree<U>
    where
        F: Fn(&T) -> U + Copy,
    {
        match &self.root {
            Some(root) => Tree {
                root: Some(root.map(f)),
            },
            None => Tree { root: None },
        }
    }

    /// Traverses the tree in pre-order (root, then children from left to right)
    pub fn traverse_pre_order<F>(&self, mut f: F)
    where
        F: FnMut(&T),
    {
        if let Some(root) = &self.root {
            root.traverse_pre_order(&mut f);
        }
    }

    /// Traverses the tree in post-order (children from left to right, then root)
    pub fn traverse_post_order<F>(&self, mut f: F)
    where
        F: FnMut(&T),
    {
        if let Some(root) = &self.root {
            root.traverse_post_order(&mut f);
        }
    }

    /// Traverses the tree level by level (breadth-first)
    pub fn traverse_breadth_first<F>(&self, f: F)
    where
        F: FnMut(&T),
    {
        if let Some(root) = &self.root {
            root.traverse_breadth_first(f);
        }
    }

    /// Finds the first node that matches the predicate in a pre-order traversal
    pub fn find<F>(&self, predicate: F) -> Option<&Node<T>>
    where
        F: Fn(&T) -> bool,
    {
        match &self.root {
            Some(root) => root.find(predicate),
            None => None,
        }
    }

    /// Finds the first node that matches the predicate in a pre-order traversal, returning mutable reference
    pub fn find_mut<F>(&mut self, predicate: F) -> Option<&mut Node<T>>
    where
        F: Fn(&T) -> bool,
    {
        match &mut self.root {
            Some(root) => root.find_mut(predicate),
            None => None,
        }
    }

    /// Returns a reference to the node at the specified path
    pub fn node_at_path(&self, path: &[usize]) -> Option<&Node<T>> {
        match &self.root {
            Some(root) => root.node_at_path(path),
            None => None,
        }
    }

    /// Returns a mutable reference to the node at the specified path
    pub fn node_at_path_mut(&mut self, path: &[usize]) -> Option<&mut Node<T>> {
        match &mut self.root {
            Some(root) => root.node_at_path_mut(path),
            None => None,
        }
    }
}

// Implement Clone if T is Clone
impl<T: Clone> Clone for Tree<T> {
    fn clone(&self) -> Self {
        Tree {
            root: self.root.clone(),
        }
    }
}



// Implement Default
impl<T> Default for Tree<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// Binary tree node, which has at most two children: left and right.
pub struct BinaryNode<T> {
    /// The data stored in this node
    pub value: T,
    /// The left child of this node
    pub left: Option<Box<BinaryNode<T>>>,
    /// The right child of this node
    pub right: Option<Box<BinaryNode<T>>>,
}

impl<T> BinaryNode<T> {
    /// Creates a new binary node with the given value and no children.
    pub fn new(value: T) -> Self {
        BinaryNode {
            value,
            left: None,
            right: None,
        }
    }

    /// Creates a new binary node with the given value and children.
    pub fn with_children(
        value: T,
        left: Option<Box<BinaryNode<T>>>,
        right: Option<Box<BinaryNode<T>>>,
    ) -> Self {
        BinaryNode { value, left, right }
    }

    /// Sets the left child of this node.
    pub fn set_left(&mut self, node: BinaryNode<T>) {
        self.left = Some(Box::new(node));
    }

    /// Sets the right child of this node.
    pub fn set_right(&mut self, node: BinaryNode<T>) {
        self.right = Some(Box::new(node));
    }

    /// Returns true if this node has a left child.
    pub fn has_left(&self) -> bool {
        self.left.is_some()
    }

    /// Returns true if this node has a right child.
    pub fn has_right(&self) -> bool {
        self.right.is_some()
    }

    /// Returns true if this node is a leaf (has no children).
    pub fn is_leaf(&self) -> bool {
        self.left.is_none() && self.right.is_none()
    }

    /// Traverses the binary tree in pre-order (current node, then left, then right).
    pub fn traverse_pre_order<F>(&self, f: &mut F)
    where
        F: FnMut(&T),
    {
        f(&self.value);
        if let Some(left) = &self.left {
            left.traverse_pre_order(f);
        }
        if let Some(right) = &self.right {
            right.traverse_pre_order(f);
        }
    }

    /// Traverses the binary tree in-order (left, then current node, then right).
    pub fn traverse_in_order<F>(&self, f: &mut F)
    where
        F: FnMut(&T),
    {
        if let Some(left) = &self.left {
            left.traverse_in_order(f);
        }
        f(&self.value);
        if let Some(right) = &self.right {
            right.traverse_in_order(f);
        }
    }

    /// Traverses the binary tree in post-order (left, then right, then current node).
    pub fn traverse_post_order<F>(&self, f: &mut F)
    where
        F: FnMut(&T),
    {
        if let Some(left) = &self.left {
            left.traverse_post_order(f);
        }
        if let Some(right) = &self.right {
            right.traverse_post_order(f);
        }
        f(&self.value);
    }

    /// Returns the total number of nodes in the binary tree (this node plus all descendants).
    pub fn size(&self) -> usize {
        let mut count = 1; // Count this node
        if let Some(left) = &self.left {
            count += left.size();
        }
        if let Some(right) = &self.right {
            count += right.size();
        }
        count
    }

    /// Returns the height of the binary tree (longest path from this node to a leaf).
    pub fn height(&self) -> usize {
        let left_height = match &self.left {
            Some(left) => left.height() + 1,
            None => 0,
        };
        let right_height = match &self.right {
            Some(right) => right.height() + 1,
            None => 0,
        };

        if left_height > right_height {
            left_height
        } else {
            right_height
        }
    }

    /// Maps each node's value to a new value using the provided function.
    pub fn map<U, F>(&self, f: F) -> BinaryNode<U>
    where
        F: Fn(&T) -> U + Copy,
    {
        let left = match &self.left {
            Some(left) => Some(Box::new(left.map(f))),
            None => None,
        };

        let right = match &self.right {
            Some(right) => Some(Box::new(right.map(f))),
            None => None,
        };

        BinaryNode {
            value: f(&self.value),
            left,
            right,
        }
    }

    /// Finds the first node that matches the predicate in a pre-order traversal.
    pub fn find<F>(&self, predicate: F) -> Option<&BinaryNode<T>>
    where
        F: Fn(&T) -> bool + Copy,
    {
        if predicate(&self.value) {
            return Some(self);
        }

        if let Some(left) = &self.left {
            if let Some(found) = left.find(predicate) {
                return Some(found);
            }
        }

        if let Some(right) = &self.right {
            if let Some(found) = right.find(predicate) {
                return Some(found);
            }
        }

        None
    }

    /// Finds the first node that matches the predicate in a pre-order traversal, returning mutable reference.
    pub fn find_mut<F>(&mut self, predicate: F) -> Option<&mut BinaryNode<T>>
    where
        F: Fn(&T) -> bool + Copy,
    {
        if predicate(&self.value) {
            return Some(self);
        }

        if let Some(left) = &mut self.left {
            if let Some(found) = left.find_mut(predicate) {
                return Some(found);
            }
        }

        if let Some(right) = &mut self.right {
            if let Some(found) = right.find_mut(predicate) {
                return Some(found);
            }
        }

        None
    }
}

// Implement Clone if T is Clone
impl<T: Clone> Clone for BinaryNode<T> {
    fn clone(&self) -> Self {
        BinaryNode {
            value: self.value.clone(),
            left: self.left.clone(),
            right: self.right.clone(),
        }
    }
}



/// A binary tree data structure.
pub struct BinaryTree<T> {
    /// The root node of the binary tree
    pub root: Option<Box<BinaryNode<T>>>,
}

impl<T> BinaryTree<T> {
    /// Creates a new, empty binary tree.
    pub fn new() -> Self {
        BinaryTree { root: None }
    }

    /// Creates a new binary tree with the given root node.
    pub fn with_root(root: BinaryNode<T>) -> Self {
        BinaryTree { root: Some(Box::new(root)) }
    }

    /// Sets the root node of the binary tree.
    pub fn set_root(&mut self, root: BinaryNode<T>) {
        self.root = Some(Box::new(root));
    }

    /// Returns true if the binary tree is empty (has no root).
    pub fn is_empty(&self) -> bool {
        self.root.is_none()
    }

    /// Returns the size (number of nodes) of the binary tree.
    pub fn size(&self) -> usize {
        match &self.root {
            Some(root) => root.size(),
            None => 0,
        }
    }

    /// Returns the height of the binary tree.
    pub fn height(&self) -> usize {
        match &self.root {
            Some(root) => root.height(),
            None => 0,
        }
    }

    /// Traverses the binary tree in pre-order if it has a root.
    pub fn traverse_pre_order<F>(&self, mut f: F)
    where
        F: FnMut(&T),
    {
        if let Some(root) = &self.root {
            root.traverse_pre_order(&mut f);
        }
    }

    /// Traverses the binary tree in-order if it has a root.
    pub fn traverse_in_order<F>(&self, mut f: F)
    where
        F: FnMut(&T),
    {
        if let Some(root) = &self.root {
            root.traverse_in_order(&mut f);
        }
    }

    /// Traverses the binary tree in post-order if it has a root.
    pub fn traverse_post_order<F>(&self, mut f: F)
    where
        F: FnMut(&T),
    {
        if let Some(root) = &self.root {
            root.traverse_post_order(&mut f);
        }
    }

    /// Finds a node that matches the predicate.
    pub fn find<F>(&self, predicate: F) -> Option<&BinaryNode<T>>
    where
        F: Fn(&T) -> bool + Copy,
    {
        match &self.root {
            Some(root) => root.find(predicate),
            None => None,
        }
    }

    /// Finds a node that matches the predicate, returning a mutable reference.
    pub fn find_mut<F>(&mut self, predicate: F) -> Option<&mut BinaryNode<T>>
    where
        F: Fn(&T) -> bool + Copy,
    {
        match &mut self.root {
            Some(root) => root.find_mut(predicate),
            None => None,
        }
    }

    /// Clears the binary tree, removing all nodes.
    pub fn clear(&mut self) {
        self.root = None;
    }
}

// Implement Clone if T is Clone
impl<T: Clone> Clone for BinaryTree<T> {
    fn clone(&self) -> Self {
        BinaryTree {
            root: self.root.clone(),
        }
    }
}



// Implement Default
impl<T> Default for BinaryTree<T> {
    fn default() -> Self {
        Self::new()
    }
}

// Implement From<BinaryNode<T>> for BinaryTree<T>
impl<T> From<BinaryNode<T>> for BinaryTree<T> {
    fn from(node: BinaryNode<T>) -> Self {
        Self::with_root(node)
    }
}

/// Binary Search Tree node.
pub struct BstNode<T: Ord> {
    /// The data stored in this node
    pub value: T,
    /// The left child of this node
    pub left: Option<Box<BstNode<T>>>,
    /// The right child of this node
    pub right: Option<Box<BstNode<T>>>,
}

impl<T: Ord> BstNode<T> {
    /// Creates a new BST node with the given value and no children.
    pub fn new(value: T) -> Self {
        BstNode {
            value,
            left: None,
            right: None,
        }
    }

    /// Inserts a value into the BST.
    pub fn insert(&mut self, value: T) {
        match value.cmp(&self.value) {
            Ordering::Less => {
                match &mut self.left {
                    Some(left) => left.insert(value),
                    None => self.left = Some(Box::new(BstNode::new(value))),
                }
            }
            Ordering::Greater => {
                match &mut self.right {
                    Some(right) => right.insert(value),
                    None => self.right = Some(Box::new(BstNode::new(value))),
                }
            }
            Ordering::Equal => {
                // Value already exists in the tree, do nothing
                // Alternatively, could replace the value here if needed
            }
        }
    }

    /// Searches for a value in the BST.
    pub fn contains(&self, value: &T) -> bool {
        match value.cmp(&self.value) {
            Ordering::Less => {
                match &self.left {
                    Some(left) => left.contains(value),
                    None => false,
                }
            }
            Ordering::Greater => {
                match &self.right {
                    Some(right) => right.contains(value),
                    None => false,
                }
            }
            Ordering::Equal => true,
        }
    }

    /// Returns a reference to the node with the given value, or None if not found.
    pub fn find(&self, value: &T) -> Option<&BstNode<T>> {
        match value.cmp(&self.value) {
            Ordering::Less => {
                match &self.left {
                    Some(left) => left.find(value),
                    None => None,
                }
            }
            Ordering::Greater => {
                match &self.right {
                    Some(right) => right.find(value),
                    None => None,
                }
            }
            Ordering::Equal => Some(self),
        }
    }

    /// Returns a mutable reference to the node with the given value, or None if not found.
    pub fn find_mut(&mut self, value: &T) -> Option<&mut BstNode<T>> {
        match value.cmp(&self.value) {
            Ordering::Less => {
                match &mut self.left {
                    Some(left) => left.find_mut(value),
                    None => None,
                }
            }
            Ordering::Greater => {
                match &mut self.right {
                    Some(right) => right.find_mut(value),
                    None => None,
                }
            }
            Ordering::Equal => Some(self),
        }
    }

    /// Computes the minimum value in the BST.
    pub fn min_value(&self) -> &T {
        match &self.left {
            Some(left) => left.min_value(),
            None => &self.value,
        }
    }

    /// Computes the maximum value in the BST.
    pub fn max_value(&self) -> &T {
        match &self.right {
            Some(right) => right.max_value(),
            None => &self.value,
        }
    }

    /// Traverses the BST in-order, which yields values in sorted order.
    pub fn traverse_in_order<F>(&self, f: &mut F)
    where
        F: FnMut(&T),
    {
        if let Some(left) = &self.left {
            left.traverse_in_order(f);
        }
        f(&self.value);
        if let Some(right) = &self.right {
            right.traverse_in_order(f);
        }
    }
}

// Implement Clone if T is Clone
impl<T: Ord + Clone> Clone for BstNode<T> {
    fn clone(&self) -> Self {
        BstNode {
            value: self.value.clone(),
            left: self.left.clone(),
            right: self.right.clone(),
        }
    }
}


// Implement From for converting from a value to a BstNode
impl<T: Ord> From<T> for BstNode<T> {
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

/// A Binary Search Tree data structure.
pub struct BinarySearchTree<T: Ord> {
    /// The root node of the BST
    pub root: Option<Box<BstNode<T>>>,
}

impl<T: Ord + Clone> BinarySearchTree<T> {
    /// Creates a new, empty BST.
    pub fn new() -> Self {
        BinarySearchTree { root: None }
    }

    /// Creates a new BST with the given root node.
    pub fn with_root(root: BstNode<T>) -> Self {
        BinarySearchTree { root: Some(Box::new(root)) }
    }

    /// Returns true if the BST is empty.
    pub fn is_empty(&self) -> bool {
        self.root.is_none()
    }

    /// Inserts a value into the BST.
    pub fn insert(&mut self, value: T) {
        match &mut self.root {
            Some(root) => root.insert(value),
            None => self.root = Some(Box::new(BstNode::new(value))),
        }
    }

    /// Searches for a value in the BST.
    pub fn contains(&self, value: &T) -> bool {
        match &self.root {
            Some(root) => root.contains(value),
            None => false,
        }
    }

    /// Returns a reference to the node with the given value, or None if not found.
    pub fn find(&self, value: &T) -> Option<&BstNode<T>> {
        match &self.root {
            Some(root) => root.find(value),
            None => None,
        }
    }

    /// Returns a mutable reference to the node with the given value, or None if not found.
    pub fn find_mut(&mut self, value: &T) -> Option<&mut BstNode<T>> {
        match &mut self.root {
            Some(root) => root.find_mut(value),
            None => None,
        }
    }

    /// Computes the minimum value in the BST, if it exists.
    pub fn min_value(&self) -> Option<&T> {
        match &self.root {
            Some(root) => Some(root.min_value()),
            None => None,
        }
    }

    /// Computes the maximum value in the BST, if it exists.
    pub fn max_value(&self) -> Option<&T> {
        match &self.root {
            Some(root) => Some(root.max_value()),
            None => None,
        }
    }

    /// Traverses the BST in-order (sorted order).
    pub fn traverse_in_order<F>(&self, mut f: F)
    where
        F: FnMut(&T),
    {
        if let Some(root) = &self.root {
            root.traverse_in_order(&mut f);
        }
    }

    /// Collects all values from the BST in sorted order.
    pub fn to_sorted_vec(&self) -> Vec<T>
    where
        T: Clone,
    {
        let mut result = Vec::new();
        self.traverse_in_order(|value| result.push(value.clone()));
        result
    }

    /// Removes a value from the BST.
    pub fn remove(&mut self, value: &T) -> Option<T> {
        Self::remove_node(&mut self.root, value)
    }

    // Helper method for removing a node
    fn remove_node(node: &mut Option<Box<BstNode<T>>>, value: &T) -> Option<T> {
        if node.is_none() {
            return None;
        }

        let current = node.as_mut().unwrap();

        match value.cmp(&current.value) {
            Ordering::Less => Self::remove_node(&mut current.left, value),
            Ordering::Greater => Self::remove_node(&mut current.right, value),
            Ordering::Equal => {
                // Node with no children or one child
                if current.left.is_none() {
                    let mut right_child = None;
                    core::mem::swap(&mut right_child, &mut current.right);
                    let removed = *node.take().unwrap();
                    *node = right_child;
                    return Some(removed.value);
                } else if current.right.is_none() {
                    let mut left_child = None;
                    core::mem::swap(&mut left_child, &mut current.left);
                    let removed = *node.take().unwrap();
                    *node = left_child;
                    return Some(removed.value);
                }

                // Node with two children: Get the inorder successor (smallest in right subtree)
                let successor_value = current.right.as_ref().unwrap().min_value().clone();
                current.value = successor_value;
                Self::remove_node(&mut current.right, &current.value)
            }
        }
    }

    /// Returns the size (number of nodes) of the BST.
    pub fn size(&self) -> usize {
        match &self.root {
            Some(root) => Self::count_nodes(root),
            None => 0,
        }
    }

    // Helper method for counting nodes
    fn count_nodes(node: &BstNode<T>) -> usize {
        let mut count = 1;
        if let Some(left) = &node.left {
            count += Self::count_nodes(left);
        }
        if let Some(right) = &node.right {
            count += Self::count_nodes(right);
        }
        count
    }

    /// Returns the height of the BST.
    pub fn height(&self) -> usize {
        match &self.root {
            Some(root) => Self::calculate_height(root),
            None => 0,
        }
    }

    // Helper method for calculating height
    fn calculate_height(node: &BstNode<T>) -> usize {
        let left_height = match &node.left {
            Some(left) => 1 + Self::calculate_height(left),
            None => 0,
        };

        let right_height = match &node.right {
            Some(right) => 1 + Self::calculate_height(right),
            None => 0,
        };

        if left_height > right_height {
            left_height
        } else {
            right_height
        }
    }

    /// Checks if the tree is a valid BST.
    pub fn is_valid_bst(&self) -> bool {
        match &self.root {
            Some(root) => Self::validate_bst(root, None, None),
            None => true,
        }
    }

    // Helper method for validating BST property
    fn validate_bst(node: &BstNode<T>, min: Option<&T>, max: Option<&T>) -> bool {
        // Check if current node's value is in the valid range
        if let Some(min_val) = min {
            if node.value <= *min_val {
                return false;
            }
        }

        if let Some(max_val) = max {
            if node.value >= *max_val {
                return false;
            }
        }

        // Recursively check left and right subtrees
        let left_valid = match &node.left {
            Some(left) => Self::validate_bst(left, min, Some(&node.value)),
            None => true,
        };

        if !left_valid {
            return false;
        }

        let right_valid = match &node.right {
            Some(right) => Self::validate_bst(right, Some(&node.value), max),
            None => true,
        };

        right_valid
    }

    /// Clears the BST, removing all nodes.
    pub fn clear(&mut self) {
        self.root = None;
    }
}

// Implement Default for BinarySearchTree
impl<T: Ord + Clone> Default for BinarySearchTree<T> {
    fn default() -> Self {
        Self::new()
    }
}

// Implement Clone for BinarySearchTree if T is Clone
impl<T: Ord + Clone> Clone for BinarySearchTree<T> {
    fn clone(&self) -> Self {
        BinarySearchTree {
            root: self.root.clone(),
        }
    }
}



// Implement From<BstNode<T>> for BinarySearchTree<T>
impl<T: Ord + Clone> From<BstNode<T>> for BinarySearchTree<T> {
    fn from(node: BstNode<T>) -> Self {
        Self::with_root(node)
    }
}

// Implement FromIterator for BinarySearchTree
impl<T: Ord + Clone> FromIterator<T> for BinarySearchTree<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut tree = BinarySearchTree::new();
        for value in iter {
            tree.insert(value);
        }
        tree
    }
}

/// An AVL tree node with self-balancing capabilities.
#[derive(Clone)]
pub struct AvlNode<T: Ord> {
    /// The data stored in this node
    pub value: T,
    /// The left child of this node
    pub left: Option<Box<AvlNode<T>>>,
    /// The right child of this node
    pub right: Option<Box<AvlNode<T>>>,
    /// Height of this node for balancing
    pub height: usize,
}

impl<T: Ord + Clone> AvlNode<T> {
    /// Creates a new AVL node with the given value and no children.
    pub fn new(value: T) -> Self {
        AvlNode {
            value,
            left: None,
            right: None,
            height: 1,
        }
    }

    /// Gets the height of the node, or 0 if the node is None.
    fn height(node: &Option<Box<AvlNode<T>>>) -> usize {
        match node {
            Some(n) => n.height,
            None => 0,
        }
    }

    /// Calculates the balance factor of a node.
    fn balance_factor(node: &Option<Box<AvlNode<T>>>) -> isize {
        match node {
            Some(n) => {
                let left_height = Self::height(&n.left) as isize;
                let right_height = Self::height(&n.right) as isize;
                left_height - right_height
            }
            None => 0,
        }
    }

    /// Updates the height of a node based on its children.
    fn update_height(&mut self) {
        let left_height = Self::height(&self.left);
        let right_height = Self::height(&self.right);
        self.height = 1 + core::cmp::max(left_height, right_height);
    }

    /// Right rotates the subtree rooted with this node.
    fn right_rotate(mut root: Box<AvlNode<T>>) -> Box<AvlNode<T>> {
        let mut new_root = root.left.take().unwrap();
        root.left = new_root.right.take();
        root.update_height();
        new_root.right = Some(root);
        new_root.update_height();
        new_root
    }

    /// Left rotates the subtree rooted with this node.
    fn left_rotate(mut root: Box<AvlNode<T>>) -> Box<AvlNode<T>> {
        let mut new_root = root.right.take().unwrap();
        root.right = new_root.left.take();
        root.update_height();
        new_root.left = Some(root);
        new_root.update_height();
        new_root
    }

    /// Balances an AVL node if needed and returns the balanced node.
    fn balance(mut node: Box<AvlNode<T>>) -> Box<AvlNode<T>> {
        node.update_height();
        let balance = Self::balance_factor(&Some(node.clone()));

        // Left heavy
        if balance > 1 {
            if Self::balance_factor(&node.left) < 0 {
                // Left-Right Case
                let left = node.left.take().unwrap();
                node.left = Some(Self::left_rotate(left));
            }
            // Left-Left Case
            return Self::right_rotate(node);
        }

        // Right heavy
        if balance < -1 {
            if Self::balance_factor(&node.right) > 0 {
                // Right-Left Case
                let right = node.right.take().unwrap();
                node.right = Some(Self::right_rotate(right));
            }
            // Right-Right Case
            return Self::left_rotate(node);
        }

        // Already balanced
        node
    }

    /// Inserts a value into the AVL tree.
    fn insert(node: Option<Box<AvlNode<T>>>, value: T) -> Option<Box<AvlNode<T>>> {
        match node {
            None => Some(Box::new(AvlNode::new(value))),
            Some(mut node) => {
                match value.cmp(&node.value) {
                    Ordering::Less => {
                        node.left = Self::insert(node.left, value);
                    }
                    Ordering::Greater => {
                        node.right = Self::insert(node.right, value);
                    }
                    Ordering::Equal => {
                        // Value already exists, do nothing
                        return Some(node);
                    }
                }

                Some(Self::balance(node))
            }
        }
    }

    /// Finds the node with the minimum value in the tree.
    fn min_value_node(node: &Option<Box<AvlNode<T>>>) -> Option<&T> {
        match node {
            None => None,
            Some(n) => {
                let mut current = n;
                while let Some(left) = &current.left {
                    current = left;
                }
                Some(&current.value)
            }
        }
    }

    /// Removes a value from the AVL tree.
    fn remove(node: Option<Box<AvlNode<T>>>, value: &T) -> Option<Box<AvlNode<T>>> {
        match node {
            None => None,
            Some(mut node) => {
                match value.cmp(&node.value) {
                    Ordering::Less => {
                        node.left = Self::remove(node.left, value);
                    }
                    Ordering::Greater => {
                        node.right = Self::remove(node.right, value);
                    }
                    Ordering::Equal => {
                        // Node with one child or no child
                        if node.left.is_none() {
                            return node.right;
                        } else if node.right.is_none() {
                            return node.left;
                        }

                        // Node with two children: Get the inorder successor (smallest in right subtree)
                        if let Some(min_value) = Self::min_value_node(&node.right) {
                            let min_value = min_value.clone();
                            node.value = min_value;
                            node.right = Self::remove(node.right, &node.value);
                        }
                    }
                }

                Some(Self::balance(node))
            }
        }
    }

    /// Traverses the AVL tree in-order.
    pub fn traverse_in_order<F>(&self, f: &mut F)
    where
        F: FnMut(&T),
    {
        if let Some(left) = &self.left {
            left.traverse_in_order(f);
        }
        f(&self.value);
        if let Some(right) = &self.right {
            right.traverse_in_order(f);
        }
    }
}



/// A self-balancing AVL tree data structure.
pub struct AvlTree<T: Ord> {
    /// The root node of the AVL tree
    pub root: Option<Box<AvlNode<T>>>,
}

impl<T: Ord + Clone> AvlTree<T> {
    /// Creates a new, empty AVL tree.
    pub fn new() -> Self {
        AvlTree { root: None }
    }

    /// Returns true if the AVL tree is empty.
    pub fn is_empty(&self) -> bool {
        self.root.is_none()
    }

    /// Inserts a value into the AVL tree.
    pub fn insert(&mut self, value: T) {
        self.root = AvlNode::insert(self.root.take(), value);
    }

    /// Removes a value from the AVL tree.
    pub fn remove(&mut self, value: &T) {
        self.root = AvlNode::remove(self.root.take(), value);
    }

    /// Searches for a value in the AVL tree.
    pub fn contains(&self, value: &T) -> bool {
        let mut current = &self.root;
        while let Some(node) = current {
            match value.cmp(&node.value) {
                Ordering::Less => current = &node.left,
                Ordering::Greater => current = &node.right,
                Ordering::Equal => return true,
            }
        }
        false
    }

    /// Returns the height of the AVL tree.
    pub fn height(&self) -> usize {
        AvlNode::height(&self.root)
    }

    /// Traverses the AVL tree in-order (sorted order).
    pub fn traverse_in_order<F>(&self, mut f: F)
    where
        F: FnMut(&T),
    {
        if let Some(root) = &self.root {
            root.traverse_in_order(&mut f);
        }
    }

    /// Collects all values from the AVL tree in sorted order.
    pub fn to_sorted_vec(&self) -> Vec<T>
    where
        T: Clone,
    {
        let mut result = Vec::new();
        self.traverse_in_order(|value| result.push(value.clone()));
        result
    }

    /// Clears the AVL tree, removing all nodes.
    pub fn clear(&mut self) {
        self.root = None;
    }
}

// Implement Default for AvlTree
impl<T: Ord + Clone> Default for AvlTree<T> {
    fn default() -> Self {
        Self::new()
    }
}

// Implement Clone for AvlTree if T is Clone
impl<T: Ord + Clone> Clone for AvlTree<T> {
    fn clone(&self) -> Self {
        AvlTree {
            root: self.root.clone(),
        }
    }
}



// Implement FromIterator for AvlTree
impl<T: Ord + Clone> FromIterator<T> for AvlTree<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut tree = AvlTree::new();
        for value in iter {
            tree.insert(value);
        }
        tree
    }
}

/// A trait for tree-like structures that provide iterators over their values.
pub trait TreeIterable<T> {
    /// Returns an iterator that performs in-order traversal.
    fn iter_in_order(&self) -> InOrderIterator<'_, T>;

    /// Returns an iterator that performs pre-order traversal.
    fn iter_pre_order(&self) -> PreOrderIterator<'_, T>;

    /// Returns an iterator that performs post-order traversal.
    fn iter_post_order(&self) -> PostOrderIterator<'_, T>;

    /// Returns an iterator that performs breadth-first traversal.
    fn iter_breadth_first(&self) -> BreadthFirstIterator<'_, T>;
}

/// Iterator for in-order traversal of a tree.
pub struct InOrderIterator<'a, T> {
    stack: Vec<&'a Node<T>>,
    current: Option<&'a Node<T>>,
}

impl<'a, T> Iterator for InOrderIterator<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        while self.current.is_some() || !self.stack.is_empty() {
            if let Some(node) = self.current {
                self.stack.push(node);
                self.current = node.children.first();
            } else {
                let node = self.stack.pop()?;
                let result = &node.value;
                self.current = if node.children.len() > 1 {
                    node.children.get(1)
                } else {
                    None
                };
                return Some(result);
            }
        }
        None
    }
}

/// Iterator for pre-order traversal of a tree.
pub struct PreOrderIterator<'a, T> {
    stack: Vec<&'a Node<T>>,
}

impl<'a, T> Iterator for PreOrderIterator<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        let node = self.stack.pop()?;

        // Push children in reverse order so they are popped in the original order
        for child in node.children.iter().rev() {
            self.stack.push(child);
        }

        Some(&node.value)
    }
}

/// Iterator for post-order traversal of a tree.
pub struct PostOrderIterator<'a, T> {
    stack: Vec<(&'a Node<T>, bool)>, // Node and a flag indicating if it's been visited
}

impl<'a, T> Iterator for PostOrderIterator<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some((node, visited)) = self.stack.pop() {
            if visited {
                return Some(&node.value);
            } else {
                self.stack.push((node, true));

                // Push children in reverse order
                for child in node.children.iter().rev() {
                    self.stack.push((child, false));
                }
            }
        }
        None
    }
}

/// Iterator for breadth-first traversal of a tree.
pub struct BreadthFirstIterator<'a, T> {
    queue: Vec<&'a Node<T>>,
}

impl<'a, T> Iterator for BreadthFirstIterator<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        let node = self.queue.remove(0);

        // Enqueue all children
        for child in &node.children {
            self.queue.push(child);
        }

        Some(&node.value)
    }
}

// Implement TreeIterable for Tree
impl<T> TreeIterable<T> for Tree<T> {
    fn iter_in_order(&self) -> InOrderIterator<'_, T> {
        match &self.root {
            Some(root) => InOrderIterator {
                stack: Vec::new(),
                current: Some(root),
            },
            None => InOrderIterator {
                stack: Vec::new(),
                current: None,
            },
        }
    }

    fn iter_pre_order(&self) -> PreOrderIterator<'_, T> {
        let mut stack = Vec::new();
        if let Some(root) = &self.root {
            stack.push(root);
        }
        PreOrderIterator { stack }
    }

    fn iter_post_order(&self) -> PostOrderIterator<'_, T> {
        let mut stack = Vec::new();
        if let Some(root) = &self.root {
            stack.push((root, false));
        }
        PostOrderIterator { stack }
    }

    fn iter_breadth_first(&self) -> BreadthFirstIterator<'_, T> {
        let mut queue = Vec::new();
        if let Some(root) = &self.root {
            queue.push(root);
        }
        BreadthFirstIterator { queue }
    }
}