package shape

import "testing"

func TestArea(t *testing.T) {
	r := NewRect(2, 3)
	if r.Area() != 6 {
		t.Fatal("Area is wrong")
	}
	if double(1) != 2 {
		t.Fatal("double is wrong")
	}
}
