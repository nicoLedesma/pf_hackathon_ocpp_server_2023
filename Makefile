##############################################################################
#Generate self-signed certificate for TLS
##############################################################################
self-signed-cert.p12-generation:
	@echo "Generating self-signed certificate for TLS"
	openssl genrsa -des3 -out key.pem 2048
	openssl req -new -key key.pem -out cert.csr -subj "/CN=localhost"
	openssl x509 -req -days 365 -in cert.csr -signkey key.pem -out cert.pem
	openssl pkcs12 -export -in cert.pem -inkey key.pem -out cert.p12 -name "Certificate Name"
	@echo "Self-signed certificate generated successfully"

