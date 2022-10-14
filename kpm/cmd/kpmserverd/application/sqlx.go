package application

import (
	_ "github.com/go-sql-driver/mysql"
	"github.com/jmoiron/sqlx"
	"os"
)

var sqlxClient *sqlx.DB

func GetSqlxClient() *sqlx.DB {

	if sqlxClient != nil {
		return sqlxClient
	}
	return createSqlxClient()
}

func createSqlxClient() *sqlx.DB {
	var sc sqlConfig
	sc.Host = "127.0.0.1"
	sc.Port = "3306"
	sc.UserName = "root"
	if v := os.Getenv("SQLX_HOST"); v != "" {
		sc.Host = v
	}
	if v := os.Getenv("SQLX_PORT"); v != "" {
		sc.Port = v
	}
	if v := os.Getenv("SQLX_USERNAME"); v != "" {
		sc.UserName = v
	}
	if v := os.Getenv("SQLX_PASSWORD"); v != "" {
		sc.Password = v
	}
	if v := os.Getenv("SQLX_DBNAME"); v != "" {
		sc.DbName = v
	}
	if v := os.Getenv("SQLX_TABLENAME"); v != "" {
		sc.TableName = v
	}
	//GetLogger().Debug().Msg(sc.UserName + ":" + sc.Password + "@tcp(" + sc.Host + ":" + sc.Port + ")/" + sc.DbName + "?charset=utf8mb4&parseTime=true&loc=Local")
	//db, err := sqlx.Connect("mysql", "root:123456@tcp(127.0.0.1:3306)/test?charset=utf8mb4&parseTime=true&loc=Local")
	db, err := sqlx.Connect("mysql", sc.UserName+":"+sc.Password+"@tcp("+sc.Host+":"+sc.Port+")/"+sc.DbName+"?charset=utf8mb4&parseTime=true&loc=Local")
	if err != nil {
		panic(err)
		return nil
	}
	return db
}
