use super::{cache::Cache, branch::Branch};

pub type Tree = Cache<Branch>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tree_new() {
        let cache = Tree::new(10);
        assert_eq!(cache.capacity(), 10);
    }
}
