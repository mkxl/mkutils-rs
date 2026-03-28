use crate::utils::Utils;
use mkutils_macros::Default as MkutilsDefault;
use num::{Zero, traits::SaturatingAdd};
use std::{
    collections::{
        BTreeMap,
        btree_map::{IntoIter as BTreeMapIntoIter, Range as BTreeMapRange, Values as BTreeMapValues},
    },
    iter::Map,
};

pub trait Interval {
    type Point: Ord + PartialEq + SaturatingAdd + Zero;

    fn begin(&self) -> Self::Point;

    fn end(&self) -> Self::Point {
        self.begin().saturating_add(self.len())
    }

    fn len(&self) -> Self::Point;

    fn is_empty(&self) -> bool {
        self.len().is_zero()
    }

    fn expand_to_cover(&mut self, other: &Self);

    fn is_touching_or_contains(&self, point: &Self::Point) -> bool {
        self.begin().range_from_len(self.len()).contains(point)
    }
}

// NOTE:
// - use [MkutilsDefault] as [std::default::Default] adds a [T: Default] trait bound
// - maintains the below invariants
//   - non-empty intervals are normalized: they never overlap or touch
//   - inserting a non-empty interval merges any touching/overlapping non-empty intervals
//   - an empty interval never coexists with a non-empty interval that it is touching or contains
//     (see [Interval::is_touching_or_contains])
#[derive(Clone, Debug, MkutilsDefault)]
pub struct IntervalSet<T: Interval> {
    intervals: BTreeMap<T::Point, T>,
}

impl<T: Interval> IntervalSet<T> {
    const fn from_intervals(intervals: BTreeMap<T::Point, T>) -> Self {
        Self { intervals }
    }

    #[must_use]
    pub const fn new() -> Self {
        Self::from_intervals(BTreeMap::new())
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.intervals.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.intervals.is_empty()
    }

    pub fn clear(&mut self) {
        self.intervals.clear();
    }

    #[must_use]
    pub fn first(&self) -> Option<&T> {
        self.intervals.first_key_value()?.into_second().some()
    }

    #[must_use]
    pub fn last(&self) -> Option<&T> {
        self.intervals.last_key_value()?.into_second().some()
    }

    pub fn pop_first(&mut self) -> Option<T> {
        self.intervals.pop_first()?.into_second().some()
    }

    pub fn pop_last(&mut self) -> Option<T> {
        self.intervals.pop_last()?.into_second().some()
    }

    pub fn iter(&self) -> BTreeMapValues<'_, T::Point, T> {
        self.intervals.values()
    }

    #[allow(clippy::type_complexity)]
    fn intervals_with_begin_leq<'a>(
        &'a self,
        point: &T::Point,
    ) -> Map<BTreeMapRange<'a, T::Point, T>, fn((&'a T::Point, &'a T)) -> &'a T> {
        self.intervals.range(..=point).map(Utils::into_second)
    }

    pub fn is_touching_or_contains(&self, point: &T::Point) -> bool {
        if let Some(furthest_leq_interval) = self.intervals_with_begin_leq(point).next_back() {
            furthest_leq_interval.is_touching_or_contains(point)
        } else {
            false
        }
    }

    fn insert_empty_interval(&mut self, empty_interval: T) {
        let begin = empty_interval.begin();

        if self.is_touching_or_contains(&begin) {
            return;
        }

        self.intervals.insert(begin, empty_interval);
    }

    pub fn split(self, point: &T::Point) -> (Self, Self) {
        let mut lt_intervals = self.intervals;
        let geq_intervals = lt_intervals.split_off(point);
        let lt_interval_set = Self::from_intervals(lt_intervals);
        let geq_interval_set = Self::from_intervals(geq_intervals);

        lt_interval_set.pair(geq_interval_set)
    }

    fn insert_non_empty_interval(&mut self, mut interval: T) {
        let (mut lt_interval_set, mut geq_interval_set) = self.mem_take().split(&interval.begin());

        // NOTE: if [furthest_lt_interval] touches or overlaps [interval], then merge it with [interval] and remove
        // [furthest_lt_interval]
        if let Some(furthest_lt_interval) = lt_interval_set.last()
            && furthest_lt_interval.is_touching_or_contains(&interval.begin())
        {
            interval.expand_to_cover(furthest_lt_interval);
            lt_interval_set.pop_last();
        }

        while let Some(nearest_geq_interval) = geq_interval_set.first() {
            // NOTE: if [nearest_geq_interval] is touching or overlaps with [interval], then merge them and remove
            // [nearest_geq_interval]
            if interval.is_touching_or_contains(&nearest_geq_interval.begin()) {
                interval.expand_to_cover(nearest_geq_interval);
                geq_interval_set.pop_first();

                continue;
            }

            break;
        }

        lt_interval_set.intervals.insert(interval.begin(), interval);
        lt_interval_set.intervals.append(&mut geq_interval_set.intervals);
        self.assign(lt_interval_set);
    }

    pub fn insert(&mut self, interval: T) -> &mut Self {
        if interval.is_empty() {
            self.insert_empty_interval(interval);
        } else {
            self.insert_non_empty_interval(interval);
        }

        self
    }
}

impl<T: Interval> IntoIterator for IntervalSet<T> {
    type Item = T;
    type IntoIter = Map<BTreeMapIntoIter<T::Point, T>, fn((T::Point, T)) -> T>;

    fn into_iter(self) -> Self::IntoIter {
        self.intervals.into_iter().map(Utils::into_second)
    }
}

impl<'a, T: Interval> IntoIterator for &'a IntervalSet<T> {
    type Item = &'a T;
    type IntoIter = BTreeMapValues<'a, T::Point, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
