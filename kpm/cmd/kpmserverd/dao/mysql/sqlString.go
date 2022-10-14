package mysql

const (
	//dbname         = "create DATABASE kpm;"

	//
	package_schema = `CREATE TABLE IF NOT EXISTS package(
                                   id BIGINT UNSIGNED AUTO_INCREMENT comment 'id',
                                   package_name VARCHAR(64) NOT NULL  comment '包名',
                                   package_admin VARCHAR(64) NOT NULL default '' comment '包管理者',
                                   package_description VARCHAR(256) NOT NULL default '' comment '简介',
                                   PRIMARY KEY ( id ),
                                  UNIQUE index_package_name(package_name) comment '唯一名字索引'
)ENGINE=RocksDB DEFAULT CHARSET=utf8mb4 collate = utf8mb4_bin;`

	version_schema = `CREATE TABLE IF NOT EXISTS version(
                                   id BIGINT UNSIGNED AUTO_INCREMENT comment 'id',
                                   package_version_unique_key VARCHAR(128) NOT NULL comment '唯一名',
                                   package_name VARCHAR(64) NOT NULL comment '包名',
                                   major INT UNSIGNED NOT NULL default 0 comment '主版本号',
                                   minor INT UNSIGNED NOT NULL default 0 comment '次版本号',
                                   patch INT UNSIGNED NOT NULL default 0 comment '修订号',
                                   pre_release_tag enum('alpha','beta','rc','release') default 'release' NOT NULL comment '先行版本',
                                   pre_release_tag_version INT UNSIGNED NOT NULL default 0 comment  '先行版本号',
                                   PRIMARY KEY ( id  ) comment  'id',
                                  UNIQUE index_package_id ( package_version_unique_key)  comment '唯一版本索引'
)ENGINE=RocksDB DEFAULT CHARSET=utf8mb4 collate = utf8mb4_bin;`
	subpkg_schema = `CREATE TABLE IF NOT EXISTS version(
                                   id BIGINT UNSIGNED AUTO_INCREMENT comment 'id',
                                   package_version_diff_unique_key VARCHAR(128) NOT NULL comment '唯一名',
                                   package_name VARCHAR(64) NOT NULL comment '包名',
                                   sub_pkg VARCHAR(64) NOT NULL comment '子包名',
                                   diff boolean NOT NULL comment '变化状态',
                                   PRIMARY KEY ( id  ) comment  'id',
                                  UNIQUE index_package_id ( package_version_diff_unique_key)  comment '唯一版本索引'
)ENGINE=RocksDB DEFAULT CHARSET=utf8mb4 collate = utf8mb4_bin;`
	searchpkg = `select package_name from kpm.package where package_name like  CONCAT('%',?,'%');`
)
