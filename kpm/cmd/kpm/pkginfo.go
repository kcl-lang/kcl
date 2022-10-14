package main

import (
	"archive/tar"
	"bytes"
	"crypto/sha512"
	"encoding/json"
	"github.com/valyala/bytebufferpool"
	"github.com/valyala/fasthttp"
	"io"
	"kpm/cmd/kpmserverd/application"
	"os"
	"path/filepath"
	"sort"
	"strings"
	"time"
)

type PkgInfo struct {
	//包名
	PackageName string `json:"name"`
	//版本
	PackageVersion string `json:"version"`
	//包大小
	PackageSize int64 `json:"package_size"`
	//整个项目的sha512校验和
	Integrity string `json:"integrity"`
	//kpmfile校验和
	KpmFileHash string `json:"kpm_file_hash,omitempty"`
	//kclmod的校验和
	KclModFileHash string `json:"kcl_mod_file_hash,omitempty"`
	//子包
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

func NewPkgInfo(pkgName, pkgVersion, pkgPath string) (pkginfo PkgInfo) {
	pkginfo.PackageName = pkgName
	pkginfo.PackageVersion = pkgVersion
	var sums []string
	require := NewSet()
	err := filepath.Walk(pkgPath, func(path string, info os.FileInfo, err error) error {

		if info.IsDir() {
			//跳过文件夹
			return nil
		}
		rel, err := filepath.Rel(pkgPath, path)
		if err != nil {
			return err
		}
		//跳过统计依赖目录
		if strings.HasPrefix(rel, ExternalDependencies+Separator) {
			return nil
		}
		if strings.HasPrefix(rel, ".git"+Separator) {
			return nil
		}
		//添加fileinfo，校验，大小，时间
		filebyte, err := os.ReadFile(path)
		if err != nil {
			return err
		}
		rp := []byte(rel)
		//统一为Linux下的分隔符
		for i := 0; i < len(rp); i++ {
			if rp[i] == '\\' {
				rp[i] = '/'
			}
		}
		fileinfo := FileInfo{
			Path:      string(rp),
			Integrity: EncodeToString(sha512.Sum512(filebyte)),
			Size:      info.Size(),
		}
		pkginfo.PackageSize += fileinfo.Size
		pkginfo.Files = append(pkginfo.Files, fileinfo)
		//生成校验

		//fmt.Println(string(rp))
		//文件相对路径
		rph := EncodeToString(sha512.Sum512(rp))
		//文件内容
		fh := EncodeToString(sha512.Sum512(filebyte))
		sum := EncodeToString(sha512.Sum512([]byte(rph + fh)))
		sums = append(sums, sum)
		//如果是kpm文件，则添加
		switch rel {
		case "kpm.json":
			pkginfo.KpmFileHash = fileinfo.Integrity
		case "kcl.mod":
			pkginfo.KclModFileHash = fileinfo.Integrity
		}
		//如果是k文件
		pathlen := len(info.Name())
		if pathlen < 2 {
			//跳过文件名长度小于等于2的文件
			return nil
		}

		if !strings.HasSuffix(info.Name(), ".k") {
			//跳过不是.k的文件
			return nil
		}
		//把模块转换成包
		namelen := len(info.Name())
		rplen := len(rp)
		pkglen := rplen - namelen - 1

		if pkglen > 0 {
			pkgpath := rp[:pkglen]
			for i := 0; i < len(pkgpath); i++ {
				if rp[i] == '/' {
					rp[i] = '.'
				}
			}
			require.SAdd(string(pkgpath))
		}
		return nil
	})
	if err != nil {
		return
	}
	tmprequires := require.SMembers()
	sort.Strings(tmprequires)
	pkginfo.SubPkgPath = tmprequires
	sort.Strings(sums)
	var sumstr string
	for i := 0; i < len(sums); i++ {
		sumstr += sums[i]
	}
	pkginfo.Integrity = EncodeToString(sha512.Sum512([]byte(sumstr)))
	return
}

// Build 构建包，如果构建失败则删除构建目录
func (p PkgInfo) Build(kpmroot, buildpath string) error {
	err := KeepDirExists(buildpath)
	if err != nil {
		err2 := os.RemoveAll(buildpath)
		if err2 != nil {
			return err2
		}
		return err
	}
	for i := 0; i < len(p.Files); i++ {
		from := kpmroot + Separator + "store" + Separator + "v1" + Separator + "files" + Separator + HashMod(application.S2B(p.Files[i].Integrity)) + Separator + p.Files[i].Integrity
		dirlevel := strings.Split(p.Files[i].Path, "/")
		to := buildpath
		for j := 0; j < len(dirlevel)-1; j++ {
			to += Separator
			to += dirlevel[j]
			err = KeepDirExists(to)
			if err != nil {
				err2 := os.RemoveAll(buildpath)
				if err2 != nil {
					return err2
				}
				return err
			}
		}
		to += Separator + dirlevel[len(dirlevel)-1]
		err = os.Link(from, to)
		if err != nil {
			err2 := os.RemoveAll(buildpath)
			if err2 != nil {
				return err2
			}
			return err
		}

	}
	return nil
}

func (p PkgInfo) CreatePublishTarByteBuffer(kpmroot string, compress string) (*bytebufferpool.ByteBuffer, error) {
	//files/*
	//pkginfo.json
	pkginfojson, err := json.Marshal(p)
	if err != nil {
		return nil, err
	}

	b := bytebufferpool.Get()
	defer bytebufferpool.Put(b)
	tw := tar.NewWriter(b)
	defer tw.Close()

	//写入pkginfo.json
	h := new(tar.Header)
	h.Name = "pkginfo.json"
	h.Size = int64(len(pkginfojson))
	h.Mode = 0777
	h.ModTime = time.Now()
	err = tw.WriteHeader(h)
	if err != nil {
		return nil, err
	}
	_, err = io.Copy(tw, bytes.NewReader(pkginfojson))
	if err != nil {
		return nil, err
	}
	//tar压缩
	for i := 0; i < len(p.Files); i++ {
		hashfilepath := kpmroot + Separator + "store" + Separator + "v1" + Separator + "files" + Separator + HashMod([]byte(p.Files[i].Integrity)) + Separator + p.Files[i].Integrity
		f, err := os.Open(hashfilepath)
		if err != nil {
			return nil, err
		}

		info, err := f.Stat()
		if err != nil {
			f.Close()
			return nil, err
		}
		// 文件信息写入tar的头
		header, err := tar.FileInfoHeader(info, "")
		if err != nil {
			f.Close()
			return nil, err
		}
		header.Name = "files/" + p.Files[i].Integrity
		err = tw.WriteHeader(header)
		if err != nil {
			f.Close()
			return nil, err
		}
		_, err = io.Copy(tw, f)
		if err != nil {
			f.Close()
			return nil, err
		}
		f.Close()
	}
	b2 := bytebufferpool.Get()
	switch compress {
	case "br":
		_, err = fasthttp.WriteBrotliLevel(b2, b.B, fasthttp.CompressBrotliBestCompression)
		if err != nil {
			return nil, err
		}
		return b2, nil
	case "gz":
		_, err = fasthttp.WriteGzipLevel(b2, b.B, fasthttp.CompressBestCompression)
		if err != nil {
			return nil, err
		}
		return b2, nil
	default:
		b2.Write(b.B)
	}
	return b2, nil
}
