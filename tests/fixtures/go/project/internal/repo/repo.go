package repo

import "github.com/example/myproject/pkg/utils"

func FetchFormatted(v float64) string {
	return utils.FormatValue(v)
}
