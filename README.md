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
We also learned a lot about openssl and deprecated features that we should avoid.

On the other hand, we used https://github.com/CompVis/stable-diffusion Stable Diffusion on an NVIDIA GPU with 8GB of VRAM to generate images that we then chose as the project logo.
This model was run locally.

As alternatives to ChatGTP, we tried to use Facebook's LLAMA and GALACTICA but we found that they don't know much about Rust or OCPP.

By this commit: 2745663f14a1f985623ed99df29b905dba6ed4dc
we had managed to connect to our OCPP simulator.
We merely received its OCPP messages and did not parse or respond.
However, we did this over a secure TLS 1.3 socket.

Here is the output of this session:

```
19:18:51 pf_hackathon_ocpp_server_2023 v0.1.0 🦀 v1.68.0 main ! 3m18s $ cargo run
   Compiling pf_hackathon_ocpp_server_2023 v0.1.0 (/home/roguh/src/pf_hackathon_ocpp_server_2023)
    Finished dev [unoptimized + debuginfo] target(s) in 1.44s
     Running `target/debug/pf_hackathon_ocpp_server_2023`
Will listen on: ws://127.0.0.1:8765
Will listen on: wss://127.0.0.1:5678
Will listen on: wss://192.168.50.174:5678
Loaded TLS identity (cert and private key) for address 192.168.50.174:5678
Loaded TLS identity (cert and private key) for address 127.0.0.1:5678
Loaded cert with Subject alt names (SNA): [localhost, 127.0.0.1, 192.168.50.174]
Loaded cert with Subject alt names (SNA): [localhost, 127.0.0.1, 192.168.50.174]
Connection to 192.168.50.174:5678 received from 172.18.0.3:39196
Websocket message from 172.18.0.3:39196
Received text message: [2,"4b05fb33-6510-445e-b7c4-d3a6c611400e","BootNotification",{"chargePointModel":"TRI93-50-01","chargePointVendor":"Tritium","chargePointSerialNumber":"12336","firmwareVersion":"v2.3.2","iccid":"89014103270749598363","imsi":"310410074959836"}]
Connection to 192.168.50.174:5678 received from 172.18.0.3:37338
Websocket message from 172.18.0.3:37338
Received text message: [2,"0d768090-eab7-4bc0-b75f-a930121a4c5f","BootNotification",{"chargePointModel":"TRI93-50-01","chargePointVendor":"Tritium","chargePointSerialNumber":"HI_RUST","firmwareVersion":"v2.3.2","iccid":"89014103270749598363","imsi":"310410074959836"}]

Websocket message from 172.18.0.3:39196
Received PING message with length: 4
Websocket message from 172.18.0.3:37338
```

The simulator was started and configured with:

```
$ git log | head
commit 570a6671d694de3074b24bfde01bf2da74d9f561
Author: Hugo O. Rivera <hugo@roguh.com>
Date:   Wed Mar 15 19:16:41 2023 -0700

$ make up-integration
$ make setup-with-pipenv
$ pipenv run ./connect_to_central_system.py --central-system-url wss://192.168.50.174:5678 --chargepoint-id HI_RUST
/simulation {'message': 'Rejected', 'error': 'TimeoutError(\'Waited 30s for response on [2,"4431c63c-4123-4e59-aa08-cc78e43d688c","BootNotification",{"chargePointModel":"TRI93-50-01","chargePointVendor":"Tritium","chargePointSerialNumber":"HI_RUST","firmwareVersion":"v2.3.2","iccid":"89014103270749598363","imsi":"310410074959836"}].\')'}
Available
{'message': 'Accepted', 'configuration': {'AllowOfflineTxForUnknownId': {'readonly': False, 'value': False, 'type': 'bool', 'reboot': False}, 'AuthorizationCacheEnabled': {'readonly': False, 'value': False, 'type': 'bool', 'reboot': False}, 'AutoStartRemoteCharge': {'readonly': False, 'value': False, 'type': 'bool', 'reboot': False}, 'CentralURL': {'readonly': False, 'value': 'wss://192.168.50.174:5678', 'type': 'str', 'reboot': False}, 'ClockAlignedDataInterval': {'readonly': False, 'value': 900, 'type': 'int', 'unit': 'second', 'reboot': False}, 'GetConfigurationMaxKeys': {'readonly': True, 'value': 128, 'type': 'int', 'reboot': False}, 'HeartbeatInterval': {'readonly': False, 'value': 10, 'type': 'int', 'unit': 'second', 'reboot': False}, 'HU1.CCUMaxHalfEnable': {'readonly': False, 'value': False, 'type': 'bool', 'reboot': False}, 'HU1.CCURFIDDisable': {'readonly': False, 'value': False, 'type': 'bool', 'reboot': False}, 'HU1.CCUSOCLimitDisable': {'readonly': False, 'value': True, 'type': 'bool', 'reboot': False}, 'InitialConnectionWaitInterval': {'readonly': False, 'value': 10, 'type': 'int', 'unit': 'second', 'reboot': False}, 'MeterValueSampleInterval': {'readonly': False, 'value': 10, 'type': 'int', 'unit': 'second', 'reboot': False}, 'ModemCcid': {'readonly': True, 'value': '89014103270749598363', 'type': 'str', 'reboot': False}, 'ModemImsi': {'readonly': True, 'value': '310410074959836', 'type': 'str', 'reboot': False}, 'NumberOfConnectors': {'readonly': True, 'value': 2, 'type': 'int'}, 'ReconnectExpBackOffMax': {'readonly': False, 'value': 2, 'type': 'int', 'unit': 'second', 'reboot': False}, 'ReconnectInterval': {'readonly': False, 'value': 1, 'type': 'int', 'unit': 'second', 'reboot': False}, 'ReconnectRandomRange': {'readonly': False, 'value': 3, 'type': 'int', 'unit': 'second', 'reboot': False}, 'ReportCumulativeUsage': {'readonly': False, 'value': False, 'type': 'bool', 'reboot': False}, 'SerialNumber': {'readonly': True, 'value': '132324', 'type': 'str', 'reboot': False}, 'SoftwareVersion': {'readonly': True, 'value': 'v2.3.2', 'type': 'str', 'reboot': False}, 'SupportedFeatureProfiles': {'readonly': True, 'value': 'Core,FirmwareManagement,LocalAuthListManagement,RemoteTrigger,Reservation,SmartCharging', 'type': 'str', 'reboot': False}, 'TransactionMessageAttempts': {'readonly': False, 'value': 1, 'type': 'int', 'reboot': False}, 'TransactionMessageRetryInterval': {'readonly': False, 'value': 10, 'type': 'int', 'unit': 'second', 'reboot': False}, 'WebsocketPingsPendingLimit': {'readonly': False, 'value': 2, 'type': 'int', 'reboot': False}, 'Model': {'readonly': True, 'value': 'TRI93-50-01', 'type': 'str', 'reboot': False}, 'Vendor': {'readonly': True, 'value': 'Tritium', 'type': 'str', 'reboot': False}}}
{'message': 'Accepted', 'reason': 'HeartbeatInterval updated to 60'}
{'message': 'Accepted', 'reason': 'An ev has been parked'}
{'message': 'Accepted', 'reason': 'charge_interval was successfully updated to 0.5'}
{'message': 'Rejected', 'error': 'TimeoutError(\'Waited 30s for response on [2,"4e0fdd83-c7d8-4a53-b213-380a9b5d6169","StatusNotification",{"connectorId":1,"errorCode":"NoError","status":"Preparing","timestamp":"2023-03-16T02:24:20.452276"}].\')'}
{'message': 'Accepted', 'reason': 'Successful DCFC HI_RUST Connector 1 state change: plugged ev'}
```

Notice the simulator waited a long time to receive our responses.


Finally, we spent the last few hours trying to parse and respond to our OCPP simulator's BootNotification and StatusNotificationn messages.
This was successful, though it will require fixing numerous compilation errors.
From here, it is clear how to implement a full-fledged OCPP server or client.
