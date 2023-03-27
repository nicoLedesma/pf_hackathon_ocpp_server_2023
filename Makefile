WSS_PORT:=5678
WSS_PORT2:=5679
PASSWORD_FILE:=password.txt
IDENTITY_PASSWORD_FILE:=identity_password.txt
LETSENCRYPT_DIR:=./letsencrypt_certs_private
# Examples: ocpp.powerflex.io ocpp.google.com ocpp.com ocpp.ocpp
# A domain that YOU own
# DOMAIN_NAME
LETSENCRYPT_CERTIFICATE_PEM:=${LETSENCRYPT_DIR}/config/live/${DOMAIN_NAME}/fullchain.pem
LETSENCRYPT_PRIVATE_SECRET_KEY_PEM:=${LETSENCRYPT_DIR}/config/live/${DOMAIN_NAME}/privkey.pem
LETSENCRYPT_IDENTITY_PKCS12_DER:=letsencrypt_identity.pkcs12.der
SELFSIGNED_IDENTITY_PKCS12_DER:=selfsigned_identity.pkcs12.der
SELFSIGNED_TLS_CERTIFICATE_PEM=./self_signed_certs/cert.pem
SELFSIGNED_TLS_PRIVATE_KEY_PEM=./self_signed_certs/private_key.pem
TLS_CERTIFICATE_PEM=./cert.pem
# DO NOT PUBLISH THE PRIVATE KEY AND KEEP PRIVATE+ENCRYPTED
TLS_PRIVATE_KEY_PEM=./private_key.pem

commitready:
	cargo fmt
	cargo check  # Faster than a rebuild
	cargo clippy
	make security-scan
	cargo test
	cargo test --release

find-largest-functions:
	# cargo install cargo-bloat
	# Reduce binary size more? https://github.com/TimonPost/cargo-unused-features
	cargo bloat --release -n 10

security-scan:
	# cargo install cargo-outdated cargo-audit
	cargo audit
	cargo outdated --root-deps-only

has-long-password:
	@if [ "$(shell wc -c "${IDENTITY_PASSWORD_FILE}" | cut -f1 -d" ")" -lt 64 ]; then \
	echo "Please make sure ${IDENTITY_PASSWORD_FILE} has a very long password on one line"; \
	exit 1; \
	fi

letsencrypt-domain-is-set:
	@if [ -z "${DOMAIN_NAME}" ]; then \
		echo Please set the DOMAIN_NAME variable ; \
		echo "Example: DOMAIN_NAME=ocpp.com make run" ; \
		exit 1 ; \
		fi

docker-build-dev:
	docker build -t ocpp_server_dev . -f Dockerfile.dev

copy-letsencrypt-pem: letsencrypt-domain-is-set
	cp ${LETSENCRYPT_CERTIFICATE_PEM} ${TLS_CERTIFICATE_PEM}
	cp ${LETSENCRYPT_PRIVATE_SECRET_KEY_PEM} ${TLS_PRIVATE_KEY_PEM}

copy-selfsigned-pem:
	cp ${SELFSIGNED_TLS_CERTIFICATE_PEM} ${TLS_CERTIFICATE_PEM}
	cp ${SELFSIGNED_TLS_PRIVATE_KEY_PEM} ${TLS_PRIVATE_KEY_PEM}

generate-empty-pem:
	# Useful for unit tests
	touch ${TLS_CERTIFICATE_PEM}
	touch ${TLS_PRIVATE_KEY_PEM}

docker-run-dev: copy-letsencrypt-pem docker-build-dev
	@echo Do not print or store contents of TLS_PRIVATE_KEY_PEM unsecured
	docker run \
		-it \
		--init \
		-e RUST_BACKTRACE="${RUST_BACKTRACE}" \
		-v "${TLS_CERTIFICATE_PEM}":/home/nonroot/certificate.pem \
		-v "${TLS_PRIVATE_KEY_PEM}":/home/nonroot/private_key.pem \
		-p ${WSS_PORT}:${WSS_PORT} \
		-p ${WSS_PORT2}:${WSS_PORT2} \
		-p 8765:8765 \
		-t ocpp_server_dev

print-cert-contents:
	openssl x509 -in ${TLS_CERTIFICATE_PEM} -noout -text

print-identity-contents:
	openssl pkcs12 -in identity.p12.der -info -nodes -passin file:identity_password.txt

print-letsencrypt-cert-contents: letsencrypt-domain-is-set
	openssl x509 -in ${LETSENCRYPT_CERTIFICATE_PEM} -noout -text

validate-server-tls:
	# Verify the server supports TLS v1.3 and that it has the right certificate chain
	openssl s_client -connect ${DOMAIN_NAME}:${WSS_PORT} -tls1_3 -showcerts

run_tiny_static_http_server:
	cd tiny_static_http_server ; make run

letsencrypt-certificate-generation:
	@echo "Suggestion: use the included ./static_http_server to serve the letsencrypt files to verify ownership of the ${LETSENCRYPT_DOMAIN}. Simply copy the necessary files into ./static_http_server and run it."
	certbot certonly --manual \
		--work-dir ${LETSENCRYPT_DIR}/work --config-dir ${LETSENCRYPT_DIR}/config --logs-dir ./letsencrypt/logs

letsencrypt-identity-file: letsencrypt-domain-is-set
	openssl pkcs12 -passout file:"${IDENTITY_PASSWORD_FILE}" -export -in ${LETSENCRYPT_CERTIFICATE_PEM} -inkey ${LETSENCRYPT_PRIVATE_SECRET_KEY_PEM}  -out ${LETSENCRYPT_IDENTITY_PKCS12_DER}  -name "Letsencrypt ${DOMAIN_NAME}"
	@echo "Letsencrypt-signed certificate generated successfully"
		

##############################################################################
#Generate self-signed certificate for TLS
##############################################################################

self-signed-certificate-generation:
	@echo "Generating self-signed certificate for TLS"
	mkdir -p ./self_signed_certs/
	# Generate private-key
	# DO NOT PUBLISH AND KEEP PRIVATE+ENCRYPTED
	openssl genrsa -des3 -passin file:"${PASSWORD_FILE}" -out ${SELFSIGNED_TLS_PRIVATE_KEY_PEM} 2048

	# Certificate signing request file CSR
	# CSR files can be made public
	#
	# ChatGPT: Yes, the x509 CN (Common Name) field is deprecated for identification purposes in SSL/TLS certificates. This is because the CN field was designed to be a flexible field that could be used for different types of identification, such as domain names, email addresses, or even person names. However, over time it became clear that this flexibility was leading to confusion and misinterpretation of the field, which in turn was leading to security vulnerabilities.
	#
	# The CN field is still used in some contexts, such as for identifying users in X.509 certificates used for digital signatures or other non-SSL/TLS purposes. However, for SSL/TLS certificates, the recommended practice is to use the Subject Alternative Name (SAN) extension instead of the CN field for identification purposes.
	#
	# The SAN extension allows for more flexible and standardized identification of SSL/TLS certificate subjects, including support for domain names, IP addresses, and email addresses. By using the SAN extension, SSL/TLS certificates can be more easily interpreted and validated, which can improve security and reduce the risk of misidentification or attacks.
	openssl req -new -passin file:"${PASSWORD_FILE}" -key ${SELFSIGNED_TLS_PRIVATE_KEY_PEM} \
		-subj "/CN=localhost/O=PowerFlex/OU=Hackathon 2023" \
		-addext "subjectAltName = DNS:Tezcatlipoca-T580, DNS:localhost, IP:127.0.0.1, IP:192.168.1.127" \
		-out ./self_signed_certs/cert.csr

	# Generate self-signed certificate
	# Certificate files can be made public
	#
	# ChatGPT: Note that including email addresses in a certificate is considered deprecated and not recommended by some security standards, as email addresses can be changed more frequently than other identifying information such as domain names.
	openssl x509 -req -passin file:"${PASSWORD_FILE}" -signkey ${SELFSIGNED_TLS_PRIVATE_KEY_PEM} -in ./cert.csr \
		-days 365 \
		-copy_extensions copy \
		-out ${SELFSIGNED_TLS_CERTIFICATE_PEM}

self-signed-identity-file:
	# Identity file containing private key and certificate
	@echo DO NOT PUBLISH ${IDENTITY_PASSWORD_FILE} AND KEEP PRIVATE AND ENCRYPTED
	@echo DO NOT PUBLISH ${SELFSIGNED_IDENTITY_PKCS12_DER} AND KEEP PRIVATE AND ENCRYPTED
	openssl pkcs12 -passout file:"${IDENTITY_PASSWORD_FILE}" -passin file:"${PASSWORD_FILE}" -export -in ${SELFSIGNED_TLS_CERTIFICATE_PEM} -inkey ${SELFSIGNED_TLS_PRIVATE_KEY_PEM} -out "${SELFSIGNED_IDENTITY_PKCS12_DER}" -name "ocpp_server Self-Signed Certificate Identity"
	@echo "Self-signed certificate generated successfully"

clean:
	rm *.pem *.csr *.der self_signed_certs/*

clean-all: clean
	rm -r ./target/
