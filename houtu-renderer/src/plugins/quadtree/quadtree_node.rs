use std::ops::AddAssign;

use bevy::utils::HashSet;

use super::{quadtree_value::QuadtreeValue, THRESHOLD};
pub trait QuadtreeNodeTrait {}

#[derive(Default, Debug)]
pub struct QuadtreeNode<T: QuadtreeValue> {
    pub depth: usize,
    pub southwestChild: Option<QuadtreeNode<T>>,
    pub southeastChild: Option<QuadtreeNode<T>>,
    pub northwestChild: Option<QuadtreeNode<T>>,
    pub northeastChild: Option<QuadtreeNode<T>>,
    pub parent: Option<QuadtreeNode<T>>,
    pub value: T,
}

impl<T: QuadtreeValue> QuadtreeNode<T> {
    pub fn empty() -> Self {
        QuadtreeNode {
            ..Default::default()
        }
    }

    pub fn is_leaf(&self) -> bool {
        self.southeastChild.is_none()
            && self.southeastChild.is_none()
            && self.northwestChild.is_none()
            && self.northeastChild.is_none()
    }

    // loop through self and all descendents, run aggregation function and return summed result
    pub fn aggregate_statistic<AggT: AddAssign<AggT>, AggFn: Fn(&QuadtreeNode<T>) -> AggT>(
        &self,
        agg_func: &AggFn,
    ) -> AggT {
        let mut agg_value: AggT = agg_func(self);
        for child in &self.children {
            agg_value += child.aggregate_statistic(agg_func);
        }
        return agg_value;
    }

    // add value to self if room, otherwise propagate to children, fall back to self if needed
    pub fn add(&mut self, value: T) {
        if self.is_leaf() {
            if self.values.len() < THRESHOLD {
                self.values.insert(value);
            } else {
                self.create_children();
                self.distribute_values();
                self.add(value);
            }
        } else {
            if self.values.len() < THRESHOLD {
                self.values.insert(value);
            } else if let Some(child) = self.get_child_containing_rect_mut(value.get_rect()) {
                child.add(value);
            } else {
                self.values.insert(value);
            }
        }
    }

    pub fn contains_value(&self, value: &T) -> bool {
        self.values.contains(value)
    }
    pub fn delete(&mut self, value: &T) -> Option<T> {
        // clean up children if needed
        if !self.is_leaf() {
            let delete_children = self.children.iter().all(|child| child.values.is_empty());
            if delete_children {
                self.children.clear();
            }
        }
        // delete value
        self.values.take(value)
    }

    pub fn get_all_descendant_nodes(&self) -> Box<dyn Iterator<Item = &QuadtreeNode<T>> + '_> {
        Box::new(
            self.children
                .iter()
                .filter(|c| !c.is_leaf())
                .flat_map(|c| c.get_all_descendant_nodes()),
        )
    }

    pub fn get_all_descendant_values(&self) -> Box<dyn Iterator<Item = &T> + '_> {
        Box::new(
            self.values.iter().chain(
                self.get_all_descendant_nodes()
                    .flat_map(|c| c.values.iter()),
            ),
        )
    }
    fn distribute_values(&mut self) {
        if self.children.len() == 0 {
            return;
        }
        let values: Vec<T> = self.values.drain().collect();
        for value in values {
            if let Some(child) = self.get_child_containing_rect_mut(value.get_rect()) {
                child.add(value);
            } else {
                self.add(value);
            }
        }
    }
}
