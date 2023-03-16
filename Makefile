PASSWORD_FILE=password.txt
IDENTITY_PASSWORD_FILE=identity_password.txt

print-cert-contents:
	openssl x509 -in cert.pem -noout -text

print-identity-contents:
	openssl pkcs12 -in identity.p12.der -info -nodes -passin file:identity_password.txt

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
	#
	# ChatGPT: Yes, the x509 CN (Common Name) field is deprecated for identification purposes in SSL/TLS certificates. This is because the CN field was designed to be a flexible field that could be used for different types of identification, such as domain names, email addresses, or even person names. However, over time it became clear that this flexibility was leading to confusion and misinterpretation of the field, which in turn was leading to security vulnerabilities.
	#
	# The CN field is still used in some contexts, such as for identifying users in X.509 certificates used for digital signatures or other non-SSL/TLS purposes. However, for SSL/TLS certificates, the recommended practice is to use the Subject Alternative Name (SAN) extension instead of the CN field for identification purposes.
	#
	# The SAN extension allows for more flexible and standardized identification of SSL/TLS certificate subjects, including support for domain names, IP addresses, and email addresses. By using the SAN extension, SSL/TLS certificates can be more easily interpreted and validated, which can improve security and reduce the risk of misidentification or attacks.
	openssl req -new -passin file:"${PASSWORD_FILE}" -key key.pem \
		-subj "/CN=localhost/O=PowerFlex/OU=Hackathon 2023" \
		-addext "subjectAltName = DNS:localhost, IP:127.0.0.1, IP:192.168.50.174" \
		-out cert.csr

	# Generate self-signed certificate
	# Certificate files can be made public
	#
	# ChatGPT: Note that including email addresses in a certificate is considered deprecated and not recommended by some security standards, as email addresses can be changed more frequently than other identifying information such as domain names.
	openssl x509 -req -passin file:"${PASSWORD_FILE}" -signkey key.pem -in cert.csr \
		-days 365 \
		-extensions san -copy_extensions copy \
		-out cert.pem

	# Identity file containing private key and certificate
	# DO NOT PUBLISH AND KEEP PRIVATE+ENCRYPTED
	openssl pkcs12 -passout file:"${IDENTITY_PASSWORD_FILE}" -passin file:"${PASSWORD_FILE}" -export -in cert.pem -inkey key.pem -out identity.p12.der -name "Hackathon Self-Signed Certificate Identity"
	@echo "Self-signed certificate generated successfully"

clean:
	rm *.pem *.csr *.der
