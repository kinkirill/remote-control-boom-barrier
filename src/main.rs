use rumqttc::{Client, Event, MqttOptions, Packet, QoS};
use std::collections::HashMap;
use std::error::Error;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener};
use std::str::FromStr;
use std::time::Duration;

fn broker_config() -> rumqttd::Config {
    let mut config = rumqttd::Config::default();
    let mut v4 = HashMap::new();
    v4.insert(
        "default".to_string(),
        rumqttd::ServerSettings {
            name: "default".to_string(),
            listen: SocketAddr::from_str("0.0.0.0:1883").unwrap(),
            tls: None,
            next_connection_delay_ms: 50,
            connections: rumqttd::ConnectionSettings {
                connection_timeout_ms: 5000,
                max_payload_size: 2048,
                max_inflight_count: 100,
                auth: None,
                external_auth: None,
                dynamic_filters: false,
            },
        },
    );
    config.router.max_connections = 100;
    config.router.max_segment_size = 10240;
    config.router.max_segment_count = 100;
    config.router.max_outgoing_packet_count = 100;
    config.v4 = Some(v4);
    config
}

fn main() -> Result<(), Box<dyn Error>> {
    eprintln!("=== Roadway Control Backend ===");
    eprintln!("Starting MQTT broker on 0.0.0.0:1883...");

    let mut broker = rumqttd::Broker::new(broker_config());
    std::thread::spawn(move || {
        if let Err(e) = broker.start() {
            eprintln!("Broker error: {e}");
        }
    });
    eprintln!("Broker running");

    let mqttoptions = MqttOptions::new("roadway_backend", "127.0.0.1", 1883);
    let (client, mut connection) = Client::new(mqttoptions, 10);

    std::thread::spawn(move || loop {
        match connection.recv() {
            Ok(Ok(Event::Incoming(Packet::Publish(publish)))) => {
                let msg = String::from_utf8_lossy(&publish.payload);
                eprintln!("[gate/status] {msg}");
            }
            Ok(Ok(_)) => {}
            Ok(Err(e)) => {
                eprintln!("[client] MQTT error: {e:?}");
                std::thread::sleep(Duration::from_secs(5));
            }
            Err(_) => {
                eprintln!("[client] Channel closed");
                break;
            }
        }
    });

    std::thread::sleep(Duration::from_secs(1));
    client.subscribe("gate/status", QoS::AtMostOnce)?;
    eprintln!("Subscribed to gate/status");

    let cmd_client = client.clone();
    std::thread::spawn(move || {
        let listener = TcpListener::bind("0.0.0.0:19090").expect("bind TCP");
        eprintln!("Command API on port 19090");
        for stream in listener.incoming() {
            match stream {
                Ok(mut stream) => {
                    let mut buf = [0; 1024];
                    match stream.read(&mut buf) {
                        Ok(n) if n > 0 => {
                            let cmd = String::from_utf8_lossy(&buf[..n]).trim().to_string();
                            if cmd.eq_ignore_ascii_case("OPEN") {
                                match cmd_client.publish("gate/control", QoS::AtLeastOnce, false, "OPEN") {
                                    Ok(_) => {
                                        let _ = stream.write_all(b"OK\n");
                                        eprintln!("[cmd] Published OPEN");
                                    }
                                    Err(e) => {
                                        let _ = stream.write_all(format!("ERR: {e}\n").as_bytes());
                                        eprintln!("[cmd] Error: {e}");
                                    }
                                }
                            } else if cmd.eq_ignore_ascii_case("exit")
                                || cmd.eq_ignore_ascii_case("quit")
                            {
                                let _ = stream.write_all(b"BYE\n");
                                std::process::exit(0);
                            } else {
                                let _ = stream.write_all(b"ERR: unknown command\n");
                            }
                        }
                        _ => {}
                    }
                }
                Err(e) => eprintln!("[tcp] Accept error: {e}"),
            }
        }
    });

    eprintln!("READY");
    eprintln!("Send commands: echo OPEN | nc 127.0.0.1 19090");

    loop {
        std::thread::sleep(Duration::from_secs(3600));
    }
}
