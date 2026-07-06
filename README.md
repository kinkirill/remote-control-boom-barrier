# Remote control boom barrier

A pet project that lets multiple people open a single boom barrier remotely. The barrier is controlled via an ESP-01S WiFi module with a 2190DH relay that presses the remote button by using relay to short the circuit, that comes with the ESP-01S module. An MQTT broker relays commands, and a Python script monitors Bitrix24 chats for trigger messages.

This repository was made so people with the same or familiar problem could use it to their advantage

## Architecture

```
Bitrix24 Chat  ->  Python Monitor -> Rust Backend (TCP :19090)
                                              |
                                          MQTT (1883)
                                              |
                                ESP-01S (ESP8266) with a relay
                                              |
                                      2190DH - the remote
                                              |
                                         Boom Barrier
```

## Components

### 1. Rust backend (`src/main.rs`)

Rust was used here as a backend, because I didn't want something to break due to being a wrong type or behaviour (Hello Python), so the most crucial part was made in a low-level and consistent language

- Starts an **MQTT broker** (via `rumqttd`) on port `1883`
- Listens for TCP commands on port `19090`
- Publishes `OPEN` to MQTT topic `gate/control` when a client sends `OPEN` via TCP
- Subscribes to `gate/status` for status feedback

```bash
cargo run --release
# Send command(LINUX):
echo OPEN | nc 127.0.0.1 19090
# Send command(WINDOWS):
$t = New-Object System.Net.Sockets.TcpClient('127.0.0.1', 19090); $s = $t.GetStream(); $w = New-Object System.IO.StreamWriter($s); $w.WriteLine('OPEN'); $w.Flush(); $t.Close()
```

### 2. ESP-01S firmware (`esp-01s.cpp`)

Arduino sketch for the **ESP8266** (ESP-01S) with a 2190DH relay module.

- Connects to WiFi and MQTT broker
- Subscribes to `gate/control`
- On `OPEN` message: sends relay hex sequence over serial to momentarily press the remote button
- Publishes status feedback to `gate/status`
- Relay connected in parallel with the boom barrier's existing remote button

Flash using Arduino IDE with `PubSubClient` and `ESP8266WiFi` libraries.

### 3. Python script (`python/main.py`)

Polls Bitrix24 chats via REST webhook and triggers the gate when a new `#` message appears.

```bash
cd python
pip install -r requirements.txt
cp .env.example .env
cp chats.py.example chats.py
python -m python.main
```

---

**NOTE:** This configuration can and must be different. Bitrix24 was used here because it helped
to limit amount of people who are allowed to interact with the system simply becuase only my collegues
exist in Bitrix24 platfrom.

#### Trigger flow while utilizing Bitrix24 platform

1. Authorized user sends `#` in their Bitrix24 chat
2. Python monitor polls `im.dialog.messages.get` every 1.5s (bots are nonexistent there)
3. On detecting a new `#` message, it publishes `OPEN` to Rust-backend MQTT topic `gate/control`
4. ESP-01S receives the message, activates relay for 500ms
5. Remote button is pressed -> boom barrier opens

#### Configuration

**`.env`**

| Variable          | Default     | Description               |
| ----------------- | ----------- | ------------------------- |
| `B24_WEBHOOK_URL` | -           | Bitrix24 REST webhook URL | OR WHATEVER YOU NEED |
| `BACKEND_HOST`    | `127.0.0.1` | Rust backend address      |
| `BACKEND_PORT`    | `19090`     | Rust backend TCP port     |

**`chats.py`** - maps Bitrix24 dialog IDs to human names:

## Project structure

```
├── src/main.rs           # Rust MQTT broker + TCP command server
├── esp-01s.cpp           # ESP8266 Arduino firmware
├── python/
│   ├── main.py           # Bitrix24 chat monitor
│   ├── chats.py.example  # Example mapping
│   ├── .env.example      # Example config
│   └── requirements.txt  # Python dependencies
├── Cargo.toml
└── readme.md
```

**Letter of honesty:** Some parts of this project were made by using existing AI tools
