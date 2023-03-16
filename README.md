# OCPP-J 1.6 websocket server
![Logo de Github](https://github.com/nicoLedesma/pf_hackathon_ocpp_server_2023/blob/66430895beee9913cb286ba041b0ccaf33ebf59e/logo.png)

A Rust server for receiving websocket connections from OCPP EVSEs.

Generated with the help of ChatGPT 3.5 and 4.

Search the commit messages for `[chatgpt]`  to see where it helped us out.

## Testing

1. Enter a secret and secure passwords in `./password.txt` and `./identity_password.txt`.
1. Generate a self signed certificate based on these secret passwords.
1. Compile and run the code

```
make self-signed-cert.p12-generation
cargo run
# Alternate: cargo run --release
```

Finally, connect a websocket client that is aware of our self-generated certificate.

```
cargo install websocat
SSL_CERT_FILE=cert.pem websocat wss://127.0.0.1:8765
```
