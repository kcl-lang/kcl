package application

import (
	"github.com/atreugo/session"
	"github.com/atreugo/session/providers/memory"
	"github.com/atreugo/session/providers/mysql"
	"github.com/atreugo/session/providers/postgre"
	"github.com/atreugo/session/providers/redis"
	"os"
	"strconv"
	"time"
)

var atreugoSession *session.Session

func GetAtreugoSession() *session.Session {
	if atreugoSession != nil {
		return atreugoSession
	}
	return createAtreugoSession()
}

type sqlConfig struct {
	Host      string
	Port      string
	UserName  string
	Password  string
	DbName    string
	TableName string
}

func createAtreugoSession() *session.Session {
	var provider session.Provider
	var err error
	var sc sqlConfig
	sc.Host = "127.0.0.1"
	if v := os.Getenv("ATREUGO_SESSION_HOST"); v != "" {
		sc.Host = v
	}
	if v := os.Getenv("ATREUGO_SESSION_PORT"); v != "" {
		sc.Port = v
	}
	if v := os.Getenv("ATREUGO_SESSION_USERNAME"); v != "" {
		sc.UserName = v
	}
	if v := os.Getenv("ATREUGO_SESSION_PASSWORD"); v != "" {
		sc.Password = v
	}
	if v := os.Getenv("ATREUGO_SESSION_DBNAME"); v != "" {
		sc.DbName = v
	}
	if v := os.Getenv("ATREUGO_SESSION_TABLENAME"); v != "" {
		sc.TableName = v
	}
	encoder := session.MSGPEncode
	decoder := session.MSGPDecode
	var defaultProvider = os.Getenv("ATREUGO_SESSION_PROVIDER")
	switch defaultProvider {
	case "memory":
		provider, err = memory.New(memory.Config{})
	case "redis":
		if sc.Port == "" {
			sc.Port = "6379"
		}
		provider, err = redis.New(redis.Config{
			KeyPrefix:   sc.TableName,
			Addr:        sc.Host + ":" + sc.Port,
			Username:    sc.UserName,
			Password:    sc.Password,
			PoolSize:    8,
			IdleTimeout: 30 * time.Second,
		})
	//case "memcache":
	//	provider, err = memcache.New(memcache.Config{
	//		KeyPrefix: "session",
	//		ServerList: []string{
	//			"0.0.0.0:11211",
	//		},
	//		MaxIdleConns: 8,
	//	})
	//case "sqlite3":
	//	cfg := sqlite3.NewConfigWith("test.db", "session")
	//	provider, err = sqlite3.New(cfg)
	case "mysql":
		encoder = session.Base64Encode
		decoder = session.Base64Decode
		port, err2 := strconv.ParseInt(sc.Port, 10, 64)
		if err2 != nil || (port < 0 || port > 65535) {
			port = 3306
		}
		cfg := mysql.NewConfigWith(sc.Host, 3306, sc.UserName, sc.Password, sc.DbName, sc.TableName)
		provider, err = mysql.New(cfg)
	case "postgre":
		encoder = session.Base64Encode
		decoder = session.Base64Decode
		port, err2 := strconv.ParseInt(sc.Port, 10, 64)
		if err2 != nil || (port < 0 || port > 65535) {
			port = 5432
		}
		cfg := postgre.NewConfigWith(sc.Host, port, sc.UserName, sc.Password, sc.DbName, sc.TableName)
		provider, err = postgre.New(cfg)
	default:
		provider, err = memory.New(memory.Config{})
	}

	if err != nil {
		panic(err)
	}

	cfg := session.NewDefaultConfig()
	cfg.EncodeFunc = encoder
	cfg.DecodeFunc = decoder
	cfg.SessionIDGeneratorFunc = RandBytes32
	as := session.New(cfg)

	if err = as.SetProvider(provider); err != nil {
		panic(err)
	}

	return as
}
