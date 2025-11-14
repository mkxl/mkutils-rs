use tokio::sync::mpsc::UnboundedReceiver as MpscUnboundedReceiver;

#[allow(async_fn_in_trait)]
pub trait UncheckedRecv<T> {
    async fn unchecked_recv(&mut self) -> Option<T>;
}

impl<T> UncheckedRecv<T> for MpscUnboundedReceiver<T> {
    async fn unchecked_recv(&mut self) -> Option<T> {
        self.recv().await
    }
}
