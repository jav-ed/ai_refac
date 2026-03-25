package validator

import "github.com/example/myproject/pkg/utils"

// Check calls utils.Validate which lives in pkg/utils/validate.go — NOT the moved file.
// After the move this import must stay as pkg/utils, not be rewritten to pkg/helpers.
func Check(v float64) bool {
	return utils.Validate(v)
}
