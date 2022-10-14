package main

import (
	"encoding/json"
	"errors"
	"os"
)

type KpmFileP struct {
	Path    string
	kpmfile *KpmFile
}

func NewKpmFileP(path string) (*KpmFileP, error) {
	exists, err := PathExists(path + Separator + "kpm.json")
	if err != nil {
		//异常
		return nil, err
	}
	if !exists {
		//不存在
		println("kpm.json doesn't exist")
		return nil, errors.New("kpm.json doesn't exist")
	}
	content, err := os.ReadFile(path + Separator + "kpm.json")
	if err != nil {
		return nil, err
	}
	kpmf := KpmFile{}
	err = json.Unmarshal(content, &kpmf)
	if err != nil {
		return nil, err
	}
	return &KpmFileP{
		Path:    path + Separator + "kpm.json",
		kpmfile: &kpmf,
	}, nil
}

// Save 保存到目标路径
func (k *KpmFileP) Save() error {
	marshal, err := json.Marshal(k.kpmfile)
	if err != nil {
		println(err.Error())
		return err
	}
	err = os.WriteFile(k.Path, marshal, 0777)
	if err != nil {
		return err
	}
	return nil
}

// Create 创建到目标路径
func (k *KpmFileP) Create() error {
	exists, err := PathExists(k.Path)
	if err != nil {
		//异常
		println("异常")
		return err
	}
	if exists {
		//存在
		println("kpm.json already exists")
		return errors.New("kpm.json already exists")
	}
	//创建

	err = k.Save()
	if err != nil {
		println("save fail,because", err.Error())
		return err
	}
	return nil
}
