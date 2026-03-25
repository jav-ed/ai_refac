package main

import (
	"fmt"

	// blank import — exercises side-effect import rewrite (setup pkg does NOT move)
	_ "github.com/example/myproject/pkg/setup"

	// aliased import — alias must survive the package rename
	u "github.com/example/myproject/pkg/utils"
)

func main() {
	fmt.Println(u.FormatValue(42.5))
}
