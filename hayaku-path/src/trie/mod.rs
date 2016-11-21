#[derive(Clone, Debug, PartialEq)]
pub struct TrieNode<T: Clone> {
    /// The key associated with this node
    key: String,
    /// The value associated with this node, if any
    value: Option<T>,
    /// All branches from this node
    children: Vec<Box<TrieNode<T>>>,
}

impl<T: Clone> TrieNode<T> {
    /// Creates a new Trie
    pub fn new() -> Self {
        TrieNode {
            key: String::new(),
            value: None,
            children: Vec::new(),
        }
    }

    /// Create a new child node with the given key-value pair and add it
    /// as a child to node `self`.
    fn add_new_child(&mut self, key: String, value: Option<T>) {
        let child = TrieNode {
            key: key,
            value: value,
            children: Vec::new(),
        };
        self.children.push(Box::new(child));
    }

    /// Inserts a key-value pair into the trie.
    pub fn insert<S: Into<String>>(&mut self, key: S, value: T) {
        let key = key.into();
        // Empty tree, simply set key/value for this node to given key/value.
        if self.key.is_empty() {
            self.key = key;
            self.value = Some(value);
        } else {
            // Non-empty tree
            // Get the length of the match for our nodes
            // NOTE: The length of the match should always be
            // at least 1. We disallow routes that do not start
            // with '/'.
            let match_len = get_match_len(&self.key, &key);
            // If the length of the match is the length of this node's key,
            // we do not need to split the node.
            if match_len == self.key.len() {
                let key = key[match_len..].to_string();
                // This failing implies that we were given two of the same key
                assert!(!key.is_empty());
                // If there are no children, we just add a new node. No need to
                // worry about another node with a matching prefix.
                if self.children.is_empty() {
                    self.add_new_child(key, Some(value));
                } else {
                    self.insert_children(key, value);
                }
            } else {
                // Match length was less than the length of this node's key.
                // Split node into two seperate nodes
                let child_key = self.key[match_len..].to_string();
                self.key = self.key[0..match_len].to_string();
                // TODO(nokaa): Cloning should be fine since this is an Rc
                let child_value = self.value.clone();
                self.add_new_child(child_key, child_value);
                self.value = None;

                // Insert new node
                let key = key[match_len..].to_string();
                // This failing implies that we were given two of the same key
                assert!(!key.is_empty());

                if self.children.is_empty() {
                    self.add_new_child(key, Some(value));
                } else {
                    self.insert_children(key, value);
                }
            }
        }
    }

    fn insert_children(&mut self, key: String, value: T) {
        // Check all children of this node for one that has a
        // common prefix of any length. If a common prefix is
        // found, we  insert at that node.
        for mut child in &mut self.children {
            if child.is_match(&key) {
                child.insert(key, value);
                return;
            }
        }
        // No matching node found, add new child
        self.add_new_child(key, Some(value));
    }

    /// Determines if key matches this node.
    // This function is used internally for determining whether or not
    // we need to split a node or create a new node upon insertion.
    fn is_match(&self, key: &str) -> bool {
        get_match_len(&self.key, key) > 0
    }
}

/// Determines the length of the shared prefix of two strings.
/// E.g. `get_match_len("apple", "ape") => 2`.
fn get_match_len(a: &str, b: &str) -> usize {
    let mut match_len = 0;
    for (ac, bc) in a.chars().zip(b.chars()) {
        if ac == bc {
            match_len += 1;
        } else {
            break;
        }
    }
    match_len
}

#[cfg(test)]
mod test {
    use super::{get_match_len, TrieNode};

    #[test]
    fn match_len() {
        let a = "apple";
        let b = "ape";
        assert_eq!(get_match_len(a, b), 2);
    }

    #[test]
    fn single_insert() {
        let mut trie = TrieNode::new();
        trie.insert("/", "Data");

        let trie2 = TrieNode {
            key: "/".to_string(),
            value: Some("Data"),
            children: Vec::new(),
        };

        assert_eq!(trie, trie2);
    }

    #[test]
    fn multiple_insert() {
        let mut trie = TrieNode::new();
        trie.insert("/", "Data");
        trie.insert("/2", "Data2");

        let trie2 = TrieNode {
            key: "/".to_string(),
            value: Some("Data"),
            children: vec![Box::new(TrieNode {
                               key: "2".to_string(),
                               value: Some("Data2"),
                               children: Vec::new(),
                           })],
        };

        assert_eq!(trie, trie2);
    }

    #[test]
    fn split_node() {
        let mut trie = TrieNode::new();
        trie.insert("/1", "Data");
        trie.insert("/2", "Data2");

        let trie2 = TrieNode {
            key: "/".to_string(),
            value: None,
            children: vec![Box::new(TrieNode {
                               key: "1".to_string(),
                               value: Some("Data"),
                               children: Vec::new(),
                           }),
                           Box::new(TrieNode {
                               key: "2".to_string(),
                               value: Some("Data2"),
                               children: Vec::new(),
                           })],
        };

        assert_eq!(trie, trie2);
    }
}
