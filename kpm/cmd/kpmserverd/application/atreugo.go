package application

import (
	"github.com/fasthttp/session/v2"
	"github.com/savsgio/atreugo/v11"
	"os"
	"strconv"
)

var atreugoServer *atreugo.Atreugo

func GetAtreugo() *atreugo.Atreugo {
	if atreugoServer != nil {
		return atreugoServer
	}
	return createAtreugo()
}

func createAtreugo() *atreugo.Atreugo {
	atg := atreugo.Config{
		Logger:                GetLogger(),
		NoDefaultServerHeader: true,
		NoDefaultDate:         true,
		GracefulShutdown:      true,
		Addr:                  "0.0.0.0:9000",
		LogAllErrors:          false,
		MaxConnsPerIP:         0,
		Prefork:               false,
		ReduceMemoryUsage:     false,
		Compress:              false,
	}
	if v := os.Getenv("ATREUGO_ADDR"); v != "" {
		atg.Addr = v
	}
	if v := os.Getenv("ATREUGO_MAXCONNSPERIP"); v != "" {
		parseInt, err := strconv.ParseInt(v, 10, 64)
		if err == nil {
			atg.MaxConnsPerIP = int(parseInt)
		}
	}
	if v := os.Getenv("ATREUGO_PREFORK"); v == "true" {
		atg.Prefork = true
	}
	if v := os.Getenv("ATREUGO_REDUCEMEMORYUSAGE"); v == "true" {
		atg.ReduceMemoryUsage = true
	}
	if v := os.Getenv("ATREUGO_COMPRESS"); v == "true" {
		atg.Compress = true
	}
	if v := os.Getenv("ATREUGO_LOGALLERRORS"); v == "true" {
		atg.LogAllErrors = true
	}
	server := atreugo.New(atg)
	//pc := prometheus.Config{}
	//if v := os.Getenv("ATREUGO_PROMETHEUS_METHOD"); v != "" {
	//	pc.Method = v
	//}
	//if v := os.Getenv("ATREUGO_PROMETHEUS_URL"); v != "" {
	//	pc.URL = v
	//}
	//if v := os.Getenv("ATREUGO_PROMETHEUS"); v == "true" {
	//	prometheus.Register(server, pc)
	//}
	return server
}

// AutoLoadSaveSessionStore 自动加载保存会话存储
func AutoLoadSaveSessionStore(a *atreugo.Atreugo) {
	a.UseBefore(loadSessionStore).UseAfter(saveSessionStore)
}

// LoadSessionStore 加载会话存储
func loadSessionStore(ctx *atreugo.RequestCtx) error {
	store, err := GetAtreugoSession().Get(ctx)
	if err != nil {
		log.Err(err).Send()
		return nil
	}
	log.Debug().Msg("加载会话成功")
	ctx.SetUserValue("store", store)
	return ctx.Next()
}
func saveSessionStore(ctx *atreugo.RequestCtx) error {
	storeI := ctx.UserValue("store")
	if storeI == nil {
		return ctx.Next()
	}
	err := GetAtreugoSession().Save(ctx, storeI.(*session.Store))
	if err != nil {
		GetLogger().Err(err).Send()
		return nil
	}
	GetLogger().Debug().Msg("保存会话")
	return ctx.Next()
}
func SetJsonString(ctx *atreugo.RequestCtx, str string) error {
	ctx.Response.Header.SetContentType("application/json")
	ctx.Response.SetBodyString(str)
	return nil
}
