use std::sync::{Arc, Mutex, Condvar};
use std::collections::VecDeque;

// Arc allows multiple parts of your program to share ownership of the same object. It keeps a reference count (internally, an atomic counter) of how many owners exist.
// When the reference count reaches zero (i.e., when all owners are dropped), it deallocates the shared memory.


// Arc allows both Sender and Receiver to hold references to the same Inner<T> without making multiple copies of it.
pub struct Sender<T> {
        inner : Arc<Inner<T>>,

}

// By wrapping the queue inside a Mutex, we ensure that only one part of the program can modify or read from the queue at a time, preventing any race conditions.
pub struct Receiver<T> {
        inner : Arc<Inner<T>>, // even though we have a single receiver, a send and receive might happen at same time, they need to be mutually exclusive to each other
}

impl<T> Sender<T> {
        pub fn send(&mut self, t: T) {
                let queue = self.inner.queue.lock().unwrap(); // lock return LockResult<MutexGuard<T>> as if the thread panics during lock, so data might not be in consitent state, to communicate this the thread sets a flag, the last thing that accessed this panic. Guard or PoisonError<Guard>
                queue.push_back(t);
                drop(queue); // drop the lock
                self.inner.available.notify_one(); // notify one thread to wake up i.e. receiver
        }
} 

impl<T> Receiver<T> {
        pub fn receive(&mut self) -> T {
                let mut queue = self.inner.queue.lock().unwrap();
                loop {
                        
                        // pop_front() returns Option<T>, so we need to provide a blocking version of receive where it waits if there isn't something in channel
                        // Here condvar comes into play
                        match queue.pop_front() {
                                Some(t) => return t,
                                None => {
                                        queue = self.inner.available.wait(queue).unwrap();
                                }
                        } // Since we use vec in Inner struct for queue, it acts like a stack and we pop the latest element that was inserted, instead we want the oldest element to be poped. We use ring buffer data structure in cases like this.
                }
}

struct Inner<T> {
        queue : Mutex<VecDeque<T>>,
        available : Condvar,
        // condvar is outside the mutex, as we if t1 thread holds mutex and we need to wake other threads up, the thread that wakes up has to take mutex
        // If we tell them to wake up while holding the mutex, they wake up and try to take the lock but they can't, they go to sleep, we will be deadlock
}

pub fn channel<T>() -> (Sender<T>, Receiver<T>){
        let inner = Inner{
                queue : Mutex::new(Vec::new()), // Mutex::default(), uses default implementation of vec
        };
        let sharedInner = Arc::new(inner);

        // sharedInner.clone() is cloning the Arc, not the underlying data.
        // The reference count is incremented when you call .clone() on arc
        // The clone() method on Arc gives each of them a reference (or a "borrower's card") to the same underlying data.

        (Sender { inner : sharedInner.clone()}, Receiver {inner : sharedInner.clone()},)
}