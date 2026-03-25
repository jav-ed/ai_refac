package services

import "github.com/example/myproject/pkg/utils"

func ProcessUser(score float64) string {
	if utils.IsValid(score) {
		return utils.FormatValue(score)
	}
	return "invalid"
}
