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

pub trait ITask: Send + Sync {
     fn on_execute(&self);
     fn on_complete(&self);
}

#[derive(Clone)]
pub struct TaskPool {
    tasks: Arc<Mutex<VecDeque<Arc<dyn ITask + Send>>>>,
}

impl TaskPool {
    pub fn new() -> Self {
        TaskPool {
            tasks: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    pub async fn enqueue(&self, task: Arc<dyn ITask + Send>) {
        let mut tasks = self.tasks.lock().await;
        tasks.push_back(task);
    }

    pub async fn dequeue(&self) -> Option<Arc<dyn ITask + Send>> {
        let mut tasks = self.tasks.lock().await;
        tasks.pop_front()
    }

    pub async fn erase(&self, task: Arc<dyn ITask + Send>) {
        let mut tasks = self.tasks.lock().await;
        tasks.retain(|t| !Arc::ptr_eq(t, &task));
    }

    pub async fn clear(&self) {
        let mut tasks = self.tasks.lock().await;
        tasks.clear();
    }

    pub async fn is_empty(&self) -> bool {
        let tasks = self.tasks.lock().await;
        tasks.is_empty()
    }
}

pub struct AsyncThreadPool {
    task_pool: Arc<TaskPool>,
}

impl AsyncThreadPool {
    pub fn new() -> Self {
        let task_pool = Arc::new(TaskPool::new());
        AsyncThreadPool { task_pool }
    }

    pub async fn add_task(&self, task: Arc<dyn ITask + Send>) {
        self.task_pool.enqueue(task).await;
    }

    pub async fn cancel_task(&self, task: Arc<dyn ITask + Send>) {
        self.task_pool.erase(task).await;
    }

    pub async fn execute(&self) {
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

    pub async fn terminate(&self) {
        self.task_pool.clear().await;
    }
}
