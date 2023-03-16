# OCPP-J 1.6 websocket server
![Logo](https://github.com/nicoLedesma/pf_hackathon_ocpp_server_2023/blob/66430895beee9913cb286ba041b0ccaf33ebf59e/logo.png)

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

## Using AI

Having clear the objective we wanted to achieve, we used AI (ChatGPT) from the beginning of the project 
we used the AI (ChatGPT) from the beginning of the project with analytical criteria and analyzing the results of its responses. 
At the beginning we tried to ask for code suggestions to build a websocket server with TLS(Rust) security which was very interesting at first, but when we compiled the code directly as suggested by the AI, we found several errors. 
These bugs were mainly in the use of crates and the different functions that were supposed to be implemented and that we could use.
We then consulted it about the bugs, copying and pasting the output of the Rust compiler, but ChatGPT could not resolve them as the various suggestions generated new bugs.
Although it is a very powerful tool that paved the way for us, copying and pasting code was not enough.

On the other hand, when we worked with openssl to generate a self-signed certificate, Chatgpt was a valuable ally to query the various errors we encountered when we wanted to parse the certificate with our Rust code.

On the other hand, we used https://github.com/CompVis/stable-diffusion Stable Diffusion on an NVIDIA GPU with 8GB of VRAM to generate images that we then chose as the project logo.

As alternatives to ChatGTP, we tried to use Facebook's LLAMA and GALACTICA but we found that they don't know much about Rust or OCPP.
