package main

import (
	"fmt"

	"github.com/example/myproject/pkg/utils"
)

func main() {
	fmt.Println(utils.FormatValue(99.9))
	fmt.Println(utils.IsValid(99.9))
}
