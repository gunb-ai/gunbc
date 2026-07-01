pub mod fact_cardinality_census;
pub mod medium_structure_census;

pub use fact_cardinality_census::{
    cross_tree_coexistence_count, cross_tree_diverged_fork_count, cross_tree_is_coexistence,
    cross_tree_is_diverged_fork,
};
pub use medium_structure_census::medium_structure_leak_facts;
