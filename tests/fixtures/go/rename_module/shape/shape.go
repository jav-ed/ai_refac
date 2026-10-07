// Package shape holds shapes that can report their area.
package shape

import "fmt"

// Shape can report its Area.
type Shape interface {
	Area() float64
}

// Rect is a rectangle; W is its width.
type Rect struct {
	W, H float64
}

// Area is the surface of the Rect.
func (r Rect) Area() float64 { return r.W * r.H }

// NewRect builds a Rect.
func NewRect(w, h float64) Rect { return Rect{W: w, H: h} }

// Count is how many shapes were described.
var Count int

// Describe prints a shape. The word Area stays in this string and comment.
func Describe(s Shape) string {
	Count++
	total := s.Area()
	return fmt.Sprintf("Area=%v total=%v", s.Area(), total)
}

// scale multiplies a number; both parameters are called n in their own scope.
func scale(n float64, factor float64) float64 { return n * factor }

func double(n float64) float64 { return scale(n, 2) }
