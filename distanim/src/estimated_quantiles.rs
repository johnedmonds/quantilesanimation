use std::fmt::Debug;

use compactorsanim::{compactor::Compactor, compactors::Compactors};

#[derive(PartialEq, Eq, Debug)]
pub struct QuantileElement<T> {
    pub weight: usize,
    pub element: T,
}

impl<T> PartialOrd for QuantileElement<T>
where
    T: Ord,
{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<T> Ord for QuantileElement<T>
where
    T: Ord,
{
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.element.cmp(&other.element)
    }
}

pub struct EstimatedQuantiles<T> {
    pub elements: Vec<QuantileElement<T>>,
}

impl<T> EstimatedQuantiles<T> {
    pub fn estimated_element_count(&self) -> usize {
        self.elements.iter().map(|e| e.weight).sum()
    }
}

impl<T, F, const LAZY: bool> From<Compactors<T, F, LAZY>> for EstimatedQuantiles<T>
where
    T: Ord + Debug,
    F: FnMut(Vec<Compactor<T>>),
{
    fn from(compactors: Compactors<T, F, LAZY>) -> Self {
        let mut elements: Vec<QuantileElement<T>> = Into::<Vec<Compactor<T>>>::into(compactors)
            .into_iter()
            .enumerate()
            .flat_map(|(level, compactor)| {
                compactor
                    .data
                    .into_iter()
                    .map(move |element| QuantileElement {
                        weight: 1 << level.clone(),
                        element,
                    })
            })
            .collect();
        elements.sort();
        Self { elements }
    }
}
