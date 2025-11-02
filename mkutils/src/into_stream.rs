use futures::Stream;
use tokio::{
    io::{AsyncBufRead, Lines},
    sync::{broadcast::Receiver as BroadcastReceiver, mpsc::UnboundedReceiver as MpscUnboundedReceiver},
    time::Interval,
};
use tokio_stream::wrappers::{
    BroadcastStream as BroadcastReceiverStream, IntervalStream, LinesStream,
    UnboundedReceiverStream as MpscUnboundedReceiverStream,
};

pub trait IntoStream {
    type Stream: Stream;

    fn into_stream(self) -> Self::Stream;
}

impl IntoStream for Interval {
    type Stream = IntervalStream;

    fn into_stream(self) -> Self::Stream {
        IntervalStream::new(self)
    }
}

impl<R: AsyncBufRead> IntoStream for Lines<R> {
    type Stream = LinesStream<R>;

    fn into_stream(self) -> Self::Stream {
        LinesStream::new(self)
    }
}

impl<T> IntoStream for MpscUnboundedReceiver<T> {
    type Stream = MpscUnboundedReceiverStream<T>;

    fn into_stream(self) -> Self::Stream {
        self.into()
    }
}

impl<T: 'static + Clone + Send> IntoStream for BroadcastReceiver<T> {
    type Stream = BroadcastReceiverStream<T>;

    fn into_stream(self) -> Self::Stream {
        self.into()
    }
}
