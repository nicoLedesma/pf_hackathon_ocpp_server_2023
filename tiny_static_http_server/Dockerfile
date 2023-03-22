FROM lipanski/docker-static-website:latest

COPY .well-known/ .well-known/

CMD ["echo", "Starting HTTP server. Good luck."]
CMD ["/busybox", "httpd", "-f", "-v", "-p", "8080", "-c", "httpd.conf"]
