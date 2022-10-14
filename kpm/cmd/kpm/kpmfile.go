package main

import (
	"encoding/json"
	"errors"
	"github.com/valyala/fasthttp"
	"kpm/cmd/kpmserverd/application"
	"net/url"
	"os"
	"strings"
)

type KpmFile struct {
	//包名，确定包的命名空间
	PackageName string `json:"package_name"`
	//确定此包的kcl最低运行版本
	KclvmMinVersion string `json:"kclvm_min_version"`
	//直接依赖，别名不重复
	Direct []Require `json:"direct,omitempty"`
	//间接依赖，不看别名，包名版本唯一即可
	Indirect []Require `json:"indirect,omitempty"`
}
type Require struct {
	//别名
	Alias string `json:"alias,omitempty"`
	//包名，确定包的命名空间
	Name string `json:"name,omitempty"`
	//确定此包的版本
	Version string `json:"version,omitempty"`
	//校验和 sha512
	Integrity string `json:"integrity"`
	//包类型 git，registry
	Type string `json:"type"`
	//git包地址
	GitAddress string `json:"git_address,omitempty"`
	//git包commit id
	GitCommit string `json:"git_commit,omitempty"`
}

func (r *Require) NewRequireFromPkgString(pkgv string, gitflag bool) error {
	// konfig
	// konfig@0.1.1
	// https://github.com/kusionstack/konfig@v0.1.0
	// https://github.com/kusionstack/konfig@v0.0.0#48f2f892637e4d4d932790dbaf5606fcb267e800
	//
	//如果带有精确版本，则先get
	//如果不带，则先尝试获取这个包最新版本
	//如果是git包，直接拉取最新版
	//如果是仓库包，则直接访问接口
	//读取包元数据反序列化在Require上
	result := strings.Split(pkgv, "@")
	if gitflag {
		r.SetPackageTypeGit()
		r.GitAddress = result[0]
	} else {
		r.SetPackageTypeRegistry()
	}
	if len(result) == 1 {
		//不带版本

		//如果是git包，直接拉取最新版
		if gitflag {
			tmp := os.TempDir() + Separator + application.B2S(application.RandBytes32())
			err := KeepDirExists(tmp)
			if err != nil {
				return err
			}
			err = RunCmd(tmp, "git", "clone", r.GitAddress)
			if err != nil {
				return err
			}
			gitaddrslice := strings.Split(r.GitAddress, "/")
			gitaddrslicelen := len(gitaddrslice)
			if gitaddrslicelen > 1 {
				tmp += Separator
				tmp += gitaddrslice[gitaddrslicelen-1]
				r.Alias = gitaddrslice[gitaddrslicelen-1]
			}
			stdout, err := RunCmdWithStdout(tmp, "git", "rev-parse", "HEAD")
			if err != nil {
				return err
			}
			r.GitCommit = strings.TrimRight(stdout, "\n")
			err = r.IsInLocal(KPM_ROOT, KPM_SERVER_ADDR_PATH)
			if err != nil {
				//不在本地
				pkginfo := NewPkgInfo(r.GitAddress, "v0.0.0#"+r.GitCommit, tmp)
				err = StoreAddFile(tmp, KPM_ROOT, false)
				if err != nil {
					return err
				}
				marshal, err := json.Marshal(pkginfo)
				if err != nil {
					return err
				}
				err = os.MkdirAll(FilePathToDirPath(r.PkgInfoLocalPath(KPM_ROOT, KPM_SERVER_ADDR_PATH)), 0777)
				if err != nil {
					return err
				}
				err = os.WriteFile(r.PkgInfoLocalPath(KPM_ROOT, KPM_SERVER_ADDR_PATH), marshal, 0777)
				if err != nil {
					//println(7, r.PkgInfoLocalPath(KPM_ROOT, KPM_SERVER_ADDR_PATH), err.Error())
					return err
				}
				err = StoreAddFile(tmp, KPM_ROOT, false)

				if err != nil {
					return err
				}
				r.Integrity = pkginfo.Integrity
			} else {
				//在本地
				file, err := os.ReadFile(r.PkgInfoLocalPath(KPM_ROOT, KPM_SERVER_ADDR_PATH))
				if err != nil {
					return err
				}
				pkginfo := PkgInfo{}
				err = json.Unmarshal(file, &pkginfo)
				if err != nil {
					return err
				}
				r.Integrity = pkginfo.Integrity
			}

		} else {
			r.Name = result[0]
			//如果是仓库包，则直接访问接口
			targeturi := KPM_SERVER_ADDR + "/s/tag/" + r.Name + "/latest"
			req := fasthttp.AcquireRequest()
			defer fasthttp.ReleaseRequest(req)
			req.Header.SetMethod("GET")
			req.SetRequestURI(targeturi)
			resp := fasthttp.AcquireResponse()
			defer fasthttp.ReleaseResponse(resp)
			if err := fasthttp.Do(req, resp); err != nil {
				return err
			}
			if resp.StatusCode() != 200 {
				return errors.New("fetch " + targeturi + " err")
			}
			if resp.Body() == nil || len(resp.Body()) == 0 {
				return errors.New("fetch " + targeturi + "data err")
			}
			r.Version = string(resp.Body())
			err := r.Get(KPM_ROOT, KPM_SERVER_ADDR)
			if err != nil {
				return err
			}
		}

	} else {
		//带版本

		if gitflag {

			gitaddrslice := strings.Split(r.GitAddress, "/")
			gitaddrslicelen := len(gitaddrslice)
			if gitaddrslicelen > 1 {
				r.Alias = gitaddrslice[gitaddrslicelen-1]
			}
			r.Version = result[1]
			result2 := strings.Split(result[1], "#")
			if len(result2) != 1 {
				r.Version = ""
				r.GitCommit = result2[1]
			}

		} else {
			r.Name = result[0]
			r.Version = result[1]
		}
		err := r.Get(KPM_ROOT, KPM_SERVER_ADDR)
		if err != nil {
			return err
		}
	}

	if debuglog {
		marshal, err := json.Marshal(r)
		if err != nil {
			return err
		}
		println("NewRequireFromPkgString:", string(marshal))
	}

	return nil
}

func (r *Require) SetPackageTypeGit() {
	r.Type = "git"
}
func (r *Require) SetPackageTypeRegistry() {
	r.Type = "registry"
}

// PkgDownload  下载包元数据与数据 info
func (r *Require) PkgDownload(kpmroot, kpmserver string) error {
	kpmserverurl, err := url.Parse(kpmserver)
	if err != nil {
		return err
	}
	kpmserverpath := kpmserverurl.Host

	if r.Type == "git" {
		tmp := os.TempDir() + Separator + application.B2S(application.RandBytes32())
		err := KeepDirExists(tmp)
		if err != nil {
			return err
		}
		//如果有版本，则使用版本，如果没有，则使用commit id
		if r.Version == "" || r.Version == "v0.0.0" {
			err = RunCmd(tmp, "git", "init")
			if err != nil {
				return err
			}
			err = RunCmd(tmp, "git", "remote", "add", "origin", r.GitAddress)
			if err != nil {
				return err
			}
			err = RunCmd(tmp, "git", "fetch", "origin", r.GitCommit)
			if err != nil {
				//println(5, err.Error())
				//return err
			}
			err = RunCmd(tmp, "git", "reset", "--hard", "FETCH_HEAD")
			if err != nil {
				return err
			}

		} else {
			//marshal, err := json.Marshal(r)
			//if err != nil {
			//	return err
			//}
			//fmt.Println("ttt", string(marshal))
			//println("gitaddr", r.GitAddress)

			//git clone --branch [tag] [git地址]
			err = RunCmd(tmp, "git", "clone", "--branch", r.Version, r.GitAddress)
			if err != nil {
				return err
			}
			gitaddrslice := strings.Split(r.GitAddress, "/")
			gitaddrslicelen := len(gitaddrslice)
			if gitaddrslicelen > 1 {
				tmp += Separator
				tmp += gitaddrslice[gitaddrslicelen-1]
			}

		}
		var ver string
		if r.Version == "" || r.Version == "v0.0.0" {
			ver = "v0.0.0#" + r.GitCommit
		} else {
			ver = r.Version
		}
		pkginfo := NewPkgInfo(r.GitAddress, ver, tmp)
		err = StoreAddFile(tmp, kpmroot, false)
		if err != nil {
			return err
		}
		marshal, err := json.Marshal(pkginfo)
		if err != nil {
			return err
		}
		//println(7, string(marshal))
		//err = KeepDirExists()
		//if err != nil {
		//	return err
		//}
		err = os.MkdirAll(FilePathToDirPath(r.PkgInfoLocalPath(kpmroot, kpmserverpath)), 0777)
		if err != nil {
			return err
		}
		err = os.WriteFile(r.PkgInfoLocalPath(kpmroot, kpmserverpath), marshal, 0777)
		if err != nil {
			//println(7, r.PkgInfoLocalPath(kpmroot, kpmserverpath), err.Error())
			return err
		}
		// /root/kpm/git/kcl_modules
		//git clone到临时目录，校验，hash单文件移动到store，硬链接文件到src，生成hash和info

	} else {
		//registry
		targeturi := kpmserver + "/s/metadata/" + r.Name + "/" + r.Version + ".json"
		req := fasthttp.AcquireRequest()
		defer fasthttp.ReleaseRequest(req)
		req.Header.SetMethod("GET")
		req.SetRequestURI(targeturi)
		resp := fasthttp.AcquireResponse()
		defer fasthttp.ReleaseResponse(resp)
		if err := fasthttp.Do(req, resp); err != nil {
			return err
		}
		if resp.StatusCode() != 200 {
			return errors.New("fetch " + targeturi + " err")
		}
		pkginfo := PkgInfo{}
		err := json.Unmarshal(resp.Body(), &pkginfo)
		if err != nil {
			return err
		}
		for i := 0; i < len(pkginfo.Files); i++ {
			//检查本地是否有文件，如果没有，则下载
			fpath := kpmroot + Separator + "store" + Separator + "v1" + Separator + "files" + Separator + HashMod(application.S2B(pkginfo.Files[i].Integrity)) + Separator + pkginfo.Files[i].Integrity
			exists, err := PathExists(fpath)
			if err != nil {
				return err
			}
			if !exists {
				req.SetRequestURI("/s/store/v1/files/" + HashMod(application.S2B(pkginfo.Files[i].Integrity)) + "/" + pkginfo.Files[i].Integrity)
				resp.Reset()
				if err = fasthttp.Do(req, resp); err != nil {
					return err
				}
				if resp.StatusCode() != 200 {
					return errors.New("fetch " + req.URI().String() + " err")
				}
				//校验下载文件
				if pkginfo.Files[i].Integrity != HashMod(resp.Body()) {
					//文件损坏
					return errors.New("the download file is corrupted")
				}

				//写入文件
				err = os.WriteFile(fpath, resp.Body(), 0777)
				if err != nil {
					return err
				}
			}
		}
		//写元数据

		//获取info，下载单文件校验，hash单文件移动到store，硬链接文件到src
		// /root/kpm/registry/kpm.kusionstack.io/kcl_modules
	}

	return nil
}

// Get GetPkg 保证依赖存在
func (r *Require) Get(kpmroot, kpmserver string) error {
	kpmserverurl, err := url.Parse(kpmserver)
	if err != nil {
		return err
	}
	kpmserverpath := kpmserverurl.Host
	//检测包目录是否存在，如果不存在则使用本地元文件构建，如果没有元文件，则下载
	if r.IsInLocal(kpmroot, kpmserverpath) != nil {
		println("not found pkg", r.ToString())
		if r.PkgInfoIsInLocal(kpmroot, kpmserverpath) != nil {
			println("not found pkginfo", r.ToString())
			err = r.PkgDownload(kpmroot, kpmserver)
			if err != nil {
				return err
			}
			println("downloading", r.ToString())
		}
		println("building", r.ToString())
		err = r.Build(kpmroot, kpmserverpath)
		if err != nil {
			return err
		}
	} else {
		if r.PkgInfoIsInLocal(kpmroot, kpmserverpath) != nil {

		}
		println("found", r.ToString())
	}

	return nil
}
func (r *Require) ToString() (pkgv string) {
	if r.Type == "git" {
		if r.Version == "" || r.Version == "v0.0.0" {
			pkgv = r.GitAddress + "@v0.0.0#" + r.GitCommit
		} else {
			pkgv = r.GitAddress + "@" + r.Version
		}
	} else {
		pkgv = r.Name + "@" + r.Version
	}

	return
}
func (r *Require) LocalPath(kpmroot, kpmserverpath string) (path string) {
	if r.Type == "git" {

		gitaddrslice := strings.Split(r.GitAddress, "/")
		gitpath := ""
		if len(gitaddrslice) > 2 {

			for i := 0; i < len(gitaddrslice[2:]); i++ {
				gitpath += Separator
				gitpath += gitaddrslice[i+2]

			}
		}

		if r.Version == "" || r.Version == "v0.0.0" {
			path = kpmroot + Separator + "git" + Separator + "kcl_modules" + gitpath + "@v0.0.0#" + r.GitCommit
		} else {
			path = kpmroot + Separator + "git" + Separator + "kcl_modules" + gitpath + "@" + r.Version
		}

	} else {
		path = kpmroot + Separator + "registry" + Separator + kpmserverpath + Separator + "kcl_modules" + Separator + r.Name + "@" + r.Version

	}
	return
}
func (r *Require) PkgInfoLocalPath(kpmroot, kpmserverpath string) (path string) {
	if r.Type == "git" {
		gitaddrslice := strings.Split(r.GitAddress, "/")
		gitpath := ""
		if len(gitaddrslice) > 2 {

			for i := 0; i < len(gitaddrslice[2:]); i++ {
				gitpath += Separator
				gitpath += gitaddrslice[i+2]

			}
		}
		if r.Version == "" || r.Version == "v0.0.0" {
			path = kpmroot + Separator + "git" + Separator + "metadata" + gitpath + "@v0.0.0#" + r.GitCommit + ".json"
		} else {
			path = kpmroot + Separator + "git" + Separator + "metadata" + gitpath + "@" + r.Version + ".json"
		}

	} else {
		path = kpmroot + Separator + "registry" + Separator + kpmserverpath + Separator + "metadata" + Separator + r.Name + "@" + r.Version + ".json"

	}
	return
}
func (r *Require) KpmFileLocalPath(kpmroot, kpmserverpath string) (path string) {
	path = r.LocalPath(kpmroot, kpmserverpath) + Separator + "kpm.json"
	return
}
func (r *Require) IsInLocal(kpmroot, kpmserverpath string) error {
	//检测包目录是否存在
	b, err := PathExists(r.LocalPath(kpmroot, kpmserverpath))
	if err != nil {
		return err
	}
	if !b {
		return errors.New("don't exist")
	}
	return nil
}
func (r *Require) PkgInfoIsInLocal(kpmroot, kpmserverpath string) error {
	path := r.PkgInfoLocalPath(kpmroot, kpmserverpath)
	//检测元文件是否存在
	b, err := PathExists(path)
	if err != nil {
		return err
	}
	if !b {
		return errors.New("don't exist")
	}
	file, err := os.ReadFile(path)
	if err != nil {
		return err
	}
	pkginfo := PkgInfo{}
	err = json.Unmarshal(file, &pkginfo)
	if err != nil {
		return err
	}
	r.Integrity = pkginfo.Integrity
	return nil

}
func (r *Require) Build(kpmroot, kpmserverpath string) error {
	path := r.PkgInfoLocalPath(kpmroot, kpmserverpath)
	println(path)
	file, err := os.ReadFile(path)

	if err != nil {

		return err
	}
	pkginfo := PkgInfo{}
	err = json.Unmarshal(file, &pkginfo)
	if err != nil {

		return err
	}
	//则使用本地元文件构建
	err = os.MkdirAll(r.LocalPath(kpmroot, kpmserverpath), 0777)
	if err != nil {
		return err
	}
	err = pkginfo.Build(kpmroot, r.LocalPath(kpmroot, kpmserverpath))
	if err != nil {
		return err
	}

	if pkginfo.KpmFileHash != "" {
		readFile, err := os.ReadFile(KPM_ROOT + Separator + "store" + Separator + "v1" + Separator + "files" + Separator + HashMod([]byte(pkginfo.KpmFileHash)) + Separator + pkginfo.KpmFileHash)
		if err != nil {
			return err
		}
		kpmfile := KpmFile{}
		err = json.Unmarshal(readFile, &kpmfile)
		if err != nil {
			return err
		}

		for i := 0; i < len(kpmfile.Direct); i++ {
			kd := kpmfile.Direct[i]
			err = kd.LinkToExternal(kpmroot, kpmserverpath, kd.LocalPath(kpmroot, kpmserverpath))
			if err != nil {
				return err
			}
		}
	}
	return nil
}
func (r *Require) LinkToExternal(kpmroot, kpmserverpath, pwd string) error {
	path := r.LocalPath(kpmroot, kpmserverpath)

	//源路径
	//_ = path

	//目标路径
	//_ = pwd + Separator + ExternalDependencies + Separator + r.Alias
	err := KeepDirExists(pwd + Separator + ExternalDependencies)
	if err != nil {
		return err
	}
	if r.Alias == "" {
		err = os.Symlink(path, pwd+Separator+ExternalDependencies+Separator+r.Name)
	} else {
		err = os.Symlink(path, pwd+Separator+ExternalDependencies+Separator+r.Alias)
	}

	if err != nil {
		return err
	}
	return nil
}
