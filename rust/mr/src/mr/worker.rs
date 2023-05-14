use std::env::temp_dir;
use std::error::Error;
use std::fs::{self, File};
use std::io::{BufReader, Read, Write};

use crate::mapreduce::ReportArgs;
use crate::mapreduce::{map_reduce_client::MapReduceClient, RequestArgs, TaskType};
use mr_types::{KeyValue, MapFunc, ReduceFunc};
use tonic::{transport::Channel, Request};

fn ihash(key: &str) -> usize {
    key.as_bytes().iter().map(|&b| b as usize).sum()
}

// Handle map task. Return data to report to coordinator
fn handle_map(
    filename: String,
    task_no: usize,
    n_reduce: usize,
    mapf: MapFunc,
) -> Result<ReportArgs, Box<dyn Error>> {
    let mut f = File::open(&filename)?;
    let mut contents = String::new();
    f.read_to_string(&mut contents)?;

    unsafe {
        let mut kvs = mapf(filename.clone(), contents);
        kvs.sort_unstable();
        // println!("key-values: {:?}", kvs);

        let mut buckets: Vec<Vec<KeyValue>> = vec![vec![]; n_reduce];
        for kv in kvs {
            let k = ihash(&kv.key) % n_reduce;
            buckets[k].push(kv);
        }

        let mut intermediates = vec![];
        for i in 0..n_reduce {
            let fname = format!("mr-{}-{}", task_no, i);
            intermediates.push(fname.clone());

            let mut fp = File::create(&fname)?;

            let data = serde_json::to_string(&buckets[i])?;
            fp.write_all(data.as_bytes())?;
        }

        Ok(ReportArgs {
            kind: TaskType::Map as i32,
            task: filename,
            intermediates,
            ..Default::default()
        })
    }
}

// Read key-value pairs from intermediate files created by a Map task
fn read_intermediates(files: &Vec<String>) -> Result<Vec<KeyValue>, Box<dyn Error>> {
    let mut res = vec![];
    for fname in files {
        let file = File::open(fname)?;
        let reader = BufReader::new(file);
        let mut data: Vec<KeyValue> = serde_json::from_reader(reader)?;
        res.append(&mut data);
    }
    Ok(res)
}

// Handle reduce task.
fn handle_reduce(
    task_no: usize,
    files: &Vec<String>,
    reducef: ReduceFunc, //&dyn Fn(String, Vec<String>) -> String,
) -> Result<ReportArgs, Box<dyn Error>> {
    let mut kvs = read_intermediates(files)?;
    kvs.sort_unstable();

    let fname = format!("mr-out-{}", task_no);
    let tmpname = temp_dir().join(&fname);
    let mut tmp = File::create(&tmpname)?;

    // Call reduce on each distinct key and its associated values
    let mut i = 0;
    while i < kvs.len() {
        let (key, mut vals) = (kvs[i].key.clone(), vec![]);
        while i < kvs.len() && kvs[i].key == key {
            vals.push(kvs[i].value.clone());
            i += 1;
        }
        unsafe {
            tmp.write_all(format!("{} {}\n", key, reducef(key.clone(), vals)).as_bytes())?;
        }
    }

    fs::rename(tmpname, fname)?;

    Ok(ReportArgs {
        kind: TaskType::Reduce as i32,
        id: task_no as i32,
        ..Default::default()
    })
}

pub async fn work(
    w: &mut MapReduceClient<Channel>,
    mapf: MapFunc,
    reducef: ReduceFunc,
) -> Result<(), Box<dyn std::error::Error>> {
    loop {
        if false {
            break;
        }
        if let Ok(reply) = w.request_task(Request::new(RequestArgs::default())).await {
            let reply = reply.get_ref();
            if reply.done {
                break;
            }

            match reply.kind.try_into() {
                Ok(TaskType::Map) => {
                    println!("received map task: {}", reply.filename);
                    let args = handle_map(
                        reply.filename.clone(),
                        reply.task_no as usize,
                        reply.n_reduce as usize,
                        mapf,
                    )?;
                    w.report_task(Request::new(args)).await?;
                }
                Ok(TaskType::Reduce) => {
                    println!("received reduce task: {}", reply.task_no);
                    let args =
                        handle_reduce(reply.task_no as usize, &reply.intermediates, reducef)?;
                    w.report_task(Request::new(args)).await?;
                }
                _ => std::thread::sleep(std::time::Duration::from_secs(1)),
            }
        } else {
            break;
        }
    }

    Ok(())
}
