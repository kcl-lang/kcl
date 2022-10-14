package response

const (
	StdOkResp        = `{"code":0,"msg":"ok"}`
	StdErrResp       = `{"code":1,"msg":"err"}`
	StdArgsWrongResp = `{"code":2,"msg":"ArgsWrong"}`
)

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

type StdResp struct {
	Code int
	Msg  string
	Data interface{}
}
