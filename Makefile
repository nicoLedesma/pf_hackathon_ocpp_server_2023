PASSWORD_FILE=password.txt
IDENTITY_PASSWORD_FILE=identity_password.txt
##############################################################################
#Generate self-signed certificate for TLS
##############################################################################
self-signed-cert.p12-generation:
	@echo "Generating self-signed certificate for TLS"
	# Generate private-key
	# DO NOT PUBLISH AND KEEP PRIVATE+ENCRYPTED
	openssl genrsa -passout file:"${PASSWORD_FILE}" -des3 -out key.pem 2048
	# Certificate signing request file CSR
	# CSR files can be made public
	openssl req -passin file:"${PASSWORD_FILE}" -new -key key.pem -out cert.csr -subj "/CN=localhost"
	# Generate self-signed certificate
	# Certificate files can be made public
	#
	# Note that including email addresses in a certificate is considered deprecated and not recommended by some security standards, as email addresses can be changed more frequently than other identifying information such as domain names. 
	openssl x509 -passin file:"${PASSWORD_FILE}" -req -in cert.csr -signkey key.pem -out cert.pem \
		-days 365 \
		-subj "/O=PowerFlex/OU=Hackathon 2023"
	# Identity file containing private key and certificate
	# DO NOT PUBLISH AND KEEP PRIVATE+ENCRYPTED
	openssl pkcs12 -passout file:"${IDENTITY_PASSWORD_FILE}" -passin file:"${PASSWORD_FILE}" -export -in cert.pem -inkey key.pem -out identity.p12.der -name "Hackathon Self-Signed Certificate Identity"
	@echo "Self-signed certificate generated successfully"

