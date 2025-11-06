use crate::utils::Utils;
use derive_more::Constructor;
use std::{
    pin::Pin,
    task::{Context, Poll},
};
use tokio::task::{JoinError, JoinSet};

#[derive(Constructor)]
pub struct Join<'a, T> {
    join_set: &'a mut JoinSet<T>,
}

impl<T: 'static> Future for Join<'_, T> {
    type Output = Result<T, JoinError>;

    fn poll(mut self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Self::Output> {
        if let Poll::Ready(Some(result)) = self.join_set.poll_join_next(context) {
            result.poll_ready()
        } else {
            Poll::Pending
        }
    }
}
