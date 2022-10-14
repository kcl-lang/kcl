package service

import (
	"archive/tar"
	"bytes"
	"crypto/sha512"
	"encoding/json"
	"github.com/jmoiron/sqlx"
	"github.com/valyala/bytebufferpool"
	"github.com/valyala/fasthttp"
	"io"
	"kpm/cmd/kpmserverd/application"
	"kpm/cmd/kpmserverd/dao/mysql"
	"kpm/cmd/kpmserverd/response"
	"os"
	"strings"
)

type DataSourceDefault struct {
	mysql mysql.Mysql
}

func (d DataSourceDefault) SearchSubPkgName(SubPkgName string) string {
	//TODO implement me
	panic("implement me")
}

func (d DataSourceDefault) Publish(pkgtgz []byte, compress string, kpmroot string, kpmserver string, kpmserverpath string) string {
	b := bytebufferpool.Get()
	defer bytebufferpool.Put(b)
	b2 := bytebufferpool.Get()
	defer bytebufferpool.Put(b2)
	switch compress {
	case "gz":
		_, err := fasthttp.WriteGunzip(b, pkgtgz)
		if err != nil {
			return ""
		}
	case "br":
		_, err := fasthttp.WriteUnbrotli(b, pkgtgz)
		if err != nil {
			return ""
		}
	default:
		_, err := b.Write(pkgtgz)
		if err != nil {
			return ""
		}
	}
	tr := tar.NewReader(bytes.NewReader(b.B))
	pkginfo := PkgInfo{}
	for {
		h, err := tr.Next()
		if err == io.EOF {
			break
		}
		if err != nil {
			return response.StdErrResp
		}
		b2.Reset()
		_, err = io.Copy(b2, tr)
		if h.Name == "pkginfo.json" {
			err = json.Unmarshal(b2.B, &pkginfo)
			if err != nil {
				return response.StdErrResp
			}
			break
		}
	}
	tr = tar.NewReader(bytes.NewReader(b.B))
	for {
		h, err := tr.Next()
		if err == io.EOF {
			break
		}
		if err != nil {
			return response.StdErrResp
		}
		// 显示文件
		log.Info().Msg(h.Name)
		// 打开文件
		b2.Reset()
		_, err = io.Copy(b2, tr)
		if err != nil {
			return response.StdErrResp
		}
		if strings.HasPrefix(h.Name, "files/") {
			hash := application.EncodeToString(sha512.Sum512(b2.B))
			if h.Name != "files/"+hash {
				//数据出错
				log.Error().Msg(h.Name + " check error occurred")
				return response.StdErrResp
			}
			path := kpmroot + Separator + "store" + Separator + "v1" + Separator + "files" + Separator + application.HashMod(b2.B) + Separator + hash

			err = os.WriteFile(path, b2.B, 0777)
			if err != nil {
				return ""
			}
		}
	}
	//TODO implement me
	panic("implement me")
}

func (d DataSourceDefault) SearchName(name string) string {
	//TODO implement me
	panic("implement me")
}

func (d DataSourceDefault) SearchSubPkgNames(SubPkgNames []string) string {
	//TODO implement me
	panic("implement me")
}

// Publish 发布

func NewDefault(db *sqlx.DB) (d DataSourceDefault) {
	d.mysql = mysql.NewMysql(db)
	return
}
