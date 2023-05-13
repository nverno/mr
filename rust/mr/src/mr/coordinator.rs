use std::sync::{Mutex, Arc};
use std::{collections::HashMap, time::SystemTime};
use tonic::{Request, Response, Status};

use crate::mapreduce::map_reduce_server::MapReduce;
use crate::mapreduce::{ReportArgs, ReportReply, RequestArgs, RequestReply, TaskType};

#[derive(Debug)]
pub struct Coordinator {
    map_tasks: HashMap<String, SystemTime>,
    reduce_tasks: HashMap<usize, SystemTime>,
    n_worker: usize,
    n_reduce: usize,
    timeout: u64,
    intermediates: HashMap<usize, Vec<String>>,
}

impl Coordinator {
    pub fn new(timeout: u64, n_reduce: usize, files: Vec<String>) -> Self {
        let mut c  = Self {
            map_tasks: HashMap::new(),
            reduce_tasks: HashMap::new(),
            intermediates: HashMap::new(),
            n_worker: 0,
            n_reduce,
            timeout,
        };
        for f in files {
            c.map_tasks.insert(f, SystemTime::UNIX_EPOCH);
        }
        for i in 0..n_reduce {
            c.reduce_tasks.insert(i, SystemTime::UNIX_EPOCH);
        }
        c
    }

    // true when no map tasks or reduce tasks remain
    pub fn done(&self) -> bool {
        self.map_tasks.is_empty() && self.reduce_tasks.is_empty()
    }

    // Find pending map task. If found, set its start time
    pub fn next_map_task(&mut self) -> Option<String> {
        let now = SystemTime::now();
        for (task, start) in &mut self.map_tasks {
            if now.duration_since(*start).unwrap().as_secs() > self.timeout {
                *start = now;
                return Some(String::from(task));
            }
        }
        None
    }

    // Find pending reduce task and set its start time
    pub fn next_reduce_task(&mut self) -> Option<usize> {
        let now = SystemTime::now();
        for (task, start) in &mut self.reduce_tasks {
            if now.duration_since(*start).unwrap().as_secs() > self.timeout {
                *start = now;
                return Some(*task);
            }
        }
        None
    }
}

#[derive(Debug)]
pub struct CoordinatorService {
    pub coordinator: Arc<Mutex<Coordinator>>,
}

#[tonic::async_trait]
impl MapReduce for CoordinatorService {
    // Called by workers to request a new task
    async fn request_task(
        &self,
        _args: Request<RequestArgs>,
    ) -> Result<Response<RequestReply>, Status> {
        let mut c = self.coordinator.lock().unwrap();
        let mut reply = RequestReply {
            n_reduce: c.n_reduce as i32,
            ..Default::default()
        };

        if let Some(task) = c.next_map_task() {
            c.n_worker += 1;
            reply.kind = TaskType::Map as i32;
            reply.filename = task;
            reply.task_no = c.n_worker as i32;
            return Ok(Response::new(reply));
        }

        if c.map_tasks.is_empty() {
            if let Some(task) = c.next_reduce_task() {
                reply.kind = TaskType::Reduce as i32;
                reply.task_no = task as i32;
                reply.intermediates = c.intermediates[&task].clone();
            }
        }

        reply.done = c.done();
        Ok(Response::new(reply))
    }

    // Called by workers to report task results
    async fn report_task(
        &self,
        args: Request<ReportArgs>,
    ) -> Result<Response<ReportReply>, Status> {
        let mut c = self.coordinator.lock().unwrap();

        let args = args.get_ref();
        if args.kind == TaskType::Map as i32 {
            c.map_tasks.remove(&args.task);
            println!(
                "Coordinator: finished map task {}: {} to go",
                args.task,
                c.map_tasks.len()
            );
            for (i, f) in args.intermediates.iter().enumerate() {
                c.intermediates
                    .entry(i)
                    .or_insert(Vec::new())
                    .push(String::from(f));
            }
        } else {
            c.reduce_tasks.remove(&(args.id as usize));
            println!(
                "Coordinator: finished reduce task {}: {} to go",
                args.task,
                c.reduce_tasks.len()
            );
        }

        Ok(Response::new(ReportReply::default()))
    }
}

impl CoordinatorService {
    pub fn new(timeout: u64, n_reduce: usize, files: Vec<String>) -> Self {
        Self {
            coordinator: Arc::new(Mutex::new(Coordinator::new(timeout, n_reduce, files))),
        }
    }
}
