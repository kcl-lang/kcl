package application

type DataSource interface {
	SearchName(name string) string
	SearchSubPkgName(SubPkgName string) string
	SearchSubPkgNames(SubPkgNames []string) string
	Publish(pkgtgz []byte, compress string, kpmroot string, kpmserver string, kpmserverpath string) string
}
