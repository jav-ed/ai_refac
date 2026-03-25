package models

import "github.com/example/myproject/pkg/utils"

type Item struct {
	Price float64
}

func (i Item) FormattedPrice() string {
	return utils.FormatValue(i.Price)
}

func (i Item) IsPriceValid() bool {
	return utils.IsValid(i.Price)
}
