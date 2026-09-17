# Sustainability metrics summary

- Firmware ELF (native-sim): **0.00 MB**
- Minimal WASM module: **33 B**
- Heartbeat round-trip wire size: **12 B**
- Steady-state control traffic: **1728 B/h/device** (~1.7 KiB/h)
- Enrollment round-trip (once): **87 B**
- Deploy round-trip (minimal WASM): **90 B**
- Gateway idle mean CPU: **4 m**; RAM: **18.2 MiB**

## Wire sizes (4-byte length prefix + CBOR)

| Message | Bytes |
|---------|------:|
| client_deploy_ack | 23 |
| client_enrollment_ack | 6 |
| client_enrollment_request | 6 |
| client_heartbeat | 6 |
| client_public_key | 40 |
| server_deploy_minimal_wasm | 67 |
| server_device_uuid | 23 |
| server_enrollment_accepted | 6 |
| server_enrollment_completed | 6 |
| server_heartbeat_ack | 6 |

## Gateway memory vs synthetic board registrations

| Boards | Gateway RAM (MiB) | CPU (m) |
|--------|------------------:|--------:|
| 1 | 19.0 | 2.0 |
| 10 | 19.0 | 2.0 |
| 25 | 19.0 | 2.0 |
| 50 | 19.0 | 2.0 |