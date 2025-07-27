use std::{
    fmt::{Debug, Display},
    future::Future,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
    time::Duration,
};
use tokio::{
    sync::{oneshot, Mutex},
    task::AbortHandle,
};

#[derive(Debug, Clone, PartialEq)]
pub enum RequestError {
    Timeout,
    DropWithoutResolve,
    // TODO: we should realistically panic, as this is an dev error!
    AlreadyClaimed,
    FailedToSend,
}

// impl Display for RequestError{
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         match self{
//             _ => todo!()
//         }
//
//     }
//
// }
//
//
// impl std::error::Error for RequestError {
//     fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
//         todo!()
//         // // Return the underlying error cause if any
//         // match self {
//         //     RequestError::NetworkError(_) => None, // Could return underlying network error
//         //     _ => None,
//         // }
//     }
// }

pub struct Request<T> {
    // Removed T: Clone bound
    sender: Arc<Mutex<Option<oneshot::Sender<Result<T, RequestError>>>>>,
}

impl<T> PartialEq for Request<T> {
    fn eq(&self, _other: &Self) -> bool {
        false
    }
}

// Fixed Debug implementation with generic parameter
impl<T> Debug for Request<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Request")
    }
}

pub struct Response<T> {
    // ugh ... needed for drop trait shenigans!
    rx: Option<oneshot::Receiver<Result<T, RequestError>>>,
    timeout_handle: AbortHandle,
}

// impl<T> Future for Response<T> {
//     type Output = Result<Result<T, RequestError>, oneshot::error::RecvError>;
//
//     fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
//         // Pin the inner receiver and poll it
//         let resp = Pin::new(&mut self.rx).poll(cx);
//
//         // Only abort timeout if we're actually ready
//         if matches!(resp, Poll::Ready(_)) {
//             self.timeout_handle.abort();
//         }
//
//         resp
//     }
// }

impl<T> Response<T> {
    pub async fn recv(mut self) -> Result<T, RequestError> {
        let rx = self.rx.take().unwrap();
        match rx.await {
            Ok(Ok(v)) => {
                self.timeout_handle.abort();
                Ok(v)
            }
            Ok(Err(e)) => Err(e),
            Err(_e) => Err(RequestError::DropWithoutResolve),
        }
    }
}

impl<T> Drop for Response<T> {
    fn drop(&mut self) {
        // Ensure timeout is cancelled when Response is dropped
        self.timeout_handle.abort();
    }
}

impl<T> Request<T>
where
    T: Send + 'static, // Required for tokio::spawn and channels
{
    pub fn new() -> (Response<T>, Self) {
        Self::with_timeout(Duration::from_secs(10))
    }

    pub fn with_timeout(duration: Duration) -> (Response<T>, Self) {
        let (tx, rx) = oneshot::channel::<Result<T, RequestError>>();
        let req = Self {
            sender: Arc::new(Mutex::new(Some(tx))),
        };

        let timeout_handle = tokio::spawn({
            let req = req.clone();
            async move {
                tokio::time::sleep(duration).await;
                // Use try_claim to avoid consuming self
                match req.sender.lock().await.take() {
                    None => return,
                    Some(sender) => {
                        let _ = sender.send(Err(RequestError::Timeout));
                    }
                }
            }
        });

        (
            Response {
                rx: Some(rx),
                timeout_handle: timeout_handle.abort_handle(),
            },
            req,
        )
    }

    // Fixed: Use &self instead of consuming self
    pub async fn fulfill(&self, value: T) -> () {
        let sender = self
            .sender
            .lock()
            .await
            .take()
            .expect("request was already claimed else where!");
        let _ = sender.send(Ok(value));
    }
}

// Manual Clone implementation to avoid T: Clone requirement
impl<T> Clone for Request<T> {
    fn clone(&self) -> Self {
        Self {
            sender: Arc::clone(&self.sender),
        }
    }
}

// Helper for creating requests without new() being async
impl<T> Request<T>
where
    T: Send + 'static,
{
    pub fn create() -> (Response<T>, Self) {
        Self::new()
    }

    pub fn create_with_timeout(duration: Duration) -> (Response<T>, Self) {
        Self::with_timeout(duration)
    }
}
