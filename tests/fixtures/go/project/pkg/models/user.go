package models

import "github.com/example/myproject/pkg/utils"

type User struct {
	Name  string
	Score float64
}

func (u User) FormattedScore() string {
	return utils.FormatValue(u.Score)
}

func (u User) IsScoreValid() bool {
	return utils.IsValid(u.Score)
}
