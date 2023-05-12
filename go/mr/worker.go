package mr

import (
	"context"
	"encoding/json"
	"fmt"
	"hash/fnv"
	"io/ioutil"
	"log"
	"net"
	"os"
	"sort"
	"time"

	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials/insecure"
)

// Map functions return a slice of KeyValue.
type KeyValue struct {
	Key   string
	Value string
}

type ByKey []KeyValue

func (a ByKey) Len() int           { return len(a) }
func (a ByKey) Swap(i, j int)      { a[i], a[j] = a[j], a[i] }
func (a ByKey) Less(i, j int) bool { return a[i].Key < a[j].Key }

// use ihash(key) % NReduce to choose the reduce
// task number for each KeyValue emitted by Map.
func ihash(key string) int {
	h := fnv.New32a()
	h.Write([]byte(key))
	return int(h.Sum32() & 0x7fffffff)
}

// Request new task from Coordinator
func callRequestTask(w MapReduceClient) *RequestReply {
	ctx, cancel := context.WithTimeout(context.Background(), time.Second)
	defer cancel()
	reply, err := w.RequestTask(ctx, &RequestArgs{})
	if err != nil {
		reply.Done = true
	}
	return reply
}

// Report Map/Reduce results to Coordinator
func callReportTask(w MapReduceClient, args *ReportArgs) {
	ctx, cancel := context.WithTimeout(context.Background(), time.Second)
	defer cancel()
	_, err := w.ReportTask(ctx, args)
	if err != nil {
		log.Println(err)
	}
}

// Handle Map tasks, report intermediate files to Coordinator
func handleMap(w MapReduceClient, filename string, taskno int, nreduce int, mapf func(string, string) []KeyValue) {
	f, err := os.Open(filename)
	if err != nil {
		log.Fatalf("cannot open %v\n", filename)
	}
	text, err := ioutil.ReadAll(f)
	if err != nil {
		log.Fatalf("cannot read %v\n", filename)
	}
	f.Close()

	kvs := mapf(filename, string(text))
	sort.Sort(ByKey(kvs))

	buckets := make([][]KeyValue, nreduce)
	for _, kv := range kvs {
		key := ihash(kv.Key) % nreduce
		buckets[key] = append(buckets[key], kv)
	}

	intermediates := make([]string, nreduce)
	for i := 0; i < nreduce; i++ {
		fname := fmt.Sprintf("mr-%v-%v", taskno, i)
		intermediates[i] = fname
		f, _ := os.Create(fname)

		enc := json.NewEncoder(f)
		for _, kv := range buckets[i] {
			err := enc.Encode(&kv)
			if err != nil {
				log.Printf("Marshal error: %v\n", err)
			}
		}
		f.Close()
	}

	callReportTask(w, &ReportArgs{Kind: TaskType_Map, Task: filename, Intermediates: intermediates})
}

// Read key-value pairs from intermediate files created by Map task
func readIntermediates(files []string) []KeyValue {
	kvs := []KeyValue{}
	for _, filename := range files {
		f, err := os.Open(filename)
		if err == nil {
			dec := json.NewDecoder(f)
			for {
				var kv KeyValue
				if err := dec.Decode(&kv); err != nil {
					break
				}
				kvs = append(kvs, kv)
			}
		} else {
			log.Println("Reducer failed to open file: ", err.Error())
		}
		f.Close()
	}
	return kvs
}

// Handle Reduce tasks and report to Coordinator
func handleReduce(w MapReduceClient, taskno int, files []string, reducef func(string, []string) string) {
	kvs := readIntermediates(files)
	sort.Sort(ByKey(kvs))

	fname := fmt.Sprintf("mr-out-%v", taskno)
	tmp, err := ioutil.TempFile(".", fname)
	if err != nil {
		log.Printf("Reducer failed to create tempfile: %v\n", tmp)
	}

	// Call Reduce on each distinct key and its associated values
	// with results into temp file
	for i := 0; i < len(kvs); i++ {
		key, vals := kvs[i].Key, []string{}
		for i < len(kvs) && kvs[i].Key == key {
			vals = append(vals, kvs[i].Value)
			i++
		}
		i--
		fmt.Fprintf(tmp, "%v %v\n", key, reducef(key, vals))
	}
	// Rename temp file to result when finished
	os.Rename(tmp.Name(), fname)

	callReportTask(w, &ReportArgs{Kind: TaskType_Reduce, Id: int32(taskno)})
}

// A Worker performs Map or Reduce tasks at the behest of the Coordinator until
// all tasks are completed
func Worker(mapf func(string, string) []KeyValue, reducef func(string, []string) string) {
	dialer := func(ctx context.Context, addr string) (net.Conn, error) {
		var d net.Dialer
		return d.DialContext(ctx, "unix", addr)
	}
	sockname := CoordinatorSock()
	conn, err := grpc.Dial(sockname,
		grpc.WithTransportCredentials(insecure.NewCredentials()),
		grpc.WithContextDialer(dialer),
	)
	if err != nil {
		log.Fatalf("grpc.Dial(%q) failed: %v\n", sockname, err)
	}
	defer conn.Close()

	worker := NewMapReduceClient(conn)
	for {
		reply := callRequestTask(worker)
		if reply.Done {
			break
		}
		switch reply.Kind {
		case TaskType_Map:
			log.Printf("received map task: %s\n", reply.Filename)
			handleMap(worker, reply.Filename, int(reply.TaskNo), int(reply.NReduce), mapf)
		case TaskType_Reduce:
			log.Printf("received reduce task: %v\n", reply.TaskNo)
			handleReduce(worker, int(reply.TaskNo), reply.Intermediates, reducef)
		default:
			time.Sleep(time.Second)
		}
	}
}
