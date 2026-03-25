package api

import "github.com/example/myproject/pkg/utils"

func DefaultFormat() string {
	return utils.FormatValue(0)
}
