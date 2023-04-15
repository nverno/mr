package main

import (
	"fmt"

	"github.com/nverno/mrwc/go/mr"
)

// type Coordinator struct {
// 	ra RequestArgs
// }

func main() {
	tst := mr.RequestArgs{}
	fmt.Printf("%v\n", tst.String())
}
