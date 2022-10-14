package service

import (
	"encoding/json"
	"kpm/cmd/kpmserverd/application"
	"kpm/cmd/kpmserverd/response"
	"path/filepath"
)

var log = application.GetLogger()

const Separator = string(filepath.Separator)

type DataSourceMock struct {
}
type PkgInfo struct {
	PackageName    string `json:"name"`
	PackageVersion string `json:"version"`
	PackageSize    int64  `json:"package_size"`
	Integrity      string `json:"integrity"`
	KpmFileHash    string `json:"kpm_file_hash,omitempty"`
	KclModFileHash string `json:"kcl_mod_file_hash,omitempty"`
	//目录,排序
	SubPkgPath []string `json:"sub_pkg_path"`
	//文件信息列表
	Files []FileInfo `json:"files"`
}

type FileInfo struct {
	//文件路径
	Path string `json:"path"`
	//校验和
	Integrity string `json:"integrity"`
	//文件大小
	Size int64 `json:"size"`
}

func (d DataSourceMock) SearchSubPkgName(subPkgName string) string {
	result, err := json.Marshal(response.SearchPkgsResp{
		Code: 0,
		Msg:  "ok",
		Data: []response.SearchPkg{{
			Name:        "test",
			Description: "Description test",
			Version:     "v0.1.1",
		}, {
			Name:        "test2",
			Description: "Description test2",
			Version:     "v0.1.2",
		}},
	})
	if err != nil {
		return response.StdErrResp
	}
	return string(result)
}

func (d DataSourceMock) SearchName(name string) string {
	result, err := json.Marshal(response.SearchPkgsResp{
		Code: 0,
		Msg:  "ok",
		Data: []response.SearchPkg{{
			Name:        name,
			Description: "Description " + name,
			Version:     "v0.1.1",
		}, {
			Name:        name + " test",
			Description: "Description " + name,
			Version:     "v0.1.2",
		}},
	})
	if err != nil {
		return response.StdErrResp
	}
	return string(result)
}

func (d DataSourceMock) SearchSubPkgNames(subPkgNames []string) string {
	//TODO implement me

	panic("implement me")
}

func (d DataSourceMock) Publish(pkgtgz []byte, compress, kpmroot, kpmserver, kpmserverpath string) string {

	result, err := json.Marshal(response.StdResp{
		Code: 0,
		Msg:  "ok",
		Data: "",
	})
	if err != nil {
		return response.StdErrResp
	}
	return string(result)
}

func NewMock() DataSourceMock {
	return DataSourceMock{}
}
