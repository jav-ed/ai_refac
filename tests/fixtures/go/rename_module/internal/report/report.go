// Package report prints shapes.
package report

import (
	"fmt"

	"example.com/shop/shape"
)

// Lines describes every shape, one per line.
func Lines(shapes []shape.Shape) []string {
	var lines []string
	for _, s := range shapes {
		lines = append(lines, fmt.Sprintf("%s (%.1f)", shape.Describe(s), s.Area()))
	}
	return lines
}
