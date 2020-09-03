use std::thread;
use std::sync::{mpsc, Arc, Mutex};

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Message>,
}

impl ThreadPool {
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);

        let (sender, receiver) = mpsc::channel();

        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);

        for i in 0..size {
            // create some workers and store em in the vector
            workers.push(Worker::new(i, Arc::clone(&receiver)));
        }

        ThreadPool{
            workers,
            sender,
        }
    }

    pub fn execute<F>(&self, f: F)
        where
            F: FnOnce() + Send + 'static
    {
        let job = Box::new(f);

        self.sender.send(Message::NewJob(job)).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        // Terminate all workers
        for _ in &mut self.workers {
            self.sender.send(Message::Terminate).unwrap();
        }

        for worker in &mut self.workers {
            println!("shutting down worker {}", worker.id);

            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap()
            }
        }
    }
}

trait FnBox {
    fn call_box(self: Box<Self>);
}

impl<F: FnOnce()> FnBox for F {
    fn call_box(self: Box<F>) {
        (*self)()
    }
}

enum Message {
    NewJob(Job),
    Terminate,
}

type Job = Box<dyn FnBox + Send + 'static>;

struct Worker {
    thread: Option<thread::JoinHandle<()>>,
    id: usize,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Message>>>) -> Worker {
        let thread = thread::spawn(move || {
            loop {
                let message = receiver.lock().unwrap().recv().unwrap();
                
                match message {
                    Message::NewJob(job) => {
                        println!("worker {} got a job, executing", id);
                        job.call_box();
                    },
                    Message::Terminate => {
                        println!("terminating worker {}", id);
                        break;
                    },
                }
            }
        });
        Worker {
            id,
            thread: Some(thread),
        }
    }
}