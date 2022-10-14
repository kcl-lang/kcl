package main

import (
	"github.com/savsgio/atreugo/v11"
	_ "go.uber.org/automaxprocs"
	"kpm/cmd/kpmserverd/application"
	"kpm/cmd/kpmserverd/service"
	"net/url"
	"os"
	"os/user"
)

var log = application.GetLogger()

func main() {
	err := ServerSetup()
	if err != nil {
		return
	}
	server := application.GetAtreugo()
	//搜索
	// /api/v1/search?q=pkgv
	//发布
	// /api/v1/user/publish
	// 强制覆盖 git push --force origin main
	api := server.NewGroupPath("/api")
	v1 := api.NewGroupPath("/v1")
	v1.UseBefore(func(ctx *atreugo.RequestCtx) error {
		ctx.SetContentType("application/json")
		return ctx.Next()
	})
	//application.NewService(service.NewMock())
	application.NewService(service.NewDefault(application.GetSqlxClient()))
	appService := application.GetService()
	v1.GET("/search", func(ctx *atreugo.RequestCtx) error {
		pkgv := ctx.RequestCtx.QueryArgs().Peek("pkgname")
		if pkgv != nil {
			ctx.SetBodyString(appService.SearchName(string(pkgv)))
			return nil
		}
		subpkgname := ctx.RequestCtx.QueryArgs().Peek("subpkgname")
		if subpkgname != nil {
			ctx.SetBodyString(appService.SearchSubPkgName(string(subpkgname)))
			return nil
		}
		ctx.SetBodyString(StdArgsWrongResp)
		return nil
	})
	//v1.POST("/search", func(ctx *atreugo.RequestCtx) error {
	//	return nil
	//})
	u := v1.NewGroupPath("/u")
	u.POST("/publish", func(ctx *atreugo.RequestCtx) error {
		//准备好发布版本
		//接收数据，解压，解析，验证，更新版本，更新tag，
		body := ctx.Request.Body()
		if len(body) == 0 {
			ctx.SetBodyString(StdArgsWrongResp)
			return nil
		}
		compress := ctx.Request.Header.Peek("X-KPM-PKG-COMPRESS")
		ctx.SetBodyString(appService.Publish(body, string(compress), KPM_ROOT, KPM_SERVER_ADDR, KPM_SERVER_ADDR_PATH))
		return nil
	})

	// /s/store/:bk/:sha512
	// /s/pkg_tag/:pkgname/tags
	// /s/metadata/:pkgname/tags

	s := server.NewGroupPath("/s")
	metadatapath := KPM_ROOT + Separator + "registry" + Separator + KPM_SERVER_ADDR_PATH + Separator + "metadata"
	//包元数据
	s.StaticCustom("/metadata", &atreugo.StaticFS{
		AllowEmptyRoot:     false,
		Root:               metadatapath,
		GenerateIndexPages: true,
		AcceptByteRange:    false,
		Compress:           true,
		//CompressBrotli:     true,
	})
	//全局hash存储
	storepath := KPM_ROOT + Separator + "store"
	s.StaticCustom("/store", &atreugo.StaticFS{
		AllowEmptyRoot:     false,
		Root:               storepath,
		GenerateIndexPages: true,
		AcceptByteRange:    false,
		Compress:           true,
		//CompressBrotli:     true,
	})
	//包的标签
	tagpath := KPM_ROOT + Separator + "registry" + Separator + KPM_SERVER_ADDR_PATH + Separator + "tag"
	s.StaticCustom("/tag", &atreugo.StaticFS{
		AllowEmptyRoot:     false,
		Root:               tagpath,
		GenerateIndexPages: true,
		AcceptByteRange:    false,
		Compress:           true,
		//CompressBrotli:     true,
	})
	err = server.ListenAndServe()
	if err != nil {
		panic(err)
	}
}
func ServerSetup() error {
	//加载环境变量
	if tmp := os.Getenv("KPM_ROOT"); tmp == "" {
		home := ""
		u, err := user.Current()
		if err != nil {
			if tmphome := os.Getenv("HOME"); tmphome != "" {
				home = tmphome
			} else {
				return nil
			}
		}
		home = u.HomeDir
		KPM_ROOT = home + Separator + "kpm"
	}
	if tmp := os.Getenv("KPM_SERVER_ADDR"); tmp != "" {
		KPM_SERVER_ADDR = tmp
	}
	parse, err := url.Parse(KPM_SERVER_ADDR)
	if err != nil {
		return err
	}
	KPM_SERVER_ADDR_PATH = parse.Host

	//初始化目录信息
	err = KeepDirExists(KPM_ROOT,
		KPM_ROOT+Separator+"registry",
		KPM_ROOT+Separator+"registry"+Separator+KPM_SERVER_ADDR_PATH,
		KPM_ROOT+Separator+"registry"+Separator+KPM_SERVER_ADDR_PATH+Separator+"tag",
		KPM_ROOT+Separator+"registry"+Separator+KPM_SERVER_ADDR_PATH+Separator+"metadata",
		KPM_ROOT+Separator+"store",
		KPM_ROOT+Separator+"store"+Separator+"v1",
		KPM_ROOT+Separator+"store"+Separator+"v1"+Separator+"files",
	)
	if err != nil {
		println("初始化目录失败")
		return err
	}
	for i := 0; i < len(hextable); i++ {
		for j := 0; j < len(hextable); j++ {
			err = KeepDirExists(KPM_ROOT + Separator + "store" + Separator + "v1" + Separator + "files" +
				Separator + string(hextable[i]) + string(hextable[j]))
			if err != nil {
				return err
			}
		}
	}
	return nil
}
