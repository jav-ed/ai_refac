package api

import "github.com/example/myproject/pkg/utils"

func HandleValue(v float64) string {
	if utils.IsValid(v) {
		return utils.FormatValue(v)
	}
	return "bad"
}
