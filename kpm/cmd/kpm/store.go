package main

import (
	"crypto/sha512"
	"io"
	"kpm/cmd/kpmserverd/application"
	"os"
	"path/filepath"
)

func StoreAddFile(fpath, kpmroot string, logflag bool) error {
	//检测是否是文件，
	//如果是文件，取模并复制一份到存储库
	f, err := os.Open(fpath)
	if err != nil {
		return err
	}
	fi, err := f.Stat()
	if err != nil {
		return err
	}
	if !fi.IsDir() {
		filebytes, err := io.ReadAll(f)
		if err != nil {
			return err
		}
		if logflag {
			print(fpath + "  -->  ")
		}

		hash := EncodeToString(sha512.Sum512(filebytes))
		t := kpmroot + Separator + "store" + Separator + "v1" + Separator + "files" + Separator + HashMod(application.S2B(hash)) + Separator + hash
		if logflag {
			println(t)
		}
		//检测文件是否存在，如果存在，则不动，如果不存在，则创建
		err = os.WriteFile(t, filebytes, 0777)
		if err != nil {
			return err
		}
	} else {
		err = filepath.Walk(fpath, func(path string, info os.FileInfo, err error) error {
			if info.IsDir() {
				//跳过文件夹
				return nil
			}
			f2, err := os.Open(path)
			if err != nil {
				return err
			}
			filebytes, err := io.ReadAll(f2)
			if err != nil {
				return err
			}
			if logflag {
				print(path + "  -->  ")
			}

			hash := EncodeToString(sha512.Sum512(filebytes))
			t := kpmroot + Separator + "store" + Separator + "v1" + Separator + "files" + Separator + HashMod(application.S2B(hash)) + Separator + hash
			if logflag {
				println(t)
			}
			//检测文件是否存在，如果存在，则不动，如果不存在，则创建
			err = os.WriteFile(t, filebytes, 0777)
			if err != nil {
				return err
			}
			return nil
		})
		if err != nil {
			return err
		}
	}
	return nil
}
