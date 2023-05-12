pub mod worker {
    tonic::include_proto!("mapreduce");
}

use mapreduce::map_reduce_client::MapReduceClient;

use super::coordinator::mapreduce;

// map functions return a vector of KeyValue
pub struct KeyValue {
    pub key: String,
    pub value: String,
}

// Request a new task from the coordinator
fn call_request_task(/*client*/) -> (/*reply*/) {
    unimplemented!();
}

// Report map/reduce results to coordinator
fn call_report_task() {
    unimplemented!();
}

// Handle map task. Report intermediate files to coordinator
fn handle_map() {
    unimplemented!();
}

// Read key-value pairs from intermediate files created by a Map task
fn read_intermediates() {
    unimplemented!();
}

// Handle reduce task.
fn handle_reduce() {
    unimplemented!();
}
