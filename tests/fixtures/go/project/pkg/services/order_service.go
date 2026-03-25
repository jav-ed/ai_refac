package services

import "github.com/example/myproject/pkg/utils"

func ProcessOrder(amount float64) string {
	return utils.FormatValue(amount)
}
