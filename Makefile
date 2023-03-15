##############################################################################
#Generate self-signed certificate for TLS
##############################################################################
self-signed-cert.p12-generation:
	@echo "Generating self-signed certificate for TLS"
	openssl genrsa -des3 -out key.pem 2048
	openssl req -new -key key.pem -out cert.csr -subj "/CN=localhost"
	openssl pkcs12 -export -in cert.pem -inkey key.pem -out cert.p12 -name "Certificate Name"
	# Generate self-signed certificate
	# Certificate files can be made public
	#
	# Note that including email addresses in a certificate is considered deprecated and not recommended by some security standards, as email addresses can be changed more frequently than other identifying information such as domain names. 
	openssl x509 -passin file:"${PASSWORD_FILE}" -req -in cert.csr -signkey key.pem -out cert.pem \
		-days 365 \
		-subj "/O=PowerFlex/OU=Hackathon 2023"
	@echo "Self-signed certificate generated successfully"

