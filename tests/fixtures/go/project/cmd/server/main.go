package main

import (
	"fmt"

	// aliased import with a different alias — alias must survive the package rename
	f "github.com/example/myproject/pkg/utils"
)

func main() {
	fmt.Println(f.FormatValue(3.14))
}
