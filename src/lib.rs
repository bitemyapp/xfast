#![allow(dead_code)]

//! [X-fast Trie](https://en.wikipedia.org/wiki/X-fast_trie) is a bitwise trie to store a bounded domain of integers.
//!
//! Currently this version of the crate is using the nightly release of rust.

use std::collections::HashMap;
use std::ptr::NonNull;

type Node<T> = NonNull<TrieNode<T>>;
/// Nodes of the trie
#[derive(Debug)]
pub struct TrieNode<T> {
    // Node key
    key: usize,
    /// Node value
    pub value: Option<T>,
    // Level at which the node is present
    level: usize,
    // Node's right subtree
    right: Option<NonNull<TrieNode<T>>>,
    // Node's left subtree
    left: Option<NonNull<TrieNode<T>>>,
    // Node's metadata about descendant node status
    is_desc_left: bool,
    is_desc_right: bool,
}

impl<T> TrieNode<T> {
    
    /// Creates a new node of the trie with `key` and `value` and no children.
    pub fn new(key: usize, value: T, level: usize) -> Box<Self> {
        Box::new(TrieNode{
            key: key,
            value: Some(value),
            level: level,
            right: None,
            left: None,
            is_desc_right: true,
            is_desc_left: true,
        })
    }

    // constructor for internal nodes
    fn new_internal(level: usize) -> Box<Self> {
        Box::new(TrieNode{
            key: 0,
            value: None,
            level,
            right: None,
            left: None,
            is_desc_left: true,
            is_desc_right: true,
        })
    }

    // return the rightmost node for @cur_node as parent
    // @max_level: max possible height of the trie
    fn get_rightmost_node(max_level: usize, mut cur_node: *mut TrieNode<T>) -> Option<Node<T>> {
        unsafe {
            while (*cur_node).level != max_level {
                match (*cur_node).right {
                    Some(right_node) => {
                        cur_node = right_node.as_ptr();
                    }
                    None => {
                        (*cur_node).left.map(|left_node| {
                            cur_node = left_node.as_ptr();
                        });
                    }
                }
            }
            NonNull::new(cur_node as *mut TrieNode<T>)
        }
    }

    // return the leftmost node for @cur_node as parent
    // @max_level: max possible height of the trie
    fn get_leftmost_node(max_level: usize, mut cur_node: *mut TrieNode<T>) -> Option<Node<T>> {
        unsafe {
            while (*cur_node).level != max_level {
                match (*cur_node).left {
                    Some(left_node) => {
                        cur_node = left_node.as_ptr();
                    }
                    None => {
                        (*cur_node).right.map(|right_node| {
                            cur_node = right_node.as_ptr();
                        });
                    }
                }
            }
            NonNull::new(cur_node as *mut TrieNode<T>)
        }
    }
}

#[derive(Debug)]
/// A bitwise trie to store integers.
///
/// The values in a X-fast trie are stored at the leaves. An internal node is added to the trie
/// only if it has leaves in its subtree.
///
/// Each level of the trie is modelled as a hash map storing the trie nodes at that level.
///
/// The range of integers need to be specified while initializing a trie.
/// # Examples
/// ```
///     use xfast::Xfast;
///
///     let mut test_trie: Xfast<&str> = Xfast::new(31);
///     test_trie.insert_key(11, "eleven");
///     test_trie.insert_key(1, "one");
///     test_trie.insert_key(5, "five");
///     assert_eq!(test_trie.len(), 3);
///        
///     let predecessor_3 = test_trie.find_predecessor(3);
///     if predecessor_3.is_some() {
///         let predecessor_value = predecessor_3.unwrap().value.unwrap();
///         assert_eq!(predecessor_value, "one");
///     }
/// ```

pub struct Xfast<T=String> {
    nr_levels: usize,
    level_maps: Vec<HashMap<usize, NonNull<TrieNode<T>>>>,
}

impl<T> Xfast<T> {
    
    /// Creates a new Xfast Trie to store a given `range` of integers
    /// # Examples
    /// ```
    /// # #![allow(unused_mut)]
    ///     use xfast::Xfast;
    ///     
    ///     let mut test_trie: Xfast<&str> = Xfast::new(31);
    /// ```
    pub fn new(range: usize) -> Self {
        let nr_levels = Self::get_levels_count(range);
        let level_maps = Self::create_map_list(nr_levels+1);
        let mut new_trie = Xfast {
            nr_levels,
            level_maps,
        };
        // insert the root node in the trie at level 0
        let root_node = TrieNode::new_internal(0);
        let root_node = NonNull::from(Box::leak(root_node));
        new_trie.level_maps[0].insert(0, root_node);
        new_trie
    }

    // levels => height of the trie
    fn get_levels_count(mut range: usize) -> usize {
        let mut levels = 0;
        while range > 0 {
            range >>= 1;
            levels += 1;
        }
        levels
    }

    // helper fn for populating a vector list of hashmaps
    fn create_map_list(nr_levels: usize) -> Vec<HashMap<usize, Node<T>>> {
        (0..nr_levels).map(|_| HashMap::new()).collect()
    }

    /// Returns the count of values stored in the trie
    /// # Examples
    /// ```
    ///     use xfast::Xfast;
    ///
    ///     let mut test_trie: Xfast<&str> = Xfast::new(31);
    ///     test_trie.insert_key(11, "eleven");
    ///     test_trie.insert_key(1, "one");
    ///     assert_eq!(test_trie.len(), 2);
    /// ```
    pub fn len(&self) -> usize {
        self.level_maps[self.nr_levels].len()
    }

    fn find_lowest_common_ancestor(&self, key: usize) -> Option<*mut TrieNode<T>> {
        let mut low = 0;
        let mut high = self.nr_levels;
        let mut ancestor_node: Option<*mut TrieNode<T>> = None;

        while high >= low {
            let mid = (low + high)/2;
            let prefix = key >> (self.nr_levels - mid);
            //check the presence of an internal node with the keyed as `prefix` in hashmap at the `mid` level 
            match self.level_maps[mid].get(&prefix) {
                Some(&value) => {
                    low = mid + 1;
                    ancestor_node = Some(value.as_ptr());
                }
                None => {
                    // prevent out of bound subtraction of a usize
                    if mid == 0 {
                        break;
                    }
                    high = mid - 1;
                }
            }
        }
        ancestor_node
    }

    /// Returns the smallest node more than or eqaul to the node associated with `key`. In case of no such node it returns None.
    /// #Examples
    /// # Examples
    /// ```
    ///     use xfast::Xfast;
    ///
    ///     let mut test_trie: Xfast<&str> = Xfast::new(31);
    ///     test_trie.insert_key(11, "eleven");
    ///     test_trie.insert_key(1, "one");
    ///     test_trie.insert_key(5, "five");
    ///     assert_eq!(test_trie.len(), 3);
    ///     
    ///     if let Some(successor_3) = test_trie.find_successor(3) {
    ///         let successor_value = successor_3.value.unwrap();
    ///         assert_eq!(successor_value, "five");
    ///     }
    ///     let successor_14 = test_trie.find_successor(14);
    ///         assert!(successor_14.is_none());
    /// ```
    pub fn find_successor(&self, key: usize) -> Option<&TrieNode<T>> {
        // find the lowest common ancestor- a node which shares maximum common prefix with the key
        let successor_node: Option<*mut TrieNode<T>> = self.find_lowest_common_ancestor(key);
        if let Some(node) = successor_node {
            unsafe {
                // successor of a key already present is the key itself
                if (*node).level == (self.nr_levels) {
                    return Some(&(*node));
                }

                //right subtree of an internal node can have the successor
                let mut updated_node = None;
                if (key >> (self.nr_levels - (*node).level -1 ) & 1) != 0 {
                    (*node).right.map(|right_node| {
                        updated_node = Some(right_node.as_ptr());
                    });
                }
                else {
                    //left subtree of the internal node has the successor
                    (*node).left.map(|left_node| {
                        updated_node = Some(left_node.as_ptr());
                    });
                }
                                
                // in case the key of the successor node (leaf node) above calculated has lower key than the currently searched key
                // navigate using the right and left pointer of the leaf node to find the smallest node which has a key >= the key being searched
                if !updated_node.is_none() && (*updated_node.unwrap()).key < key {
                    let mut temp_node = None;
                    (*updated_node.unwrap()).right.map(|right_node| {
                        temp_node = Some(&(*right_node.as_ptr()));
                    });
                    return temp_node;
                }
                if !updated_node.is_none() {
                    return Some(&(*updated_node.unwrap()));
                }
                return None;
            }
        }
        None
    }

    /// Returns the largest node less that or eqaul to the node with `key`. In case of no such node it returns None.
    /// #Examples
    /// # Examples
    /// ```
    ///     use xfast::Xfast;
    ///
    ///     let mut test_trie: Xfast<&str> = Xfast::new(31);
    ///     test_trie.insert_key(11, "eleven");
    ///     test_trie.insert_key(1, "one");
    ///     test_trie.insert_key(5, "five");
    ///     assert_eq!(test_trie.len(), 3);
    ///        
    ///     if let Some(predecessor_3) = test_trie.find_predecessor(3) {
    ///         let predecessor_value = predecessor_3.value.unwrap();
    ///         assert_eq!(predecessor_value, "one");
    ///     }
    ///
    ///     let predecessor_0 = test_trie.find_predecessor(0);
    ///         assert!(predecessor_0.is_none());
    /// ```
    pub fn find_predecessor(&self, key: usize) -> Option<&TrieNode<T>> {
        // find the lowest common ancestor- a node which shares maximum common prefix with the key
        let predecessor_node: Option<*mut TrieNode<T>> = self.find_lowest_common_ancestor(key);
        if let Some(node) = predecessor_node {
            unsafe {
                // predecessor of a key already present is the key itself
                if (*node).level == (self.nr_levels) {
                    return Some(&(*node));
                }

                let mut updated_node = None;
                if (key >> (self.nr_levels - (*node).level -1) & 1) != 0 {
                    (*node).right.map(|right_node| {
                        updated_node = Some(right_node.as_ptr());
                    });
                }
                else {
                    (*node).left.map(|left_node| {
                        updated_node = Some(left_node.as_ptr());
                    });
                }

                if !updated_node.is_none() && (*updated_node.unwrap()).key > key {
                    let mut temp_node = None;
                    (*updated_node.unwrap()).left.map(|left_node| {
                        temp_node = Some(&(*left_node.as_ptr()));
                    });
                    return temp_node;
                }
                if !updated_node.is_none() {
                return Some(&(*updated_node.unwrap()));
                }
                return None;
                }
            }
        None
    }

    fn populate_internal_nodes(&mut self, key: usize) {
        let mut level = 1;
        let max_levels = self.nr_levels;
        while level < max_levels {
            let prefix = key >> (max_levels - level);
            if let None = self.level_maps[level].get(&prefix) {
                let temp_node = TrieNode::new_internal(level);
                let temp_node = NonNull::from(Box::leak(temp_node));
                self.level_maps[level].insert(prefix, temp_node);
                // add to the right child if the bit is 1 at that index else make it the left child
                if (prefix & 1) != 0 {
                    let temp_prefix = prefix >> 1;
                    self.level_maps[level-1].get(&temp_prefix).map(|&value| unsafe{
                        (*value.as_ptr()).right = Some(temp_node);
                        (*value.as_ptr()).is_desc_right = false;
                    });
                }
                else {
                    let temp_prefix = prefix >> 1;
                    self.level_maps[level-1].get(&temp_prefix).map(|&value| unsafe{
                        (*value.as_ptr()).left = Some(temp_node);
                        (*value.as_ptr()).is_desc_left = false;
                    }); 
                }
            }
            level += 1;
        }
    }

    fn update_descendant_ptr(&mut self, key: usize) {
        let mut prefix = key;
        let mut level = self.nr_levels - 1;

        while level > 0 {
            prefix = prefix >> 1;
            // find an internal node prefixed as `prefix` at `level` in the level_map
            self.level_maps[level].get(&prefix).map(|&value| unsafe {
                //check if this node has a left child
                match (*value.as_ptr()).left {
                    //the internal node doesn't have a left child
                    None => {
                        //An internal node is inserted in a trie only when it has one its children
                        //Therefore, this node has a right child which is used to find its descendant ptr
                        (*value.as_ptr()).right.map(|right_node| {
                            (*value.as_ptr()).left = TrieNode::get_leftmost_node(self.nr_levels, right_node.as_ptr());
                            (*value.as_ptr()).is_desc_left = true;
                        });
                    },
                    // Left child is present
                    Some(left_ptr) => {
                        //this internal node can have a right child or not
                        match (*value.as_ptr()).right {
                            // the right child is not present
                            None => {
                                (*value.as_ptr()).right = TrieNode::get_rightmost_node(self.nr_levels, left_ptr.as_ptr());
                                (*value.as_ptr()).is_desc_right = true;
                            }
                            // right child is also present
                            Some(right_ptr)=> {
                                // if any of the left or the right child is associated with a descendant pointer then update with the latest descendant pointer. 
                                // At any given instance only one descendant ptr can be present
                                if (*value.as_ptr()).is_desc_right {
                                    (*value.as_ptr()).right = TrieNode::get_rightmost_node(self.nr_levels, left_ptr.as_ptr());
                                }
                                else if (*value.as_ptr()).is_desc_left {
                                    (*value.as_ptr()).left = TrieNode::get_leftmost_node(self.nr_levels, right_ptr.as_ptr());
                                }
                            }
                        }
                    }
                }
            });
            level -= 1;
        }

        // update the descendant ptr for the root node
        self.level_maps[0].get(&0).map(|&value| unsafe {
            let is_left_descendant = (*value.as_ptr()).is_desc_left;
            let is_right_descendant = (*value.as_ptr()).is_desc_right;
            if is_left_descendant {
                (*value.as_ptr()).right.map(|right_node| {
                    (*value.as_ptr()).left = TrieNode::get_leftmost_node(self.nr_levels, right_node.as_ptr());
                });
            }
            if is_right_descendant {
                (*value.as_ptr()).left.map(|left_node| {
                    (*value.as_ptr()).right = TrieNode::get_rightmost_node(self.nr_levels, left_node.as_ptr());
                });
            }
        });
    }

    /// Insert `key` and `value` into the trie
    /// # Examples
    /// ```
    ///     use xfast::Xfast;
    ///
    ///     let mut test_trie: Xfast<&str> = Xfast::new(31);
    ///     test_trie.insert_key(11, "eleven");
    /// ```
    pub fn insert_key(&mut self, key: usize, value: T) {
        //create a new node with key and val
        let new_node = TrieNode::new(key, value, self.nr_levels);
        let new_node = Some(NonNull::from(Box::leak(new_node)));
        //find predecessor and successor for the new node
        let predecessor = self.find_predecessor(key);
        let successor = self.find_successor(key);
        
        //update the right and left pointers of the new node to refer to its successors and predecessors resp.
        //update the right ptr in the predecessor ,and left ptr in the successor with the new_node.
        predecessor.map(|pred_node| unsafe{
            //FIXME
            let pred_node = &(*pred_node) as *const TrieNode<T> as *mut TrieNode<T>;
            new_node.map(|node| {
                (*node.as_ptr()).right = (*pred_node).right;
                (*node.as_ptr()).left = NonNull::new(pred_node);
            });
            (*pred_node).right = new_node;
        });

        successor.map(|suc_node| unsafe{
            let suc_node = &(*suc_node) as *const TrieNode<T> as *mut TrieNode<T>;
            new_node.map(|node| {
                (*node.as_ptr()).left = (*suc_node).left;
                (*node.as_ptr()).right = NonNull::new(suc_node);
            });
            (*suc_node).left = new_node;
        });

        //populate intermediate iternal nodes on the path down the new_node
        self.populate_internal_nodes(key);
        
        //insert the new_node at the last level and update the ptr of its parent node using the prefix bit
        self.level_maps[self.nr_levels].insert(key, new_node.unwrap());
        let temp_key = key >> 1;
        self.level_maps[self.nr_levels-1].get(&temp_key).map(|&value| unsafe {
            if (key & 1) != 0 {
                (*value.as_ptr()).right= new_node;
                (*value.as_ptr()).is_desc_right = false;
            }
            else {
                (*value.as_ptr()).left = new_node;
                (*value.as_ptr()).is_desc_left = false;
            }
        });

        // update descendant ptrs
        self.update_descendant_ptr(key);
    }

    fn delete_internal_node(&mut self, key: usize) {
        let mut level = self.nr_levels-1;
        let mut prefix = key;
        let mut child_prefix = key;

        while level > 0 {
            prefix = prefix >> 1;
            if let Some(internal_node) = self.level_maps[level].get(&prefix) {
                unsafe {
                    //check if it has a descendant node
                    if (child_prefix &1) == 1 {
                        //check left node
                        if !(*internal_node.as_ptr()).is_desc_left {
                            break;
                        }
                    }
                    else if !(*internal_node.as_ptr()).is_desc_right {
                            break;
                    }
                }
            }
                    
            let parent_prefix = prefix >> 1;
            self.level_maps[level-1].get(&parent_prefix).map(|parent_node| unsafe{
                //node present in right subtree
                if (prefix & 1) != 0 {
                    (*parent_node.as_ptr()).right = None;
                    (*parent_node.as_ptr()).is_desc_right = true;
                }
                else {
                    (*parent_node.as_ptr()).left = None;
                    (*parent_node.as_ptr()).is_desc_left = true;
                }
            });
            self.level_maps[level].remove(&prefix);
            child_prefix = child_prefix>>1;
            level -= 1;
        }
    }

    /// Delete a key from the trie. If the node doesn't exist it returns None else retuns the deleted   `TrieNode` wrapped in a `NonNull` struct.
    /// # Examples
    ///  ```
    ///     use xfast::Xfast;
    ///
    ///     let mut test_trie: Xfast<&str> = Xfast::new(31);
    ///     test_trie.insert_key(11, "eleven");
    ///     test_trie.insert_key(1, "one");
    ///     test_trie.insert_key(5, "five");
    ///     assert_eq!(test_trie.len(), 3);
    ///        
    ///     test_trie.delete_key(5);
    ///     assert_eq!(test_trie.len(), 2);
    ///     assert!(test_trie.delete_key(2).is_none());
    ///     assert_eq!(test_trie.len(), 2);
    /// ```
    pub fn delete_key(&mut self, key: usize) -> Option<Node<T>>{
        //find the key in the lowest level
        let deleted_node = self.find_key_as_non_null(key);
        if deleted_node.is_none() {
            return None;
        }

        let deleted_node = deleted_node.unwrap();
        
        self.level_maps[self.nr_levels-1].get(&(key>>1)).map(|internal_node| unsafe{
            if (key &1) == 1 {
                (*internal_node.as_ptr()).right = None;
                (*internal_node.as_ptr()).is_desc_right = true;
            }
            else {
                (*internal_node.as_ptr()).left = None;
                (*internal_node.as_ptr()).is_desc_left = true;
            }
        });
        
        self.delete_internal_node(key);
        unsafe {
            let predecessor_node = (*deleted_node.as_ptr()).left;
            let successor_node = (*deleted_node.as_ptr()).right;
            
            if !predecessor_node.is_none() {
                (*predecessor_node.unwrap().as_ptr()).right = successor_node;
            }
            if !successor_node.is_none() {

                (*successor_node.unwrap().as_ptr()).left = predecessor_node;
            }
        }
        let deleted_node = self.level_maps[self.nr_levels].remove(&key);
        self.update_descendant_ptr(key);
        deleted_node
    }

    fn find_key_as_non_null(&self, key: usize) -> Option<Node<T>> {
        self.level_maps[self.nr_levels].get(&key).map(|&value| {
            value
        })
    }

    /// Find a key in the trie
    /// # Examples
    ///  ```
    ///     use xfast::Xfast;
    ///
    ///     let mut test_trie: Xfast<&str> = Xfast::new(31);
    ///     test_trie.insert_key(11, "eleven");
    ///     test_trie.insert_key(1, "one");
    ///     if let Some(node_1) = test_trie.find_key(1) {
    ///         // all the leaf nodes values have non trivial values and assert is_some. 
    ///         // So unwrapping will not panic
    ///         assert_eq!(node_1.value.unwrap(), "one");
    ///     }
    /// ```
    pub fn find_key(&self, key: usize) -> Option<&TrieNode<T>> {
        self.level_maps[self.nr_levels].get(&key).map(|&value| unsafe {
            &(*value.as_ptr())
        })
    }

    /// Returns an iterator around all the key-TrieNode pairs stored in the trie.
    /// # Examples
    /// ```
    ///     use xfast::Xfast;
    ///
    ///     let mut test_trie: Xfast<&str> = Xfast::new(31);
    ///     test_trie.insert_key(11, "eleven");
    ///     test_trie.insert_key(1, "one");
    ///     test_trie.insert_key(19, "nineteen");
    ///     for (key, node) in test_trie.iter() {
    ///         println!("key: {} value: {:?}", key, node);
    ///     }
    /// ```
    pub fn iter(&self) -> XfastIter<T> {
        let leaf_map = &self.level_maps[self.nr_levels];
        let mut keys: Vec<usize> = vec!();
        
        for &cur_key in leaf_map.keys() {
            keys.push(cur_key);
        }

        XfastIter {
            leaf_map,
            keys,
            index: 0,
        }
    }

    /// Returns a mutable iterator around all the key-TrieNode pairs stored in the trie.
    /// # Examples
    /// ```
    ///     use xfast::Xfast;
    ///
    ///     let mut test_trie: Xfast<&str> = Xfast::new(31);
    ///     test_trie.insert_key(11, "eleven");
    ///     test_trie.insert_key(1, "one");
    ///     test_trie.insert_key(19, "nineteen");
    ///     for (key, node) in test_trie.iter_mut() {
    ///         if key % 2 == 1 {
    ///             node.value = Some("updated_odd");    
    ///         }
    ///     }
    ///
    ///     if let Some(node_1) = test_trie.find_key(1) {
    ///         assert_eq!(node_1.value.unwrap(), "updated_odd");
    ///     }
    /// ```
    pub fn iter_mut(&mut self) -> XfastIterMut<T> {
        let leaf_map = &self.level_maps[self.nr_levels];
        let mut keys: Vec<usize> = vec!();
        
        for &cur_key in leaf_map.keys() {
            keys.push(cur_key);
        }

        XfastIterMut {
            leaf_map,
            keys,
            index: 0,
        }
    }
}

/// Iterator around the Xfast key and value (TrieNode) pairs

pub struct XfastIter<'a, T> {
    leaf_map: &'a HashMap<usize, Node<T>>,
    keys: Vec<usize>,
    index: usize,
}

impl<'a, T> Iterator for XfastIter<'a, T> {
    type Item = (&'a usize, &'a TrieNode<T>);

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.leaf_map.len() {
            let key = self.keys[self.index];
            self.index += 1;
            let kv_pair = self.leaf_map.get_key_value(&key);
            kv_pair.map(|(key, value)| unsafe{
                let value = value.as_ref();
                (key, value)
            })
        }
        else {
            None
        }
    }
}

/// Mutable Iterator around the Xfast key and value (TrieNode) pairs
pub struct XfastIterMut<'a, T> {
    leaf_map: &'a HashMap<usize, Node<T>>,
    keys: Vec<usize>,
    index: usize,
}

impl<'a, T> Iterator for XfastIterMut<'a, T> {
    type Item = (&'a usize, &'a mut TrieNode<T>);

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.leaf_map.len() {
            let key = self.keys[self.index];
            self.index += 1;
            let kv_pair = self.leaf_map.get_key_value(&key);
            kv_pair.map(|(key, value)| unsafe{
                let value = &mut (*value.as_ptr());
                (key, value)
            })
        }
        else {
            None
        }
    }
}

impl<'a, T> IntoIterator for &'a Xfast<T> {
    type Item = (&'a usize, &'a TrieNode<T>);
    type IntoIter = XfastIter<'a, T>;
    fn into_iter(self) -> XfastIter<'a, T> {
        self.iter()
    }
}



mod test{
    use super::Xfast;

    fn init()  -> Xfast<String> {
        let mut test_trie: Xfast<String> = Xfast::new(31);
        test_trie.insert_key(11, String::from("eleven"));
        test_trie.insert_key(1, String::from("one"));
        test_trie.insert_key(18, String::from("eighteen"));
        test_trie.insert_key(5, String::from("five"));
        test_trie
    }

    #[test]
    fn successor() -> Result<(), String> {
        let test_trie = init();
        if let Some(successor) = test_trie.find_successor(7) {
            if successor.key == 11 {
                return Ok(())
            }
        }
        Err(String::from("Successor of 7 is wrong"))
    }

    #[test]
    fn none_successor() -> Result<(), String> {
        let test_trie = init();
        if test_trie.find_successor(19).is_none() {
            Ok(())
        }
        else {
            Err(String::from("Successor of 19 is wrong"))
        }
    }

    #[test]
    fn predecessor() -> Result<(), String> {
        let test_trie = init();
        if let Some(predecessor) = test_trie.find_predecessor(8) {
            if predecessor.key == 5 {
                return Ok(())
            }
        }
        Err(String::from("Predecessor of 8 is wrong"))
    }

    #[test]
    fn none_predecessor() -> Result<(), String> {
        let test_trie = init();
        if test_trie.find_predecessor(0).is_none() {
            Ok(())
        }
        else {
            Err(String::from("Predecessor of 1 is wrong"))
        }
    }

    #[test]
    fn find_key_present() -> Result<(), String> {
        let test_trie = init();
        if let Some(value) = test_trie.find_key(11) {
            if value.key == 11 {
                    return Ok(());
            }
        }
        Err(String::from("Key should have been present"))
    }

    #[test]
    fn find_key_not_present() -> Result<(), String> {
        let test_trie = init();
        if test_trie.find_key(7).is_none() {
            return Ok(());
        }
        Err(String::from("Key should not have been present"))
    }

    #[test]
    fn delete_node() -> Result<(), String> {
        let mut test_trie = init();
        test_trie.delete_key(18);
        if test_trie.find_key(18).is_none() {
            Ok(())
        }
        else {
            Err(String::from("Key should have been deleted"))
        }   
    }

    #[test]
    fn successor_after_del() -> Result<(), String> {
        let mut test_trie = init();
        test_trie.delete_key(18);
        if test_trie.find_successor(18).is_none() {
            Ok(())
        }
        else {
            Err(String::from("Successor of 18 is wrong"))
        }
    }

    #[test]
    fn predecessor_after_del() -> Result<(), String> {
        let mut test_trie = init();
        test_trie.delete_key(18);
        if let Some(predecessor) = test_trie.find_predecessor(18) {
            if predecessor.key == 11 {
                return Ok(());
            }
        }
        Err(String::from("Successor of 18 is wrong"))
    }

    #[test]
    fn deleting_non_existent() -> Result<(), String> {
        let mut test_trie = init();
        if test_trie.delete_key(19).is_none() {
            Ok(())
        }
        else {
            Err(String::from("The deleted node didn't exist!!"))
        }
    }
}