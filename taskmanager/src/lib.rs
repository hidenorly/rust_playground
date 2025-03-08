/*
  Copyright (C) 2025 hidenorly

   Licensed under the Apache License, Version 2.0 (the "License");
   you may not use this file except in compliance with the License.
   You may obtain a copy of the License at

       http://www.apache.org/licenses/LICENSE-2.0

   Unless required by applicable law or agreed to in writing, software
   distributed under the License is distributed on an "AS IS" BASIS,
   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
   See the License for the specific language governing permissions and
   limitations under the License.
*/

use std::sync::{Arc, Mutex};
use std::thread;
use std::collections::VecDeque;
use std::option::Option;

pub trait ITask: Send + Sync {
    fn on_execute(&self);
    fn on_complete(&self);
}

#[derive(Clone)]
struct TaskPool {
    tasks: Arc<Mutex<VecDeque<Arc<dyn ITask + Send>>>>,
}

impl TaskPool {
    fn new() -> Self {
        TaskPool {
            tasks: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    fn enqueue(&self, task: Arc<dyn ITask + Send>) {
        let mut tasks = self.tasks.lock().unwrap();
        tasks.push_back(task);
    }

    fn dequeue(&self) -> Option<Arc<dyn ITask + Send>> {
        let mut tasks = self.tasks.lock().unwrap();
        tasks.pop_front()
    }

    fn erase(&self, task: Arc<dyn ITask + Send>) {
        let mut tasks = self.tasks.lock().unwrap();
        tasks.retain(|t| !Arc::ptr_eq(t, &task));
    }

    fn clear(&self) {
        let mut tasks = self.tasks.lock().unwrap();
        tasks.clear();
    }

    fn is_empty(&self) -> bool {
        let tasks = self.tasks.lock().unwrap();
        tasks.is_empty()
    }
}

struct ThreadExecutor {
    task_pool: Arc<TaskPool>,
    thread: Option<thread::JoinHandle<()>>,
    stopping: bool,
    current_running_task: Option<Arc<dyn ITask + Send>>,
}

impl ThreadExecutor {
    fn new(task_pool: Arc<TaskPool>) -> Self {
        ThreadExecutor {
            task_pool,
            thread: None,
            stopping: false,
            current_running_task: None,
        }
    }

    fn execute(&mut self) {
        if self.thread.is_none() {
            let task_pool = self.task_pool.clone();
            self.thread = Some(thread::spawn(move || {
                Self::_execute(task_pool);
            }));
        }
    }

    fn terminate(&mut self) {
        if let Some(thread) = self.thread.take() {
            self.stopping = true;
            if let Some(_task) = &self.current_running_task {
                // Here we would cancel the task if it supports cancellation
                // This can be added later if needed
            }
            thread.join().unwrap();
            self.stopping = false;
        }
    }

    fn _execute(task_pool: Arc<TaskPool>) {
        while !task_pool.is_empty() {
            if let Some(task) = task_pool.dequeue() {
                task.on_execute();
                task.on_complete();
            } else {
                thread::yield_now();
            }
        }
    }

    fn cancel_task_if_running(&mut self, task: Arc<dyn ITask + Send>) {
        if let Some(current_task) = &self.current_running_task {
            if Arc::ptr_eq(current_task, &task) {
                // Cancel logic for task here
            }
        }
    }
}

pub struct ThreadPool {
    max_threads: usize,
    task_pool: Arc<TaskPool>,
    threads: Vec<ThreadExecutor>,
}

impl ThreadPool {
    pub fn new(num_threads: usize) -> Self {
        let task_pool = Arc::new(TaskPool::new());
        let mut threads = Vec::new();
        for _ in 0..num_threads {
            threads.push(ThreadExecutor::new(task_pool.clone()));
        }
        ThreadPool {
            max_threads: num_threads,
            task_pool,
            threads,
        }
    }

    pub fn add_task(&self, task: Arc<dyn ITask + Send>) {
        self.task_pool.enqueue(task);
    }

    pub fn cancel_task(&mut self, task: Arc<dyn ITask + Send>) {
        self.task_pool.erase(task.clone());

        for thread in &mut self.threads {
            thread.cancel_task_if_running(task.clone());
        }
    }

    pub fn execute(&mut self) {
        for thread in &mut self.threads {
            thread.execute();
        }
    }

    pub fn terminate(&mut self) {
        for thread in &mut self.threads {
            thread.terminate();
        }
    }
}
