// Control file: no dependency on pkg/utils — must be byte-identical after move.
package config

type Config struct {
	Host string
	Port int
}

func Default() Config {
	return Config{Host: "localhost", Port: 8080}
}
