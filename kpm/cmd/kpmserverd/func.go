package main

import "os"

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
