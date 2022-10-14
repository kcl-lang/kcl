package main

import (
	"bufio"
	"bytes"
	"crypto/sha512"
	"errors"
	"io"
	"os"
	"os/exec"
	"path/filepath"
	"sort"
	"strings"
)

// KeepDirExists 确保某目录一定存在
func KeepDirExists(paths ...string) error {
	for i := 0; i < len(paths); i++ {
		exists, err := PathExists(paths[i])
		if err != nil {
			return err
		}
		if !exists {
			//不存在，所以创建
			err = os.Mkdir(paths[i], os.ModePerm)
			if err != nil {
				return err
			}
		}
	}
	return nil
}
func GetKclvmMinVersion() (string, error) {
	t, err := RunCmdWithStdout(KPM_ROOT, "kcl", "-V")
	if err != nil {
		return "", err
	}
	f := strings.Split(t, " ")
	for i := 0; i < len(f); i++ {
		g := strings.Split(f[i], ".")
		if len(g) == 3 {
			v := strings.TrimRight(f[i], ";")
			return v, nil
		}
	}
	return "", errors.New("not found")
}

//func Mkdirs(path string) error {
//	dirlevel := strings.Split(path, Separator)
//	tmp := ""
//	for i := 0; i < len(dirlevel); i++ {
//		if i != 0 {
//			tmp += Separator
//		}
//		tmp += dirlevel[i]
//		err := KeepDirExists(tmp)
//		if err != nil {
//			return err
//		}
//	}
//	return nil
//}

func FilePathToDirPath(str string) string {
	tmp := strings.Split(str, Separator)
	tmplen := len(tmp)
	if tmplen > 1 {
		tmp2len := len(tmp[tmplen-1])
		tmp3len := len(str)
		return str[:tmp3len-tmp2len]
	}
	return str
}
func PathExists(path string) (bool, error) {
	_, err := os.Stat(path)
	if err == nil {

		return true, nil
	}
	if os.IsNotExist(err) {

		return false, nil
	}

	return false, err
}
func RunCmd(dir string, name string, args ...string) error {
	cmd := exec.Command(
		name, args...)
	cmd.Dir = dir
	if debuglog {
		println("cmd dir: ", dir)
		print(" cmd args: ")
		for i := 0; i < len(args); i++ {
			print(args[i], " ")
		}
		println()
	}
	err := cmd.Start()
	if err != nil {
		if debuglog {
			println(err.Error())
		}
		return err
	}
	err = cmd.Wait()
	if err != nil {
		if debuglog {
			println(err.Error())
		}
		return err
	}
	return nil
}
func RunCmdWithStdout(dir string, name string, args ...string) (string, error) {
	cmd := exec.Command(
		name, args...)
	cmd.Dir = dir
	var stdout, stderr bytes.Buffer
	cmd.Stdout = &stdout
	cmd.Stderr = &stderr
	err := cmd.Start()
	if err != nil {
		if debuglog {
			println(err.Error())
		}
		return "", err
	}
	err = cmd.Wait()
	if err != nil {
		if debuglog {
			println(err.Error())
		}
		return "", err
	}
	outStr, errStr := stdout.String(), stderr.String()
	if err != nil {
		if debuglog {
			println(err.Error())
		}
		return "", err
	}
	if errStr != "" {
		return "", errors.New(errStr)
	}
	return outStr, nil
}

// VerifyDir 验证计算目录的sha512
func VerifyDir(rpath string) (string, error) {
	var sumlists []string
	err := filepath.Walk(rpath,
		func(path string, info os.FileInfo, err error) error {
			if err != nil {
				if debuglog {
					println(err.Error())
				}
				return err
			}
			//fmt.Println(path)
			//获取相对路径到结构体切片
			if info.IsDir() {
				//跳过文件夹
				return nil
			}
			file, err2 := os.ReadFile(path)
			if err2 != nil {
				if debuglog {
					println(err2.Error())
				}
				return err2
			}

			rel, err := filepath.Rel(rpath, path)
			if err != nil {
				if debuglog {
					println(err.Error())
				}
				return err
			}
			rp := []byte(rel)
			//统一为Linux下的分隔符
			for i := 0; i < len(rp); i++ {
				if rp[i] == '\\' {
					rp[i] = '/'
				}
			}
			//fmt.Println(string(rp))

			rph := EncodeToString(sha512.Sum512(rp))
			fh := EncodeToString(sha512.Sum512(file))
			sum := EncodeToString(sha512.Sum512([]byte(rph + fh)))
			sumlists = append(sumlists, sum)
			//fmt.Println(sum)
			return nil
		})
	if err != nil {
		if debuglog {
			println(err.Error())
		}
		return "", err
	}
	//排序
	sort.Strings(sumlists)
	var sumstr string
	for i := 0; i < len(sumlists); i++ {
		sumstr += sumlists[i]
	}
	s := EncodeToString(sha512.Sum512([]byte(sumstr)))
	//fmt.Println(s, "llm")
	return s, nil
}
func FindRequires(dir string) ([]string, error) {
	var require = NewSet()
	//系统包不参与依赖统计
	systempkglen := len(systempkg)
	err := filepath.Walk(dir, func(path string, info os.FileInfo, err error) error {

		if info.IsDir() {
			//跳过文件夹
			return nil
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
		//fmt.Println(path)
		//读取文件
		f, err := os.Open(path)
		if err != nil {
			return err
		}
		//读取每一行
		r := bufio.NewReader(f)

		for {
			line, err := r.ReadString('\n')
			if err != nil && err != io.EOF {
				//遇到未知错误
				return err
			}
			if err == io.EOF {
				break
			}
			line = strings.TrimSpace(line)
			linelen := len(line)

			if linelen <= 6 {
				//跳过无效短行
				continue
			}
			if line[0:1] == "#" {
				//跳过注释
				continue
			}
			if line[0:6] != "import" {
				//遇到非import直接退出
				break
			}
			//fmt.Println(line)
			//import
			//var lastbyte uint8
			lastbyte := ' '
			var modname []byte
			loadmodname := false
			for i := 6; i < linelen; i++ {
				//当上一个元素是空格，这个元素不是空格的时候跳入

				if !loadmodname {
					if lastbyte == ' ' && line[i] != ' ' {
						loadmodname = true
						lastbyte = int32(line[i])
						modname = append(modname, line[i])
						continue
					}
				}
				//当上一个元素不是空格，这个元素是空格的时候跳出
				if lastbyte != ' ' && line[i] == ' ' {
					break
				}
				modname = append(modname, line[i])
				lastbyte = int32(line[i])
			}
			//检查是否是系统模块
			modnamestr := string(modname[1:])
			for i := 0; i < systempkglen; i++ {
				//fmt.Println(systempkg[i], modnamestr)
				if systempkg[i] == modnamestr {
					return nil
				}
			}
			require.SAdd(modnamestr)
		}

		return nil
	})
	if err != nil {
		return nil, err
	}
	return require.SMembers(), nil
}
func SaveFile(path string, bytes []byte) error {
	file, err := os.OpenFile(path, os.O_RDWR|os.O_CREATE|os.O_TRUNC, 0777)
	defer func(file *os.File) {
		err = file.Close()
		if err != nil {

		}
	}(file)
	if err != nil {
		return err
	}
	writer := bufio.NewWriter(file)
	_, err = writer.Write(bytes)
	if err != nil {
		return err
	}
	err = writer.Flush()
	if err != nil {
		return err
	}
	return nil
}
