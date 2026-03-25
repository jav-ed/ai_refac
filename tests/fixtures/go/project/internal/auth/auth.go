package auth

import "github.com/example/myproject/pkg/utils"

func Authenticate(token float64) bool {
	return utils.IsValid(token)
}
