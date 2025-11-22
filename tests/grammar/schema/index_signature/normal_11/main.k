schema User:
    id: str

schema Users:
    [str]: User

schema DB:
    users: Users = {}

    check:
        all user in users {
            user == users[user].id
        }

schema DBs:
    [str]: DB

dbs_user: DBs = {
    user = DB {
        users: {
            app = User {id = "app"}
        }
    }
}
db_user = dbs_user.user
