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
use taskmanager_async::{AsyncThreadPool, ITask};

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
