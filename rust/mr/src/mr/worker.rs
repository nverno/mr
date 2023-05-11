
// map functions return a vector of KeyValue
pub struct KeyValue {
    pub key: String,
    pub value: String,
}

pub struct Worker {}

impl Worker {
    // Request a new task from the coordinator
    fn call_request_task(/*client*/) -> (/*reply*/) {
        todo!();
    }

    // Report map/reduce results to coordinator
    fn call_report_task() {
        todo!();
    }

    // Handle map task. Report intermediate files to coordinator
    fn handle_map() {
        todo!();
    }

    // Read key-value pairs from intermediate files created by a Map task
    fn read_intermediates() {
        todo!();
    }

    // Handle reduce task.
    fn handle_reduce() {
        todo!();
    }
}
