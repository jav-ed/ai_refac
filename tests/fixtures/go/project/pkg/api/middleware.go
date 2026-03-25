package api

import "github.com/example/myproject/pkg/utils"

func ValidateMiddleware(v float64) bool {
	return utils.IsValid(v)
}
