package main

import (
	"encoding/json"
	"errors"
	"fmt"
	"github.com/valyala/bytebufferpool"
	"github.com/valyala/fasthttp"
	"net/url"
	"os"
	"os/user"
	"strings"
)

func CLI(args ...string) {
	if len(args) < 1 {
		println(CliHelp)
		return
	}
	err := CliSetup()

	if err != nil {
		return
	}
	switch args[0] {
	case "init":
		if len(args) != 2 {
			println(CliInitHelp)
			return
		}
		err = CliInit(args[1])
		if err != nil {
			println(err.Error())
			return
		}

	case "add":
		if len(args) < 2 {
			println(CliAddHelp)

			return
		}
		err = CliAdd(args[1:]...)
		if err != nil {
			println(err.Error())
			return
		}

	case "del":
		if len(args) < 2 {
			println(CliDelHelp)
			return
		}
		err = CliDel(args[1:]...)
		if err != nil {
			println(err.Error())
			return
		}

	case "search":
		if len(args) != 2 {
			println(CliSearchHelp)
			return
		}

		err = CliSearch(args[1:]...)
		if err != nil {
			println(err.Error())
			return
		}
	case "publish":
		if len(args) < 2 {
			println(CliPublishHelp)
			return
		}
		err = CliPublish(args[1:]...)
		if err != nil {
			println(err.Error())
			return
		}
	case "store":
		if len(args) == 1 {
			println(CliStoreHelp)
			return
		} else {
			switch args[1] {
			case "add":
				if len(args) < 3 {
					//请输入参数
					println(CliStoreAddHelp)
					return
				}
				err = CliStoreAdd(args[2:]...)
				if err != nil {
					println(err.Error())
					return
				}
			case "addfile":
				if len(args) < 3 {
					println(CliStoreAddFileHelp)
					return
				}
				err = CliStoreAddFile(args[2])
				if err != nil {
					println(err.Error())
					return
				}
			default:
				println(CliNotFound)
				return

			}
		}
		//无参命令
	case "tidy":
		err = CliTidy()
		if err != nil {
			println(err.Error())
			return
		}
	case "download":
		err = CliDownload(args[1:]...)
		if err != nil {
			println(err.Error())
			return
		}
	case "graph":
		err = CliGraph()
		if err != nil {
			println(err.Error())
			return
		}
	case "verify":
		err = CliVerify()
		if err != nil {
			println(err.Error())
			return
		}

	default:
		println(CliNotFound)
		println(CliHelp)
		//弹出使用方法
	}
}

// CliSetup 加载环境变量，初始化目录与设置
func CliSetup() error {
	var err error
	pwd, err = os.Getwd()
	if err != nil {
		return nil
	}
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
		KPM_ROOT+Separator+"registry"+Separator+KPM_SERVER_ADDR_PATH+Separator+"kcl_modules",
		KPM_ROOT+Separator+"registry"+Separator+KPM_SERVER_ADDR_PATH+Separator+"tag",
		KPM_ROOT+Separator+"registry"+Separator+KPM_SERVER_ADDR_PATH+Separator+"metadata",
		KPM_ROOT+Separator+"git",
		KPM_ROOT+Separator+"git"+Separator+"kcl_modules",
		KPM_ROOT+Separator+"git"+Separator+"metadata",
		KPM_ROOT+Separator+"store",
		KPM_ROOT+Separator+"store"+Separator+"v1",
		KPM_ROOT+Separator+"store"+Separator+"v1"+Separator+"files",
	)
	if err != nil {
		println("setup fail,", err.Error())
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
	version, err := GetKclvmMinVersion()
	if err == nil {
		KclvmMinVersion = "v" + version
	}

	return nil
}

// CliAdd 添加包，检查vm版本，如果比当前版本大，则失败，只负责链接或者复制
func CliAdd(args ...string) error {
	//flag_global := false
	flag_git := false
	//flag_internal := false
	var pkgvs []string
	//var pkgs []Require
	for i := 0; i < len(args); i++ {
		if strings.HasPrefix(args[i], "-") {
			switch args[i] {
			//case "-g":
			//	flag_global = true
			case "-git":
				flag_git = true
				//case "--internal":
				//	flag_internal = true
			}
		} else {
			pkgvs = args[i:]
			break
		}
	}
	//读取kpmfile
	kpmfilep, err := NewKpmFileP(pwd)
	if err != nil {
		return err
	}
	direct := kpmfilep.kpmfile.Direct
	directMap := make(map[string]Require, 16)
	for i := 0; i < len(direct); i++ {
		if direct[i].Alias == "" {
			if direct[i].Type != "git" {
				directMap[direct[i].Name] = direct[i]
			}
		} else {
			directMap[direct[i].Alias] = direct[i]
		}

	}
	//间接依赖，添加原则，唯一版本即可
	indirect := kpmfilep.kpmfile.Indirect
	indirectMap := make(map[string]Require, 16)
	for i := 0; i < len(indirect); i++ {
		indirectMap[indirect[i].Type+"|"+indirect[i].Name+"|"+indirect[i].GitAddress+"|"+indirect[i].Version+"|"+indirect[i].GitCommit] = indirect[i]
	}
	for i := 0; i < len(pkgvs); i++ {

		r := &Require{}
		err := r.NewRequireFromPkgString(pkgvs[i], flag_git)
		if err != nil {
			return err
		}

		err = r.Get(KPM_ROOT, KPM_SERVER_ADDR)
		if err != nil {
			return err
		}
		//检查命名是否冲突，先读，后写
		if r.Alias == "" {
			if r.Type != "git" {
				//仓库包
				_, stat := directMap[r.Name]
				if stat {
					//冲突
					println("Naming conflicts")
					continue
				}
				directMap[r.Name] = *r
			}
		} else {
			_, stat := directMap[r.Alias]
			if stat {
				//冲突
				println("Naming conflicts")
				continue
			}
			directMap[r.Alias] = *r
			file, err := os.ReadFile(r.KpmFileLocalPath(KPM_ROOT, KPM_SERVER_ADDR_PATH))
			if err == nil {
				//解析
				kpmfile := KpmFile{}
				err = json.Unmarshal(file, &kpmfile)
				if err != nil {
					return err
				}
				//检查kcl版本，如果高于当前版本则拒绝
				//工作版本
				ver := &Version{}
				err = ver.NewFromString(kpmfilep.kpmfile.KclvmMinVersion)
				if err != nil {
					return err
				}
				//当前解析依赖的版本
				nowver := &Version{}
				err = ver.NewFromString(kpmfile.KclvmMinVersion)
				if err != nil {
					return err
				}
				if ver.Cmp(*nowver) == -1 {
					//println("The current pending load dependency aabc needs to be greater than "+kpmfile.KclvmMinVersion+" version of KclvmMinVersion")
					return errors.New("The current pending load dependency aabc needs to be greater than " + kpmfile.KclvmMinVersion + " version of KclvmMinVersion")
				}
				//遍历得到直接依赖和间接依赖
				for j := 0; j < len(kpmfile.Direct); j++ {
					tmp := kpmfile.Direct[j]
					indirectMap[tmp.Type+"|"+tmp.Name+"|"+tmp.GitAddress+"|"+tmp.Version+"|"+tmp.GitCommit] = tmp
				}
				for j := 0; j < len(kpmfile.Indirect); j++ {
					tmp := kpmfile.Indirect[j]
					indirectMap[tmp.Type+"|"+tmp.Name+"|"+tmp.GitAddress+"|"+tmp.Version+"|"+tmp.GitCommit] = tmp
				}

			}
			//没有文件，不需要解析依赖
			//return err

		}
		err = r.LinkToExternal(KPM_ROOT, KPM_SERVER_ADDR_PATH, pwd)
		if err != nil {
			println(err.Error())
			return err
		}
	}
	//回填依赖数据并保存
	kpmfilep.kpmfile.Direct = kpmfilep.kpmfile.Direct[:0]
	for _, v := range directMap {
		kpmfilep.kpmfile.Direct = append(kpmfilep.kpmfile.Direct, v)
	}
	kpmfilep.kpmfile.Indirect = kpmfilep.kpmfile.Indirect[:0]
	for _, v := range indirectMap {
		kpmfilep.kpmfile.Indirect = append(kpmfilep.kpmfile.Indirect, v)
	}
	if debuglog {
		fmt.Println("directMap", directMap)
	}

	err = kpmfilep.Save()
	if err != nil {
		return err
	}
	return nil
}

// CliDel 移除链接,删除直接依赖的包信息,别名
func CliDel(args ...string) error {
	kpmfilep, err := NewKpmFileP(pwd)
	if err != nil {
		return err
	}
	direct := kpmfilep.kpmfile.Direct
	directMap := make(map[string]Require, 16)
	for i := 0; i < len(direct); i++ {
		if direct[i].Alias == "" {
			if direct[i].Type != "git" {
				directMap[direct[i].Name] = direct[i]
			}
		} else {
			directMap[direct[i].Alias] = direct[i]
		}

	}
	for i := 0; i < len(args); i++ {
		t, stat := directMap[args[i]]
		if !stat {
			//
			//println("del  dependencies", args[i], " fail,it does not exist in kpmfile")
			return errors.New("del  dependencies " + args[i] + " fail,it does not exist in kpmfile")
		}
		name := t.Alias
		if t.Alias == "" {
			name = t.Name
		}
		err = os.Remove(pwd + Separator + ExternalDependencies + Separator + name)
		if err != nil {
			println("del  dependencies", name, " fail")
			return err
		}
		delete(directMap, args[i])
		println("del  dependencies", name, " success")
	}
	kpmfilep.kpmfile.Direct = kpmfilep.kpmfile.Direct[:0]
	for _, v := range directMap {
		kpmfilep.kpmfile.Direct = append(kpmfilep.kpmfile.Direct, v)
	}

	err = kpmfilep.Save()
	if err != nil {
		return err
	}
	return nil
}
func CliDownload(args ...string) error {
	p, err := NewKpmFileP(pwd)
	if err != nil {
		return err
	}
	for i := 0; i < len(p.kpmfile.Indirect); i++ {
		rp := &p.kpmfile.Indirect[i]
		err = rp.Get(KPM_ROOT, KPM_SERVER_ADDR)
		if err != nil {
			return err
		}
	}
	for i := 0; i < len(p.kpmfile.Direct); i++ {
		rp := &p.kpmfile.Direct[i]
		err = rp.Get(KPM_ROOT, KPM_SERVER_ADDR)
		if err != nil {
			return err
		}
		err = rp.LinkToExternal(KPM_ROOT, KPM_SERVER_ADDR_PATH, pwd)
		if err != nil {
			return err
		}
	}
	return nil
}
func CliGraph() error {
	p, err := NewKpmFileP(pwd)
	if err != nil {
		return err
	}
	err = Graph(p.kpmfile)
	if err != nil {
		return err
	}
	return nil
}

func Graph(k *KpmFile) error {
	if k == nil {
		return nil
	}
	for i := 0; i < len(k.Direct); i++ {
		rp := &k.Direct[i]

		if rp.Type == "git" {
			if rp.Version == "" || rp.Version == "v0.0.0" {
				println(k.PackageName, rp.GitAddress+"@v0.0.0#"+rp.GitCommit)
			} else {
				println(k.PackageName, rp.GitAddress+"@"+rp.Version)
			}
		} else {
			println(k.PackageName, rp.Name+"@"+rp.Version)
		}
	}
	for i := 0; i < len(k.Direct); i++ {
		rp := &k.Direct[i]
		if rp.Type == "git" {
			if rp.Version == "" || rp.Version == "v0.0.0" {
				println(k.PackageName, rp.GitAddress+"@v0.0.0#"+rp.GitCommit)

			} else {
				println(k.PackageName, rp.GitAddress+"@"+rp.Version)
			}
		} else {
			println(k.PackageName, rp.Name+"@"+rp.Version)
		}
		//读取文件
		path := rp.LocalPath(KPM_ROOT, KPM_SERVER_ADDR_PATH)
		file, err := os.ReadFile(path)
		if err != nil {
			return err
		}
		pkginfo := PkgInfo{}
		err = json.Unmarshal(file, &pkginfo)
		if err != nil {
			return err
		}
		if pkginfo.KpmFileHash != "" {
			//路径
			readFile, err := os.ReadFile(KPM_ROOT + Separator + "store" + Separator + "v1" + Separator + "files" + Separator + HashMod([]byte(pkginfo.KpmFileHash)) + Separator + pkginfo.KpmFileHash)
			if err != nil {
				return err
			}
			kpmfile := KpmFile{}
			err = json.Unmarshal(readFile, &kpmfile)
			if err != nil {
				return err
			}
			err = Graph(&kpmfile)
		}

		if err != nil {
			return err
		}
	}
	return nil
}

func CliInit(pkg string) error {
	kpmfp := &KpmFileP{
		Path: pwd + Separator + "kpm.json",
		kpmfile: &KpmFile{
			PackageName:     pkg,
			KclvmMinVersion: KclvmMinVersion,
		},
	}
	err := kpmfp.Create()
	if err != nil {
		return errors.New("Create kpm.json fail!Because " + err.Error())
	}

	println("Create kpm.json success!")
	_, err = os.Stat(pwd + Separator + "kcl.mod")
	if err == nil {
		return nil
	}
	//文件不存在,所以创建
	err = os.WriteFile(pwd+Separator+"kcl.mod", []byte(DefaultKclModContent+`"`+KclvmMinVersion+`"`), 0777)
	if err != nil {
		return err
	}
	return nil
}

func CliPublish(args ...string) error {
	compress := "br"
	pkgv := strings.Split(args[0], "@")
	if len(pkgv) != 2 {
		return errors.New("ArgsWrong")
	}
	pkginfo := NewPkgInfo(pkgv[0], pkgv[1], pwd)
	//先打包目录
	buffer, err := pkginfo.CreatePublishTarByteBuffer(KPM_ROOT, compress)
	if err != nil {
		return err
	}
	req := fasthttp.AcquireRequest()
	defer fasthttp.ReleaseRequest(req)
	req.Header.SetMethod("POST")
	req.Header.Set("X-KPM-PKG-COMPRESS", compress)
	req.SetHost(KPM_SERVER_ADDR_PATH)
	req.SetRequestURI(KPM_SERVER_ADDR + "/api/v1/u/publish")
	req.SetBodyRaw(buffer.B)
	bytebufferpool.Put(buffer)
	resp := fasthttp.AcquireResponse()
	defer fasthttp.ReleaseResponse(resp)
	println(req.Header.String())
	if err = fasthttp.Do(req, resp); err != nil {
		return err
	}

	if resp.StatusCode() != 200 {
		return errors.New("fetch " + KPM_SERVER_ADDR + " err")
	}
	stdresp := StdResp{}
	err = json.Unmarshal(resp.Body(), &stdresp)
	if err != nil {
		return err
	}
	if stdresp.Code != 0 {

		return errors.New("fetch " + KPM_SERVER_ADDR + " failed")
	}
	println("publish success!")
	//本地生成info，服务器反馈需要上传的包hash文件，上传hash文件，服务器开始校验
	return nil
}

// CliSearch 在线模糊搜索或者精准搜索包，不支持git包
func CliSearch(args ...string) error {
	req := fasthttp.AcquireRequest()
	defer fasthttp.ReleaseRequest(req)
	req.Header.SetMethod("GET")
	req.SetHost(KPM_SERVER_ADDR_PATH)
	req.SetRequestURI(KPM_SERVER_ADDR + "/api/v1/search")
	req.URI().QueryArgs().Set("pkgname", args[0])
	resp := fasthttp.AcquireResponse()
	defer fasthttp.ReleaseResponse(resp)
	if err := fasthttp.Do(req, resp); err != nil {
		return err
	}
	if resp.StatusCode() != 200 {
		return errors.New("fetch " + KPM_SERVER_ADDR + " failed")
	}
	pkgsresp := SearchPkgsResp{}
	err := json.Unmarshal(resp.Body(), &pkgsresp)
	if err != nil {
		return err
	}
	if pkgsresp.Code != 0 {
		return errors.New("fetch " + KPM_SERVER_ADDR + " failed")
	}
	if len(pkgsresp.Data) == 0 {
		println("Search results is empty")
		return nil
	}
	println("Name", "Version", "Description")
	for i := 0; i < len(pkgsresp.Data); i++ {
		println(pkgsresp.Data[i].Name, pkgsresp.Data[i].Version, pkgsresp.Data[i].Description)
	}
	return nil
}

func CliTidy() error {
	rq, err := FindRequires(pwd)
	if err != nil {
		return err
	}
	subpkgMap := make(map[string]Set, 16)
	for i := 0; i < len(rq); i++ {
		if strings.HasPrefix(rq[i], ExternalDependencies+".") {
			dotcount := 0
			var pkgAlias []byte
			var subpkg string
			for j := 0; j < len(rq[i]); j++ {
				if rq[i][j] == '.' {
					dotcount++
				}
				if dotcount == 1 {
					pkgAlias = append(pkgAlias, rq[i][j])
				}
				if dotcount == 2 {
					subpkg = rq[i][j+1:]
					break
				}
			}
			set, exist := subpkgMap[string(pkgAlias[1:])]
			if exist {
				set.SAdd(subpkg)
			} else {
				subpkgMap[string(pkgAlias[1:])] = AcquireSet()
			}
		}

	}

	//先过滤extern，再得到别名，再通过子包搜索是在哪个包下

	return nil
}
func CliVerify() error {
	return nil
}
func CliStoreAdd(args ...string) error {
	flag_git := false
	var pkgvs []string
	for i := 0; i < len(args); i++ {
		if strings.HasPrefix(args[i], "-") {
			switch args[i] {
			case "-git":
				flag_git = true
			}
		} else {
			pkgvs = args[i:]
			break
		}
	}
	for i := 0; i < len(pkgvs); i++ {
		//args[i]
		rp := &Require{}
		err := rp.NewRequireFromPkgString(pkgvs[i], flag_git)
		if err != nil {
			return err
		}
		err = rp.Get(KPM_ROOT, KPM_SERVER_ADDR)
		if err != nil {
			return err
		}
	}
	return nil
}
func CliStoreAddFile(fpath string) error {
	err := StoreAddFile(fpath, KPM_ROOT, true)
	if err != nil {
		return err
	}
	return nil
}
