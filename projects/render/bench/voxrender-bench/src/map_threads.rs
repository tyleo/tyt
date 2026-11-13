use crate::load_threads;
use std::{
    sync::{
        Mutex,
        atomic::{AtomicUsize, Ordering},
    },
    thread,
};

/// What `work` gives for each index under `count`, in the indices' order,
/// worked out on a load's threads.
pub fn map_threads<T: Send>(count: usize, work: impl Fn(usize) -> T + Sync) -> Vec<T> {
    let made: Mutex<Vec<Option<T>>> = Mutex::new((0..count).map(|_| None).collect());
    let next = AtomicUsize::new(0);

    thread::scope(|scope| {
        for _ in 0..load_threads().min(count) {
            scope.spawn(|| {
                loop {
                    let index = next.fetch_add(1, Ordering::Relaxed);

                    if index >= count {
                        break;
                    }

                    let value = work(index);

                    made.lock().expect("no thread panics with the lock")[index] = Some(value);
                }
            });
        }
    });

    made.into_inner()
        .expect("the threads ended")
        .into_iter()
        .map(|value| value.expect("each index was worked"))
        .collect()
}

#[cfg(test)]
mod tests {
    use crate::map_threads;

    #[test]
    fn each_index_keeps_its_place() {
        assert_eq!(
            map_threads(100, |index| index * 2),
            (0..200).step_by(2).collect::<Vec<_>>()
        );
        assert!(map_threads(0, |index| index).is_empty());
    }
}
