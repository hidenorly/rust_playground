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

use std::sync::Arc;
use tokio::sync::Mutex;
use std::collections::VecDeque;

trait ITask: Send + Sync {
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

    async fn enqueue(&self, task: Arc<dyn ITask + Send>) {
        let mut tasks = self.tasks.lock().await;
        tasks.push_back(task);
    }

    async fn dequeue(&self) -> Option<Arc<dyn ITask + Send>> {
        let mut tasks = self.tasks.lock().await;
        tasks.pop_front()
    }

    async fn erase(&self, task: Arc<dyn ITask + Send>) {
        let mut tasks = self.tasks.lock().await;
        tasks.retain(|t| !Arc::ptr_eq(t, &task));
    }

    async fn clear(&self) {
        let mut tasks = self.tasks.lock().await;
        tasks.clear();
    }

    async fn is_empty(&self) -> bool {
        let tasks = self.tasks.lock().await;
        tasks.is_empty()
    }
}

struct AsyncThreadPool {
    task_pool: Arc<TaskPool>,
}

impl AsyncThreadPool {
    fn new() -> Self {
        let task_pool = Arc::new(TaskPool::new());
        AsyncThreadPool { task_pool }
    }

    async fn add_task(&self, task: Arc<dyn ITask + Send>) {
        self.task_pool.enqueue(task).await;
    }

    async fn cancel_task(&self, task: Arc<dyn ITask + Send>) {
        self.task_pool.erase(task).await;
    }

    async fn execute(&self) {
        while !self.task_pool.is_empty().await {
            if let Some(task) = self.task_pool.dequeue().await {
                tokio::spawn(async move {
                    task.on_execute();
                    task.on_complete();
                });
            } else {
                tokio::task::yield_now().await;
            }
        }
    }

    async fn terminate(&self) {
        self.task_pool.clear().await;
    }
}

struct MyTask {
    id: i32,
}

impl MyTask {
    fn new(id: i32) -> Self {
        MyTask { id }
    }
}

impl ITask for MyTask {
    fn on_execute(&self) {
        println!("Executing task with id: {}", self.id);
    }

    fn on_complete(&self) {
        println!("Completed task with id: {}", self.id);
    }
}

#[tokio::main]
async fn main() {
    let pool = AsyncThreadPool::new();

    println!("Adding tasks");
    for i in 0..5 {
        pool.add_task(Arc::new(MyTask::new(i))).await;
    }

    println!("Executing thread pool");
    pool.execute().await;

    println!("Waiting for tasks to complete");
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    println!("Adding more tasks");
    for i in 5..7 {
        pool.add_task(Arc::new(MyTask::new(i))).await;
    }

    println!("Waiting for tasks to complete");
    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

    println!("Terminating thread pool");
    pool.terminate().await;
}
