schema Nginx:
    """Schema for Nginx configuration files"""
    http: Http

schema Http:
    server: Server

schema Server:
    listen: int | str    # The attribute `listen` can be int type or a string type.
    location?: Location  # Optional, but must be non-empty when specified

schema Location:
    root: str
    index: str

Nginx {
    http.server = {
        listen = 80
    }
}
Nginx {
    http.server = {
        listen = 8080
        location = {
            root = "/var/www/html"
            index = "index.html"
        }
    }
}
