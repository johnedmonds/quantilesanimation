use itertools::{EitherOrBoth, Itertools};

use crate::compactor::Compactor;
use std::{cell::RefCell, cmp::max};

const CAPACITY_COEFFICIENT: f64 = 0.7;

#[derive(Debug)]
pub struct Compactors<T, F, const LAZY: bool>
where
    T: Ord,
    F: FnMut(Vec<Compactor<T>>),
{
    k: usize,
    compactors: Vec<RefCell<Compactor<T>>>,
    num_compactions: u32,
    frame_handler: F,
}

impl<T, F, const LAZY: bool> Into<Vec<Compactor<T>>> for Compactors<T, F, LAZY>
where
    T: Ord,
    F: FnMut(Vec<Compactor<T>>),
{
    fn into(self) -> Vec<Compactor<T>> {
        self.compactors
            .into_iter()
            .map(|compactor| compactor.into_inner())
            .collect()
    }
}

fn capacity(k: usize, num_compactors: usize, level: usize) -> usize {
    ((CAPACITY_COEFFICIENT.powf((num_compactors - level - 1) as f64) * k as f64).ceil() as u32 + 1)
        as usize
}

impl<T, F, const LAZY: bool> Compactors<T, F, LAZY>
where
    T: Ord,
    T: Clone,
    F: FnMut(Vec<Compactor<T>>),
{
    pub fn new(k: usize, frame_handler: F) -> Self
    where
        F: FnMut(Vec<Compactor<T>>),
    {
        Self {
            compactors: vec![RefCell::new(Compactor::new(capacity(k, 1, 0)))],
            num_compactions: 0,
            k,
            frame_handler,
        }
    }
    pub fn update(&mut self, element: T) {
        self.compactors[0].borrow_mut().update(element);
        self.record_frame();
        self.compact();
    }
    pub fn merge<G>(&mut self, other: Compactors<T, G, LAZY>)
    where
        G: FnMut(Vec<Compactor<T>>),
    {
        let mut compactors = Vec::new();
        std::mem::swap(&mut self.compactors, &mut compactors);
        self.compactors = compactors
            .into_iter()
            .zip_longest(other.compactors.into_iter())
            .map(|element| match element {
                EitherOrBoth::Both(a, b) => {
                    a.borrow_mut().data.extend(b.into_inner().data);
                    a
                }
                EitherOrBoth::Left(a) => a,
                EitherOrBoth::Right(a) => a,
            })
            .collect();
        self.compact();
    }

    fn size(&self) -> usize {
        self.compactors.iter().map(|c| c.borrow().data.len()).sum()
    }
    fn total_capacity(&self) -> usize {
        self.compactors.iter().map(|c| c.borrow().capacity).sum()
    }
    fn capacity(&self, level: usize) -> usize {
        capacity(self.k, self.compactors.len(), level)
    }
    fn grow_to_include_level(&mut self, level: usize) {
        if self.compactors.len() - 1 >= level {
            return;
        }
        // Initialize with 0 because we reset them immediately after.
        self.compactors
            .resize_with(max(level + 1, self.compactors.len()), || {
                RefCell::new(Compactor::new(0))
            });
        for (level, compactor) in self.compactors.iter().enumerate() {
            compactor.borrow_mut().capacity = self.capacity(level);
        }
    }
    fn compact(&mut self) {
        self.num_compactions += 1;
        if LAZY {
            for level in 0..self.compactors.len() {
                if self.size() >= self.total_capacity() {
                    if self.compactors[level].borrow().data.len()
                        >= self.compactors[level].borrow().capacity
                    {
                        self.refactor_level(level);
                    }
                } else {
                    break;
                }
            }
        } else {
            for level in 0..self.compactors.len() {
                if self.compactors[level].borrow().data.len()
                    >= self.compactors[level].borrow().capacity
                {
                    self.refactor_level(level);
                }
            }
        }
    }

    fn refactor_level(&mut self, level: usize) {
        self.grow_to_include_level(level + 1);
        let use_evens = self.num_compactions % 2 == 0;
        self.compactors[level].borrow_mut().data.sort();
        self.record_frame();
        let mut compactor = Vec::new();
        std::mem::swap(
            &mut compactor,
            &mut self.compactors[level].borrow_mut().data,
        );
        self.compactors[level + 1].borrow_mut().data.extend(
            compactor
                .into_iter()
                .enumerate()
                .filter(|(i, _)| (i % 2 == 0) == use_evens)
                .map(|(_, element)| element),
        );
        self.compactors[level].borrow_mut().data = Vec::new();
        self.record_frame();
    }
    fn record_frame(&mut self) {
        (self.frame_handler)(self.compactors.iter().map(|c| c.borrow().clone()).collect());
    }
}
