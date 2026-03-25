package reports

import "github.com/example/myproject/pkg/utils"

func Generate(v float64) string {
	return utils.FormatValue(v)
}
