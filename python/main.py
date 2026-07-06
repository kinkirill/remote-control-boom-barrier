import os
import socket
import time

import dotenv
import requests

from .chats import GATE_CHATS

try:
    _ = dotenv.load_dotenv()
except ImportError:
    pass

B24_WEBHOOK_URL = os.getenv("B24_WEBHOOK_URL")
BACKEND_HOST = os.getenv("BACKEND_HOST", "127.0.0.1")
BACKEND_PORT = int(os.getenv("BACKEND_PORT", "19090"))

POLL_INTERVAL = 1.5


def collect_b24_message(dialog_id: str):
    method = "im.dialog.messages.get"
    payload = {
        "DIALOG_ID": dialog_id,
        "LAST_ID": 0,
        "LIMIT": 1,
    }
    response = requests.post(f"{B24_WEBHOOK_URL}{method}", json=payload)
    return response.json()


def send_open_command() -> bool:
    try:
        with socket.create_connection((BACKEND_HOST, BACKEND_PORT), timeout=5) as sock:
            sock.sendall(b"OPEN")
            response = sock.recv(1024)
            return b"OK" in response
    except Exception as e:
        print(f"[ERROR] send_open_command: {e}")
        return False


def get_last_message(dialog_id: str):
    data = collect_b24_message(dialog_id)
    messages = data.get("result", {}).get("messages", [])
    if not messages:
        return None
    return messages[0]


def main():
    last_seen: dict[str, tuple[int, str]] = {}

    print(f"Monitoring {len(GATE_CHATS)} chat(s), poll interval {POLL_INTERVAL}s")
    print(f"Backend: {BACKEND_HOST}:{BACKEND_PORT}")

    for dialog_id, name in GATE_CHATS.items():
        msg = get_last_message(dialog_id)
        if msg:
            last_seen[dialog_id] = (msg.get("id", 0), msg.get("text", "").strip())
            print(f"[{name}] #{dialog_id} — init skip msg#{msg['id']}")

    print("Monitoring started, reacting only to new messages\n")

    while True:
        for dialog_id, name in GATE_CHATS.items():
            msg = get_last_message(dialog_id)
            if not msg:
                continue

            msg_id = msg.get("id", 0)
            msg_text = msg.get("text", "").strip()

            prev = last_seen.get(dialog_id)
            if prev == (msg_id, msg_text):
                continue

            last_seen[dialog_id] = (msg_id, msg_text)

            if msg_text == "#":
                print(f"[{name} ID:{dialog_id}] — OPEN command")
                ok = send_open_command()
                if ok:
                    print(f"[{name}] OPEN sent successfully")
                else:
                    print(f"[{name}] OPEN failed")

        time.sleep(POLL_INTERVAL)


if __name__ == "__main__":
    try:
        main()
    except KeyboardInterrupt:
        print("\nStopped")
