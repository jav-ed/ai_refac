package service

import (
	"github.com/example/myproject/pkg/utils"
)

// Exercises: unaliased import — qualifier changes from utils. to helpers. at call sites.
func Process(v float64) string {
	return utils.FormatValue(v)
}

func Check(v float64) bool {
	return utils.IsValid(v)
}
