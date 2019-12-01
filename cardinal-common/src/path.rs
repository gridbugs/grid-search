use crate::step::Step;
use direction::CardinalDirection;
use grid_2d::Coord;
use std::collections::{vec_deque, VecDeque};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PathNode {
    pub to_coord: Coord,
    pub in_direction: CardinalDirection,
}

pub struct PathIter<'a> {
    iter: vec_deque::Iter<'a, Step>,
}

impl<'a> Iterator for PathIter<'a> {
    type Item = PathNode;
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|step| PathNode {
            to_coord: step.to_coord,
            in_direction: step.in_direction.to_cardinal_direction(),
        })
    }
}

pub struct Path {
    steps: VecDeque<Step>,
}

impl Path {
    pub fn iter(&self) -> PathIter {
        PathIter {
            iter: self.steps.iter(),
        }
    }
    pub fn len(&self) -> usize {
        self.steps.len()
    }
    pub(crate) fn clear(&mut self) {
        self.steps.clear();
    }
    pub(crate) fn prepend(&mut self, step: Step) {
        self.steps.push_front(step);
    }
}
