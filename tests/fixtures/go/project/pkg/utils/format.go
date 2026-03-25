// THIS FILE IS THE MOVE TARGET in tests/go_move.rs
// Move: pkg/utils/format.go -> pkg/helpers/format.go
// Package changes from `utils` to `helpers`.

package utils

import "fmt"

func FormatValue(v float64) string {
	return fmt.Sprintf("%.2f", v)
}

func IsValid(v float64) bool {
	return v > 0
}
