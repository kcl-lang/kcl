package application

import (
	"github.com/rs/zerolog"
	"os"
)

var log *zerolog.Logger

func GetLogger() *zerolog.Logger {
	if log != nil {
		return log
	}
	return createLogger()
}
func createLogger() *zerolog.Logger {
	if GetRunEnv() == Production {
		zerolog.SetGlobalLevel(zerolog.InfoLevel)
	}
	logs := zerolog.New(os.Stdout).With().Timestamp().Caller().Logger()
	return &logs
}

//func Panic(msg string) {
//	GetLogger().Error().Msg(msg)
//	os.Exit(1)
//}
