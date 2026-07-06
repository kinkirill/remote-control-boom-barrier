// ATTENTION: ARDUINO IDE IS REQUIRED TO EDIT THIS FILE
#include <ESP8266WiFi.h>
#include <PubSubClient.h>

// 1. Network settings
const char *ssid = "WI-FI-SSID";
const char *password = "very$trongP@$$WORD";
const char *mqtt_server = "10.10.1.1";

// This is the specific hex code to manipulate relay of the ESP-01S
const byte relay_on[] = {0xA0, 0x01, 0x01, 0xA2};
const byte relay_off[] = {0xA0, 0x01, 0x00, 0xA1};

WiFiClient espClient;
PubSubClient client(espClient);

void setup_wifi() {
  delay(10);
  WiFi.begin(ssid, password);
  while (WiFi.status() != WL_CONNECTED) {
    delay(500);
  }
}

// 3. MQTT Message Processor
void callback(char *topic, byte *payload, unsigned int length) {
  String message = "";
  for (unsigned int i = 0; i < length; i++) {
    message += (char)payload[i];
  }

  if (message == "OPEN") {
    // Fire the working open sequence over the serial bus
    Serial.write(relay_on, sizeof(relay_on));

    delay(500); // Hold the gate remote button down for 0.5 seconds

    // Fire the close sequence to release the remote button
    Serial.write(relay_off, sizeof(relay_off));

    // Notify your Rust server that the physical cycle executed
    client.publish("gate/status", "CLOSED_AUTOMATICALLY");
  }
}

void reconnect() {
  while (!client.connected()) {
    if (client.connect("ESP01S_Gate_Relay")) {
      delay(100); // Give the connection a moment to breathe
      client.subscribe("gate/control");
    } else {
      delay(5000);
    }
  }
}

void setup() {
  // Relay works only on 9600.
  // NOTE: There were test on 115200 baud, and they were unsuccessful
  Serial.begin(9600);

  setup_wifi();
  client.setServer(mqtt_server, 1883);
  client.setCallback(callback);
}

void loop() {
  if (!client.connected()) {
    reconnect();
  }
  client.loop();
}
