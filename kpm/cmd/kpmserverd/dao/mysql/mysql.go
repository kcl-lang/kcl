package mysql

import (
	"github.com/jmoiron/sqlx"
)

type Mysql struct {
	db *sqlx.DB
}

func NewMysql(db *sqlx.DB) Mysql {
	return Mysql{db: db}
}

type Package struct {
	id                 uint64
	PackageName        string
	PackageAdmin       string
	PackageDescription string
}

func (m Mysql) AddPkg(pkgname, admin string) error {
	//tx, err :=m.db.Prepare("")
	//if err != nil {
	//	return err
	//}
	//_, err := tx.Exec()
	//if err != nil {
	//	return err
	//}

	return nil
}
func (m Mysql) SearchPkg(pkgname string) ([]string, error) {
	tx, err := m.db.Prepare(searchpkg)
	if err != nil {
		return nil, err
	}
	r, err := tx.Query(pkgname)
	if err != nil {
		return nil, err
	}
	pkgs := make([]string, 10)
	pkgs = pkgs[:0]

	for r.Next() {
		var name string
		err = r.Scan(&name)
		if err != nil {

			return nil, err
		}
		pkgs = append(pkgs, name)
	}
	return pkgs, nil
}
