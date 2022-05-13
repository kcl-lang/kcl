// Copyright 2021 The KCL Authors. All rights reserved.

//go:build ingore
// +build ingore

package main

import (
	"archive/zip"
	"crypto/tls"
	"flag"
	"fmt"
	"io"
	"log"
	"net/http"
	"os"
	"strings"
)

const (
	// https://www.python.org/ftp/python/3.9.6/python-3.9.6-embed-amd64.zip
	// https://npm.taobao.org/mirrors/python/3.9.6/python-3.9.6-embed-amd64.zip

	baseUrl        = "https://www.python.org/ftp/python"
	baseUrl_taobao = "https://npm.taobao.org/mirrors/python"
)

var (
	flagDownloadUrl = flag.String("url", baseUrl_taobao+"/3.9.6/python-3.9.6-embed-amd64.zip", "set python-x.y.z-embed-amd64.zip")
	flagOutputFile  = flag.String("output", "python-3.9.6-embed-amd64.zip", "set output file")
)

func main() {
	flag.Parse()

	if s := *flagOutputFile; fileExists(s) {
		fmt.Printf("File %s exists\n", s)
		return
	}

	var err error
	if err = DownloadFile(*flagDownloadUrl, *flagOutputFile); err != nil {
		log.Fatal(err)
	}
	fmt.Printf("Download %s ok\n", *flagDownloadUrl)
}

func DownloadFile(url, filename string) (errRet error) {
	f, err := os.Create(filename)
	if err != nil {
		return fmt.Errorf("failed to create %s: %v", filename, err)
	}
	defer f.Close()
	defer func() {
		if errRet != nil {
			os.Remove(filename)
		}
	}()

	tr := &http.Transport{
		TLSClientConfig: &tls.Config{InsecureSkipVerify: true},
	}
	client := &http.Client{Transport: tr}

	resp, err := client.Get(url)
	if err != nil {
		return fmt.Errorf("failed to download %s: %v", url, err)
	}
	defer resp.Body.Close()

	_, err = io.Copy(f, resp.Body)
	if err != nil {
		return fmt.Errorf("failed to write %s: %v", filename, err)
	}
	return nil
}

func fileExists(name string) bool {
	if strings.HasSuffix(name, "zip") {
		archive, err := zip.OpenReader(name)
		if err != nil {
			return false
		}
		defer archive.Close()
		return true
	} else {
		f, err := os.Open(name)
		if err != nil {
			return false
		}
		defer f.Close()
		return true
	}
}
