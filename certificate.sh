openssl req -newkey rsa:2048 -nodes -keyout ca.key -x509 -days 365 -out ca.crt -subj "/C=US/ST=California/L=San Francisco/O=YourOrg/OU=YourOrgUnit/CN=YourOrg CA" -extensions v3_ca -config <(printf "[ req ]
distinguished_name=req_distinguished_name
x509_extensions = v3_ca

[ req_distinguished_name ]
countryName=Country Name (2 letter code)
stateOrProvinceName=State or Province Name (full name)
localityName=Locality Name (eg, city)
organizationName=Organization Name (eg, company)
organizationalUnitName=Organizational Unit Name (eg, section)
commonName=Common Name (eg, fully qualified host name)
emailAddress=Email Address

[ v3_ca ]
basicConstraints = critical, CA:TRUE
keyUsage = keyCertSign, cRLSign
subjectKeyIdentifier=hash
authorityKeyIdentifier=keyid:always,issuer
")
openssl genrsa -out server.key 2048
openssl genrsa -out client.key 2048
openssl req -new -key server.key -out server.csr -subj "/C=US/ST=California/L=San Francisco/O=YourOrg/OU=YourOrgUnit/CN=server"
openssl req -new -key client.key -out client.csr -subj "/C=US/ST=California/L=San Francisco/O=YourOrg/OU=YourOrgUnit/CN=client"
openssl x509 -req -in server.csr -CA ca.crt -CAkey ca.key -CAcreateserial -out server.crt -days 365 -extensions v3_req -extfile <(printf "[ v3_req ]
basicConstraints = CA:FALSE
keyUsage = nonRepudiation, digitalSignature, keyEncipherment
subjectAltName = @alt_names

[alt_names]
DNS.1 = localhost
IP.1 = 127.0.0.1
")
openssl x509 -req -in client.csr -CA ca.crt -CAkey ca.key -CAcreateserial -out client.crt -days 365 -extensions v3_req -extfile <(printf "[ v3_req ]
basicConstraints = CA:FALSE
keyUsage = nonRepudiation, digitalSignature, keyEncipherment
")
