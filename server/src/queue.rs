use futures::channel::oneshot;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex, Weak};
use tracing::{debug, info, warn};

#[derive(Clone, Copy, Debug)]
pub enum QueueBehavior {
    QueueMessages { max_size: Option<usize> },
    DropMessages,
}

pub struct MessageQueue<T> {
    inner: Arc<Mutex<Inner<T>>>,
}

struct Inner<T> {
    waiters: VecDeque<Arc<Waiter<T>>>,
    messages: VecDeque<T>,
    behavior: QueueBehavior,
}

struct Waiter<T> {
    sender: Mutex<Option<oneshot::Sender<T>>>,
}

pub struct Subscription<T> {
    receiver: Option<oneshot::Receiver<T>>,
    waiter: Option<Weak<Waiter<T>>>,
    inner: Arc<Mutex<Inner<T>>>,
}

impl<T> MessageQueue<T> {
    pub fn new(behavior: QueueBehavior) -> Self {
        info!("Creating a new MessageQueue with behavior: {:?}", behavior);
        MessageQueue {
            inner: Arc::new(Mutex::new(Inner {
                waiters: VecDeque::new(),
                messages: VecDeque::new(),
                behavior,
            })),
        }
    }

    pub fn subscribe(&self) -> Subscription<T> {
        let (sender, receiver) = oneshot::channel();
        let mut inner = self.inner.lock().unwrap();
        if let Some(msg) = inner.messages.pop_front() {
            debug!("Delivering queued message to new subscriber");
            let _ = sender.send(msg);
            Subscription {
                receiver: Some(receiver),
                waiter: None,
                inner: self.inner.clone(),
            }
        } else {
            debug!("Adding new subscriber to waiters queue");
            let waiter = Arc::new(Waiter {
                sender: Mutex::new(Some(sender)),
            });
            let waiter_weak = Arc::downgrade(&waiter);
            inner.waiters.push_back(waiter);
            Subscription {
                receiver: Some(receiver),
                waiter: Some(waiter_weak),
                inner: self.inner.clone(),
            }
        }
    }

    pub fn send(&self, mut msg: T) {
        let mut inner = self.inner.lock().unwrap();
        while let Some(waiter) = inner.waiters.pop_front() {
            let sender = {
                let mut sender_opt = waiter.sender.lock().unwrap();
                sender_opt.take()
            };
            if let Some(sender) = sender {
                match sender.send(msg) {
                    Ok(()) => {
                        debug!("Message sent to subscriber");
                        return;
                    }
                    Err(returned_msg) => {
                        warn!("Failed to send message to subscriber, retrying");
                        msg = returned_msg;
                    }
                }
            } else {
                warn!("Waiter sender already taken, skipping");
            }
        }
        match inner.behavior {
            QueueBehavior::QueueMessages { max_size } => {
                if let Some(max) = max_size {
                    if inner.messages.len() >= max {
                        warn!("Message queue full, dropping oldest message");
                        inner.messages.pop_front();
                    }
                }
                debug!("Queueing message as no subscribers are available");
                inner.messages.push_back(msg);
            }
            QueueBehavior::DropMessages => {
                info!("Dropping message as per configured behavior");
            }
        }
    }
}

impl<T> Subscription<T> {
    pub async fn receive(mut self) -> Option<T> {
        if let Some(receiver) = self.receiver.take() {
            match receiver.await {
                Ok(msg) => {
                    debug!("Subscriber received message");
                    Some(msg)
                }
                Err(_) => {
                    warn!("Subscriber failed to receive message");
                    None
                }
            }
        } else {
            warn!("Receiver already taken");
            None
        }
    }
}

impl<T> Drop for Subscription<T> {
    fn drop(&mut self) {
        if let Some(waiter_weak) = self.waiter.take() {
            if let Some(waiter) = waiter_weak.upgrade() {
                let _ = waiter.sender.lock().unwrap().take();
                let mut inner = self.inner.lock().unwrap();
                if let Some(pos) = inner.waiters.iter().position(|w| Arc::ptr_eq(w, &waiter)) {
                    debug!("Removing subscriber from waiters queue");
                    inner.waiters.remove(pos);
                }
            }
        }
    }
}
