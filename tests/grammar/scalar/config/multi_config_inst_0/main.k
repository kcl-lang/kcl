{
    http.server = {
        listen = 80
    }
}
{
    http.server = {
        listen = 8080
        location = {
            root = "/var/www/html"
            index = "index.html"
        }
    }
}
