package application

var ds DataSource

func NewService(source DataSource) {
	if source != nil {
		ds = source
		return
	}
	panic("DataSource is nil,panic")
}
func GetService() DataSource {
	return ds
}
