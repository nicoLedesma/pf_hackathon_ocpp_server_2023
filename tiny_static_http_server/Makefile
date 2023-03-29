run:
	docker build -t tiny_static_http_server .
	docker run --init -p 8080:8080 -t tiny_static_http_server

clean:
	# Do not remove the .gitignore or .gitkeep
	rm -r .well-known/acme-challenge/*
	touch .well-known/acme-challenge/.gitkeep
