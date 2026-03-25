package models

import "github.com/example/myproject/pkg/utils"

type Order struct {
	Amount float64
}

func (o Order) FormattedAmount() string {
	return utils.FormatValue(o.Amount)
}
