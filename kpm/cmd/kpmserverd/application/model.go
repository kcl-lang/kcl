package application

type SearchPkg struct {
	//包名
	Name string
	//描述
	Description string
	//版本
	Version string
}
type SearchPkgs []SearchPkg
type SearchPkgsResp struct {
	Code int
	Msg  string
	Data SearchPkgs
}
