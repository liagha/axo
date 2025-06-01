#![allow(dead_code)]

extern crate alloc;

use {
    crate::{
        memory::swap,
        compare::{max, Ordering},
    },
    
    alloc::{
        vec::Vec,
        boxed::Box,
        string::String,
    },
};

#[derive(Eq, Hash, PartialEq)]
pub struct Tree<T> {
    pub root: Option<Node<T>>,
}

#[derive(Clone, Eq, Hash, PartialEq)]
pub struct Node<T> {
    pub value: T,
    pub children: Vec<Node<T>>,
}

impl<T> Node<T> {
    pub fn new(value: T) -> Self {
        Node {
            value,
            children: Vec::new(),
        }
    }

    pub fn with_children(value: T, children: Vec<Node<T>>) -> Self {
        Node { value, children }
    }

    pub fn add_child(&mut self, child: Node<T>) {
        self.children.push(child);
    }

    pub fn add_value(&mut self, value: T) {
        self.children.push(Node::new(value));
    }

    pub fn child_count(&self) -> usize {
        self.children.len()
    }

    pub fn has_children(&self) -> bool {
        !self.children.is_empty()
    }

    pub fn get_child(&self, index: usize) -> Option<&Node<T>> {
        self.children.get(index)
    }

    pub fn get_child_mut(&mut self, index: usize) -> Option<&mut Node<T>> {
        self.children.get_mut(index)
    }

    pub fn remove_child(&mut self, index: usize) -> Option<Node<T>> {
        if index < self.children.len() {
            Some(self.children.remove(index))
        } else {
            None
        }
    }

    pub fn traverse_pre_order<F>(&self, f: &mut F)
    where
        F: FnMut(&T),
    {
        f(&self.value);
        for child in &self.children {
            child.traverse_pre_order(f);
        }
    }

    pub fn traverse_post_order<F>(&self, f: &mut F)
    where
        F: FnMut(&T),
    {
        for child in &self.children {
            child.traverse_post_order(f);
        }
        f(&self.value);
    }

    pub fn traverse_breadth_first<F>(&self, mut f: F)
    where
        F: FnMut(&T),
    {
        let mut queue = Vec::new();
        queue.push(self);

        while !queue.is_empty() {
            let node = queue.remove(0);

            f(&node.value);

            for child in &node.children {
                queue.push(child);
            }
        }
    }

    pub fn size(&self) -> usize {
        let mut count = 1; 
        for child in &self.children {
            count += child.size();
        }
        count
    }

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
    pub fn new() -> Self {
        Tree { root: None }
    }

    pub fn with_root(value: T) -> Self {
        Tree {
            root: Some(Node::new(value)),
        }
    }

    pub fn with_root_node(node: Node<T>) -> Self {
        Tree { root: Some(node) }
    }

    pub fn is_empty(&self) -> bool {
        self.root.is_none()
    }

    pub fn root(&self) -> Option<&Node<T>> {
        self.root.as_ref()
    }

    pub fn root_mut(&mut self) -> Option<&mut Node<T>> {
        self.root.as_mut()
    }

    pub fn set_root(&mut self, node: Node<T>) {
        self.root = Some(node);
    }

    pub fn add_child(&mut self, value: T) -> Result<(), &'static str> {
        match self.root.as_mut() {
            Some(root) => {
                root.add_value(value);
                Ok(())
            }
            None => Err("Cannot add child to an empty tree"),
        }
    }

    pub fn add_child_node(&mut self, node: Node<T>) -> Result<(), &'static str> {
        match self.root.as_mut() {
            Some(root) => {
                root.add_child(node);
                Ok(())
            }
            None => Err("Cannot add child to an empty tree"),
        }
    }

    pub fn size(&self) -> usize {
        match &self.root {
            Some(root) => root.size(),
            None => 0,
        }
    }

    pub fn height(&self) -> usize {
        match &self.root {
            Some(root) => root.height(),
            None => 0,
        }
    }

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

    pub fn traverse_pre_order<F>(&self, mut f: F)
    where
        F: FnMut(&T),
    {
        if let Some(root) = &self.root {
            root.traverse_pre_order(&mut f);
        }
    }

    pub fn traverse_post_order<F>(&self, mut f: F)
    where
        F: FnMut(&T),
    {
        if let Some(root) = &self.root {
            root.traverse_post_order(&mut f);
        }
    }

    pub fn traverse_breadth_first<F>(&self, f: F)
    where
        F: FnMut(&T),
    {
        if let Some(root) = &self.root {
            root.traverse_breadth_first(f);
        }
    }

    pub fn find<F>(&self, predicate: F) -> Option<&Node<T>>
    where
        F: Fn(&T) -> bool,
    {
        match &self.root {
            Some(root) => root.find(predicate),
            None => None,
        }
    }

    pub fn find_mut<F>(&mut self, predicate: F) -> Option<&mut Node<T>>
    where
        F: Fn(&T) -> bool,
    {
        match &mut self.root {
            Some(root) => root.find_mut(predicate),
            None => None,
        }
    }

    pub fn node_at_path(&self, path: &[usize]) -> Option<&Node<T>> {
        match &self.root {
            Some(root) => root.node_at_path(path),
            None => None,
        }
    }

    pub fn node_at_path_mut(&mut self, path: &[usize]) -> Option<&mut Node<T>> {
        match &mut self.root {
            Some(root) => root.node_at_path_mut(path),
            None => None,
        }
    }
}

impl<T: Clone> Clone for Tree<T> {
    fn clone(&self) -> Self {
        Tree {
            root: self.root.clone(),
        }
    }
}



impl<T> Default for Tree<T> {
    fn default() -> Self {
        Self::new()
    }
}

pub struct BinaryNode<T> {
    pub value: T,
    pub left: Option<Box<BinaryNode<T>>>,
    pub right: Option<Box<BinaryNode<T>>>,
}

impl<T> BinaryNode<T> {
    pub fn new(value: T) -> Self {
        BinaryNode {
            value,
            left: None,
            right: None,
        }
    }

    pub fn with_children(
        value: T,
        left: Option<Box<BinaryNode<T>>>,
        right: Option<Box<BinaryNode<T>>>,
    ) -> Self {
        BinaryNode { value, left, right }
    }

    pub fn set_left(&mut self, node: BinaryNode<T>) {
        self.left = Some(Box::new(node));
    }

    pub fn set_right(&mut self, node: BinaryNode<T>) {
        self.right = Some(Box::new(node));
    }

    pub fn has_left(&self) -> bool {
        self.left.is_some()
    }

    pub fn has_right(&self) -> bool {
        self.right.is_some()
    }

    pub fn is_leaf(&self) -> bool {
        self.left.is_none() && self.right.is_none()
    }

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

    pub fn size(&self) -> usize {
        let mut count = 1; 
        if let Some(left) = &self.left {
            count += left.size();
        }
        if let Some(right) = &self.right {
            count += right.size();
        }
        count
    }

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

impl<T: Clone> Clone for BinaryNode<T> {
    fn clone(&self) -> Self {
        BinaryNode {
            value: self.value.clone(),
            left: self.left.clone(),
            right: self.right.clone(),
        }
    }
}



pub struct BinaryTree<T> {
    pub root: Option<Box<BinaryNode<T>>>,
}

impl<T> BinaryTree<T> {
    pub fn new() -> Self {
        BinaryTree { root: None }
    }

    pub fn with_root(root: BinaryNode<T>) -> Self {
        BinaryTree { root: Some(Box::new(root)) }
    }

    pub fn set_root(&mut self, root: BinaryNode<T>) {
        self.root = Some(Box::new(root));
    }

    pub fn is_empty(&self) -> bool {
        self.root.is_none()
    }

    pub fn size(&self) -> usize {
        match &self.root {
            Some(root) => root.size(),
            None => 0,
        }
    }

    pub fn height(&self) -> usize {
        match &self.root {
            Some(root) => root.height(),
            None => 0,
        }
    }

    pub fn traverse_pre_order<F>(&self, mut f: F)
    where
        F: FnMut(&T),
    {
        if let Some(root) = &self.root {
            root.traverse_pre_order(&mut f);
        }
    }

    pub fn traverse_in_order<F>(&self, mut f: F)
    where
        F: FnMut(&T),
    {
        if let Some(root) = &self.root {
            root.traverse_in_order(&mut f);
        }
    }

    pub fn traverse_post_order<F>(&self, mut f: F)
    where
        F: FnMut(&T),
    {
        if let Some(root) = &self.root {
            root.traverse_post_order(&mut f);
        }
    }

    pub fn find<F>(&self, predicate: F) -> Option<&BinaryNode<T>>
    where
        F: Fn(&T) -> bool + Copy,
    {
        match &self.root {
            Some(root) => root.find(predicate),
            None => None,
        }
    }

    pub fn find_mut<F>(&mut self, predicate: F) -> Option<&mut BinaryNode<T>>
    where
        F: Fn(&T) -> bool + Copy,
    {
        match &mut self.root {
            Some(root) => root.find_mut(predicate),
            None => None,
        }
    }

    pub fn clear(&mut self) {
        self.root = None;
    }
}

impl<T: Clone> Clone for BinaryTree<T> {
    fn clone(&self) -> Self {
        BinaryTree {
            root: self.root.clone(),
        }
    }
}



impl<T> Default for BinaryTree<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> From<BinaryNode<T>> for BinaryTree<T> {
    fn from(node: BinaryNode<T>) -> Self {
        Self::with_root(node)
    }
}

pub struct BstNode<T: Ord> {
    pub value: T,
    pub left: Option<Box<BstNode<T>>>,
    pub right: Option<Box<BstNode<T>>>,
}

impl<T: Ord> BstNode<T> {
    pub fn new(value: T) -> Self {
        BstNode {
            value,
            left: None,
            right: None,
        }
    }

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
            Ordering::Equal => {}
        }
    }

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

    pub fn min_value(&self) -> &T {
        match &self.left {
            Some(left) => left.min_value(),
            None => &self.value,
        }
    }

    pub fn max_value(&self) -> &T {
        match &self.right {
            Some(right) => right.max_value(),
            None => &self.value,
        }
    }

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

impl<T: Ord + Clone> Clone for BstNode<T> {
    fn clone(&self) -> Self {
        BstNode {
            value: self.value.clone(),
            left: self.left.clone(),
            right: self.right.clone(),
        }
    }
}


impl<T: Ord> From<T> for BstNode<T> {
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

pub struct BinarySearchTree<T: Ord> {
    pub root: Option<Box<BstNode<T>>>,
}

impl<T: Ord + Clone> BinarySearchTree<T> {
    pub fn new() -> Self {
        BinarySearchTree { root: None }
    }

    pub fn with_root(root: BstNode<T>) -> Self {
        BinarySearchTree { root: Some(Box::new(root)) }
    }

    pub fn is_empty(&self) -> bool {
        self.root.is_none()
    }

    pub fn insert(&mut self, value: T) {
        match &mut self.root {
            Some(root) => root.insert(value),
            None => self.root = Some(Box::new(BstNode::new(value))),
        }
    }

    pub fn contains(&self, value: &T) -> bool {
        match &self.root {
            Some(root) => root.contains(value),
            None => false,
        }
    }

    pub fn find(&self, value: &T) -> Option<&BstNode<T>> {
        match &self.root {
            Some(root) => root.find(value),
            None => None,
        }
    }

    pub fn find_mut(&mut self, value: &T) -> Option<&mut BstNode<T>> {
        match &mut self.root {
            Some(root) => root.find_mut(value),
            None => None,
        }
    }

    pub fn min_value(&self) -> Option<&T> {
        match &self.root {
            Some(root) => Some(root.min_value()),
            None => None,
        }
    }

    pub fn max_value(&self) -> Option<&T> {
        match &self.root {
            Some(root) => Some(root.max_value()),
            None => None,
        }
    }

    pub fn traverse_in_order<F>(&self, mut f: F)
    where
        F: FnMut(&T),
    {
        if let Some(root) = &self.root {
            root.traverse_in_order(&mut f);
        }
    }

    pub fn to_sorted_vec(&self) -> Vec<T>
    where
        T: Clone,
    {
        let mut result = Vec::new();
        self.traverse_in_order(|value| result.push(value.clone()));
        result
    }

    pub fn remove(&mut self, value: &T) -> Option<T> {
        Self::remove_node(&mut self.root, value)
    }

    fn remove_node(node: &mut Option<Box<BstNode<T>>>, value: &T) -> Option<T> {
        if node.is_none() {
            return None;
        }

        let current = node.as_mut().unwrap();

        match value.cmp(&current.value) {
            Ordering::Less => Self::remove_node(&mut current.left, value),
            Ordering::Greater => Self::remove_node(&mut current.right, value),
            Ordering::Equal => {
                if current.left.is_none() {
                    let mut right_child = None;
                    swap(&mut right_child, &mut current.right);
                    let removed = *node.take().unwrap();
                    *node = right_child;
                    return Some(removed.value);
                } else if current.right.is_none() {
                    let mut left_child = None;
                    swap(&mut left_child, &mut current.left);
                    let removed = *node.take().unwrap();
                    *node = left_child;
                    return Some(removed.value);
                }

                let successor_value = current.right.as_ref().unwrap().min_value().clone();
                current.value = successor_value;
                Self::remove_node(&mut current.right, &current.value)
            }
        }
    }

    pub fn size(&self) -> usize {
        match &self.root {
            Some(root) => Self::count_nodes(root),
            None => 0,
        }
    }

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

    pub fn height(&self) -> usize {
        match &self.root {
            Some(root) => Self::calculate_height(root),
            None => 0,
        }
    }

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

    pub fn is_valid_bst(&self) -> bool {
        match &self.root {
            Some(root) => Self::validate_bst(root, None, None),
            None => true,
        }
    }

    fn validate_bst(node: &BstNode<T>, min: Option<&T>, max: Option<&T>) -> bool {
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

    pub fn clear(&mut self) {
        self.root = None;
    }
}

impl<T: Ord + Clone> Default for BinarySearchTree<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Ord + Clone> Clone for BinarySearchTree<T> {
    fn clone(&self) -> Self {
        BinarySearchTree {
            root: self.root.clone(),
        }
    }
}



impl<T: Ord + Clone> From<BstNode<T>> for BinarySearchTree<T> {
    fn from(node: BstNode<T>) -> Self {
        Self::with_root(node)
    }
}

impl<T: Ord + Clone> FromIterator<T> for BinarySearchTree<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut tree = BinarySearchTree::new();
        for value in iter {
            tree.insert(value);
        }
        tree
    }
}

#[derive(Clone)]
pub struct AvlNode<T: Ord> {
    pub value: T,
    pub left: Option<Box<AvlNode<T>>>,
    pub right: Option<Box<AvlNode<T>>>,
    pub height: usize,
}

impl<T: Ord + Clone> AvlNode<T> {
    pub fn new(value: T) -> Self {
        AvlNode {
            value,
            left: None,
            right: None,
            height: 1,
        }
    }

    fn height(node: &Option<Box<AvlNode<T>>>) -> usize {
        match node {
            Some(n) => n.height,
            None => 0,
        }
    }

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

    fn update_height(&mut self) {
        let left_height = Self::height(&self.left);
        let right_height = Self::height(&self.right);
        self.height = 1 + max(left_height, right_height);
    }

    fn right_rotate(mut root: Box<AvlNode<T>>) -> Box<AvlNode<T>> {
        let mut new_root = root.left.take().unwrap();
        root.left = new_root.right.take();
        root.update_height();
        new_root.right = Some(root);
        new_root.update_height();
        new_root
    }

    fn left_rotate(mut root: Box<AvlNode<T>>) -> Box<AvlNode<T>> {
        let mut new_root = root.right.take().unwrap();
        root.right = new_root.left.take();
        root.update_height();
        new_root.left = Some(root);
        new_root.update_height();
        new_root
    }

    fn balance(mut node: Box<AvlNode<T>>) -> Box<AvlNode<T>> {
        node.update_height();
        let balance = Self::balance_factor(&Some(node.clone()));

        if balance > 1 {
            if Self::balance_factor(&node.left) < 0 {
                let left = node.left.take().unwrap();
                node.left = Some(Self::left_rotate(left));
            }
            return Self::right_rotate(node);
        }

        if balance < -1 {
            if Self::balance_factor(&node.right) > 0 {
                let right = node.right.take().unwrap();
                node.right = Some(Self::right_rotate(right));
            }
            return Self::left_rotate(node);
        }

        node
    }

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
                        return Some(node);
                    }
                }

                Some(Self::balance(node))
            }
        }
    }

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
                        if node.left.is_none() {
                            return node.right;
                        } else if node.right.is_none() {
                            return node.left;
                        }

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



pub struct AvlTree<T: Ord> {
    pub root: Option<Box<AvlNode<T>>>,
}

impl<T: Ord + Clone> AvlTree<T> {
    pub fn new() -> Self {
        AvlTree { root: None }
    }

    pub fn is_empty(&self) -> bool {
        self.root.is_none()
    }

    pub fn insert(&mut self, value: T) {
        self.root = AvlNode::insert(self.root.take(), value);
    }

    pub fn remove(&mut self, value: &T) {
        self.root = AvlNode::remove(self.root.take(), value);
    }

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

    pub fn height(&self) -> usize {
        AvlNode::height(&self.root)
    }

    pub fn traverse_in_order<F>(&self, mut f: F)
    where
        F: FnMut(&T),
    {
        if let Some(root) = &self.root {
            root.traverse_in_order(&mut f);
        }
    }

    pub fn to_sorted_vec(&self) -> Vec<T>
    where
        T: Clone,
    {
        let mut result = Vec::new();
        self.traverse_in_order(|value| result.push(value.clone()));
        result
    }

    pub fn clear(&mut self) {
        self.root = None;
    }
}

impl<T: Ord + Clone> Default for AvlTree<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Ord + Clone> Clone for AvlTree<T> {
    fn clone(&self) -> Self {
        AvlTree {
            root: self.root.clone(),
        }
    }
}



impl<T: Ord + Clone> FromIterator<T> for AvlTree<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut tree = AvlTree::new();
        for value in iter {
            tree.insert(value);
        }
        tree
    }
}

pub trait TreeIterable<T> {
    fn iter_in_order(&self) -> InOrderIterator<'_, T>;

    fn iter_pre_order(&self) -> PreOrderIterator<'_, T>;

    fn iter_post_order(&self) -> PostOrderIterator<'_, T>;

    fn iter_breadth_first(&self) -> BreadthFirstIterator<'_, T>;
}

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

pub struct PreOrderIterator<'a, T> {
    stack: Vec<&'a Node<T>>,
}

impl<'a, T> Iterator for PreOrderIterator<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        let node = self.stack.pop()?;

        for child in node.children.iter().rev() {
            self.stack.push(child);
        }

        Some(&node.value)
    }
}

pub struct PostOrderIterator<'a, T> {
    stack: Vec<(&'a Node<T>, bool)>, 
}

impl<'a, T> Iterator for PostOrderIterator<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some((node, visited)) = self.stack.pop() {
            if visited {
                return Some(&node.value);
            } else {
                self.stack.push((node, true));

                for child in node.children.iter().rev() {
                    self.stack.push((child, false));
                }
            }
        }
        None
    }
}

pub struct BreadthFirstIterator<'a, T> {
    queue: Vec<&'a Node<T>>,
}

impl<'a, T> Iterator for BreadthFirstIterator<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        let node = self.queue.remove(0);

        for child in &node.children {
            self.queue.push(child);
        }

        Some(&node.value)
    }
}

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