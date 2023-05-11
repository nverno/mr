// TODO: create coordinator struct
//

pub struct Coordinator {
    
}

impl Coordinator {
    fn done() -> bool {
        // true if no map tasks or reduce tasks remain
        todo!();
    }

    // create new gRPC server
    fn server() {}

    // Find pending map task
    // when found, set its start time
    fn next_map_task() -> (String, bool) {
        todo!();
    }

    // Find pending reduce task and set its start time
    fn next_reduce_task() -> (i32, bool) {
        todo!();
    }

    // Called by workers to request a new task
    fn request_task(/*ctx, args*/) -> (/*reply, error*/) {
        todo!();
    }

    // Called by workers to report task results
    fn report_task(/*ctx, args*/) -> (/*reply, error*/) {
        todo!();
    }
}

