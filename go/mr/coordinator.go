package mr

import (
	"context"
	"log"
	"net"
	"os"
	"strconv"
	sync "sync"
	"time"

	"google.golang.org/grpc"
)

type Coordinator struct {
	UnimplementedWcServer
	mu            sync.Mutex
	mapTasks      map[string]int64
	reduceTasks   map[int]int64
	nWorker       int
	nReduce       int
	timeout       int64
	intermediates map[int][]string
}

func (c *Coordinator) Done() bool {
	c.mu.Lock()
	defer c.mu.Unlock()

	return len(c.mapTasks) == 0 && len(c.reduceTasks) == 0
}

// Cook up a unique-ish UNIX-domain socket name in /var/tmp, for the coordinator.
func CoordinatorSock() string {
	s := "/var/tmp/824-mr-"
	s += strconv.Itoa(os.Getuid())
	return s
}

func MakeCoordinator(files []string, nReduce int) *Coordinator {
	c := Coordinator{nReduce: nReduce}

	c.mapTasks = make(map[string]int64)
	c.intermediates = make(map[int][]string)
	c.reduceTasks = make(map[int]int64)
	c.timeout = 10

	for _, file := range files {
		c.mapTasks[file] = 0
	}
	for i := 0; i < nReduce; i++ {
		c.reduceTasks[i] = 0
	}

	go c.server()
	return &c
}

func (c *Coordinator) server() {
	sockname := CoordinatorSock()
	os.Remove(sockname)
	lis, err := net.Listen("unix", sockname)
	if err != nil {
		log.Fatal("listen error:", err)
	}
	s := grpc.NewServer()
	RegisterWcServer(s, c)
	log.Printf("server listening at %v", lis.Addr())
	if err := s.Serve(lis); err != nil {
		log.Fatalf("failed to server: %v", err)
	}
}

// find pending map task
// if found, sets its start time to time.Now
func (c *Coordinator) NextMapTask() (string, bool) {
	now := time.Now().Unix()
	for task, start := range c.mapTasks {
		if now-start > c.timeout {
			c.mapTasks[task] = now
			return task, true
		}
	}

	return "", false
}

func (c *Coordinator) NextReduceTask() (int, bool) {
	now := time.Now().Unix()
	for task, start := range c.reduceTasks {
		if now-start > c.timeout {
			c.reduceTasks[task] = now
			return task, true
		}
	}

	return -1, false
}

func (c *Coordinator) RequestTask(ctx context.Context, args *RequestArgs) (*RequestReply, error) {
	c.mu.Lock()
	defer c.mu.Unlock()

	reply := &RequestReply{
		Kind:    TaskType_None,
		NReduce: int32(c.nReduce),
		Done:    false,
	}

	task, ok := c.NextMapTask()
	if ok {
		c.nWorker++
		reply.Kind = TaskType_Map
		reply.Filename = task
		reply.TaskNo = int32(c.nWorker)
		return reply, nil
	}
	if len(c.mapTasks) == 0 {
		rtask, rok := c.NextReduceTask()
		if rok {
			reply.Kind = TaskType_Reduce
			reply.TaskNo = int32(rtask)
			reply.Intermediates = c.intermediates[rtask]
		}
	}

	reply.Done = len(c.mapTasks) == 0 && len(c.reduceTasks) == 0
	return reply, nil
}

func (c *Coordinator) ReportTask(ctx context.Context, args *ReportArgs) (*ReportReply, error) {
	c.mu.Lock()
	defer c.mu.Unlock()

	if args.Kind == TaskType_Map {
		delete(c.mapTasks, args.Task)
		log.Printf("Coordinator: removing map task %v, %v remain", args.Task, len(c.mapTasks))
		for i, f := range args.Intermediates {
			c.intermediates[i] = append(c.intermediates[i], f)
		}
	} else {
		delete(c.reduceTasks, int(args.Id))
		log.Printf("Coordinator: removing reduce task %v, %v remain", args.Id, len(c.reduceTasks))
	}
	return &ReportReply{}, nil
}
