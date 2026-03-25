// validate.go stays in pkg/utils/ after the move — only format.go moves.
// Package declaration must remain `package utils` and this file must be unchanged.
package utils

func Validate(v float64) bool {
	return v >= 0 && v <= 1000
}
