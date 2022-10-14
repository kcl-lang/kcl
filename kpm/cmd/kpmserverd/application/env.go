package application

import "os"

//local	本地开发环境
//dev/daily/development	日常开发环境
//pre/prepub	预生产环境
//prod/production	生产环境
//test/unittest	单元测试环境
//benchmark	性能测试环境

const RunEnvField = "RUNTIME_ENV"
const (
	// Local 本地生产环境
	Local = iota
	// Development dev/daily/development	日常开发环境
	Development
	// Prepub pre/prepub	预生产环境
	Prepub
	// Production prod/production	生产环境
	Production
	// UnitTest test/unittest	单元测试环境
	UnitTest
	// Benchmark benchmark	性能测试环境
	Benchmark
)

func GetRunEnv() int {
	switch os.Getenv(RunEnvField) {
	case "Local":
		return Local
	case "Development":
		return Development
	case "Prepub":
		return Prepub
	case "Production":
		return Production
	case "UnitTest":
		return UnitTest
	case "Benchmark":
		return Benchmark
	default:
		return Local
	}

}
