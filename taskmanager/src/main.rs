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
use std::thread;
use std::time::Duration;

use taskmanager::{ThreadPool, ITask};

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

fn main() {
    let mut pool = ThreadPool::new(4);

    println!("Adding tasks");
    for i in 0..5 {
        pool.add_task(Arc::new(MyTask::new(i)));
    }

    println!("Executing thread pool");
    pool.execute();

    println!("Waiting for tasks to complete");
    thread::sleep(Duration::from_secs(2));

    println!("Adding more tasks");
    for i in 5..7 {
        pool.add_task(Arc::new(MyTask::new(i)));
    }

    println!("Waiting for tasks to complete");
    thread::sleep(Duration::from_secs(3));

    println!("Terminating thread pool");
    pool.terminate();
}
