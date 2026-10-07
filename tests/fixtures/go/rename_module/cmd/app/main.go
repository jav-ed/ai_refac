package main

import (
	"fmt"

	"example.com/shop/internal/report"
	"example.com/shop/shape"
)

func main() {
	r := shape.NewRect(3, 4)
	fmt.Println(r.Area(), shape.Count)
	for _, line := range report.Lines([]shape.Shape{r}) {
		fmt.Println(line)
	}
}
