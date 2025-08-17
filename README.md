# my-http-server

- auto Markdown to HTML
- use actix_web for server

## use

1. put Markdown file to `./public`
2. put css(ect.) to `./_public`
3. run my-http-server

## env

`REQUEST_LOGGER`: default is `%{url}xi "%r" %s "%{Referer}i" "%{User-Agent}i"`, more info `actix_web::middleware::logger::Logger`
