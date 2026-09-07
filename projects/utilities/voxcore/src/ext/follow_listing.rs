/// A list aligned by index with one of a state's listings, kept in step
/// through the [`VoxExt`](crate::ext::VoxExt) hooks. A retained entity gets a
/// default entry at its index, a released one loses its entry, and a moved
/// one carries its entry along.
///
/// A list the index does not reach is left alone. A malformed ext then stays
/// malformed for the writer's count check to report instead of growing to
/// fit.
pub trait FollowListing {
    /// A default entry lands at `index`.
    fn follow_retain(&mut self, index: usize);

    /// The entry at `index` goes.
    fn follow_release(&mut self, index: usize);

    /// The entry at `from` moves to `to`, shifting the entries between them
    /// one slot.
    fn follow_move(&mut self, from: usize, to: usize);
}

impl<T: Default> FollowListing for Vec<T> {
    fn follow_retain(&mut self, index: usize) {
        if index <= self.len() {
            self.insert(index, T::default());
        }
    }

    fn follow_release(&mut self, index: usize) {
        if index < self.len() {
            self.remove(index);
        }
    }

    fn follow_move(&mut self, from: usize, to: usize) {
        if from < self.len() && to < self.len() {
            let entry = self.remove(from);
            self.insert(to, entry);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::ext::FollowListing;

    #[test]
    fn a_list_follows_retains_releases_and_moves() {
        let mut list: Vec<Option<u8>> = vec![Some(1), Some(2), Some(3)];

        list.follow_retain(1);

        assert_eq!(list, [Some(1), None, Some(2), Some(3)]);

        list.follow_release(0);

        assert_eq!(list, [None, Some(2), Some(3)]);

        list.follow_move(2, 0);

        assert_eq!(list, [Some(3), None, Some(2)]);

        list.follow_move(0, 2);

        assert_eq!(list, [None, Some(2), Some(3)]);
    }

    #[test]
    fn an_index_past_the_list_leaves_it_alone() {
        let mut list: Vec<u8> = Vec::new();

        list.follow_retain(2);

        list.follow_release(0);

        list.follow_move(0, 1);

        assert!(list.is_empty());

        list.follow_retain(0);

        assert_eq!(list, [0]);
    }
}
