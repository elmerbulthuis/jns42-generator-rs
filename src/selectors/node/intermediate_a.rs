use super::NodeSelectors;
use crate::schemas;

impl NodeSelectors for schemas::intermediate_a::Node {
    fn select_is_empty(&self) -> bool {
        if self.super_node_id.is_none() {
            return false;
        }

        if !self.compounds.is_empty() {
            return false;
        }

        if !self.types.is_empty() {
            return false;
        }

        true
    }
}