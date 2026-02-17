use anyhow::{anyhow, Result};
use base64::{engine::general_purpose, Engine};
use url::Url;
use uuid::Uuid;

use super::models::*;

/// Parse a single proxy link into a Server
pub fn parse_link(link: &str) -> Result<Server> {
    let link = link.trim();
    if link.starts_with("vless://") {
        parse_vless(link)
    } else if link.starts_with("vmess://") {
        parse_vmess(link)
    } else if link.starts_with("ss://") {
        parse_shadowsocks(link)
    } else if link.starts_with("trojan://") {
        parse_trojan(link)
    } else if link.starts_with("hy2://") {
        parse_hysteria2(link)
    } else if link.starts_with("hysteria2://") {
        parse_hysteria2(link)
    } else if link.starts_with("tuic://") {
        parse_tuic(link)
    } else if link.starts_with("ssh://") {
        parse_ssh(link)
    } else if link.starts_with("wg://") || link.starts_with("wireguard://") {
        parse_wireguard(link)
    } else if is_valid_json(link) {
        parse_json_config(link)
    } else {
        Err(anyhow!("Unsupported link format: {}", &link[..20.min(link.len())]))
    }
}

/// Parse multiple links (one per line)
pub fn parse_links(text: &str) -> Vec<Server> {
    text.lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .filter_map(|l| parse_link(l).ok())
        .collect()
}

/// Decode base64 subscription content, then parse links
pub fn parse_subscription_content(content: &str) -> Vec<Server> {
    // Try base64 decode first
    let decoded = general_purpose::STANDARD
        .decode(content.trim())
        .or_else(|_| general_purpose::URL_SAFE.decode(content.trim()))
        .or_else(|_| general_purpose::URL_SAFE_NO_PAD.decode(content.trim()));

    match decoded {
        Ok(bytes) => {
            if let Ok(text) = String::from_utf8(bytes) {
                // Check if the decoded content is a JSON array of configs
                if is_valid_json(&text) {
                    parse_json_subscription_content(&text)
                } else {
                    parse_links(&text)
                }
            } else {
                // If decoding failed, try parsing as-is
                if is_valid_json(content) {
                    parse_json_subscription_content(content)
                } else {
                    parse_links(content)
                }
            }
        }
        Err(_) => {
            // If base64 decoding failed, try parsing as-is
            if is_valid_json(content) {
                parse_json_subscription_content(content)
            } else {
                parse_links(content)
            }
        }
    }
}

/// Parse subscription content that is JSON (either single JSON or array of JSON configs)
fn parse_json_subscription_content(content: &str) -> Vec<Server> {
    let trimmed = content.trim();
    
    // Try to parse as a single JSON object first
    if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(trimmed) {
        if json_value.is_object() {
            // Single JSON config
            if let Ok(server) = parse_json_config(trimmed) {
                return vec![server];
            }
        } else if json_value.is_array() {
            // Array of JSON configs
            let mut servers = Vec::new();
            if let Some(array) = json_value.as_array() {
                for item in array {
                    if item.is_object() {
                        if let Ok(json_str) = serde_json::to_string(item) {
                            if let Ok(server) = parse_json_config(&json_str) {
                                servers.push(server);
                            }
                        }
                    }
                }
            }
            return servers;
        }
    }
    
    // If JSON parsing fails, return empty vector
    Vec::new()
}

// ── VLESS ──────────────────────────────────────────────

fn parse_vless(link: &str) -> Result<Server> {
    // vless://uuid@host:port?params#name
    let url = Url::parse(link)?;

    let uuid = url.username().to_string();
    let host = url.host_str().ok_or(anyhow!("No host"))?.to_string();
    let port = url.port().unwrap_or(443);
    let name = percent_decode(url.fragment().unwrap_or("VLESS Server"));

    let params = QueryParams::from_url(&url);

    let transport = match params.get("type").as_deref() {
        Some("tcp") => Transport::Tcp,
        Some("kcp") => Transport::Kcp,
        Some("ws") => Transport::Ws,
        Some("grpc") => Transport::Grpc,
        Some("h2") | Some("http") => Transport::Http,
        Some("quic") => Transport::Quic,
        Some("xhttp") => Transport::XHttp,
        Some("httpupgrade") => Transport::HttpUpgrade,
        _ => Transport::Tcp,
    };

    let ws = if transport == Transport::Ws {
        Some(WsSettings {
            path: params.get("path").unwrap_or_else(|| "/".to_string()),
            host: params.get("host"),
        })
    } else {
        None
    };

    let grpc = if transport == Transport::Grpc {
        Some(GrpcSettings {
            service_name: params.get("serviceName").unwrap_or_default(),
        })
    } else {
        None
    };

    let xhttp = if transport == Transport::XHttp {
        Some(XHttpSettings {
            host: params.get("host"),
            path: params.get("path").unwrap_or_else(|| "/".to_string()),
            mode: params.get("mode").unwrap_or_else(|| "auto".to_string()),
        })
    } else {
        None
    };

    let httpupgrade = if transport == Transport::HttpUpgrade {
        Some(HttpUpgradeSettings {
            host: params.get("host"),
            path: params.get("path").unwrap_or_else(|| "/".to_string()),
        })
    } else {
        None
    };

    let kcp = if transport == Transport::Kcp {
        Some(KcpSettings {
            header_type: params.get("headerType").unwrap_or_else(|| "none".to_string()),
            seed: params.get("seed"),
        })
    } else {
        None
    };

    let quic = if transport == Transport::Quic {
        Some(QuicSettings {
            header_type: params.get("headerType").unwrap_or_else(|| "none".to_string()),
            quic_security: params.get("quicSecurity").unwrap_or_else(|| "none".to_string()),
            key: params.get("key").unwrap_or_default(),
        })
    } else {
        None
    };

    let security = params.get("security").unwrap_or_else(|| "none".to_string());
    let reality = if security == "reality" {
        Some(RealitySettings {
            public_key: params.get("pbk").unwrap_or_default(),
            short_id: params.get("sid").unwrap_or_default(),
        })
    } else {
        None
    };

    let tls = TlsSettings {
        enabled: security == "tls" || security == "reality",
        server_name: params.get("sni"),
        insecure: params.get("allowInsecure").as_deref() == Some("1"),
        alpn: params
            .get("alpn")
            .map(|a| a.split(',').map(String::from).collect())
            .unwrap_or_default(),
        fingerprint: params.get("fp"),
        reality,
        // Additional TLS fields
        disable_sni: params.get("disable_sni").as_deref() == Some("1") || params.get("disable_sni").as_deref() == Some("true"),
        min_version: params.get("min_version"),
        max_version: params.get("max_version"),
        cipher_suites: params
            .get("cipher_suites")
            .map(|c| c.split(',').map(String::from).collect())
            .unwrap_or_default(),
        curve_preferences: params
            .get("curve_preferences")
            .map(|c| c.split(',').map(String::from).collect())
            .unwrap_or_default(),
        certificate: params.get("certificate"),
        certificate_path: params.get("certificate_path"),
        certificate_public_key_sha256: params
            .get("certificate_public_key_sha256")
            .map(|c| c.split(',').map(String::from).collect())
            .unwrap_or_default(),
        client_certificate: params.get("client_certificate"),
        client_certificate_path: params.get("client_certificate_path"),
        client_key: params.get("client_key"),
        client_key_path: params.get("client_key_path"),
        utls_enabled: params.get("utls_enabled").as_deref() == Some("1") || params.get("utls_enabled").as_deref() == Some("true"),
    };

    Ok(Server {
        id: Uuid::new_v4().to_string(),
        name,
        address: host,
        port,
        protocol: Protocol::Vless,
        uuid: Some(uuid),
        password: None,
        method: None,
        flow: params.get("flow"),
        alter_id: None,
        transport,
        ws,
        grpc,
        xhttp,
        httpupgrade,
        kcp: None,
        quic: None,
        tls,
        ssh_settings: None,
        wireguard_settings: None,
        tun_settings: None,
        subscription_id: None,
        latency_ms: None,
        json_config: None,
    })
}

// ── VMess ──────────────────────────────────────────────

fn parse_vmess(link: &str) -> Result<Server> {
    // vmess://base64json
    let encoded = link.strip_prefix("vmess://").ok_or(anyhow!("Invalid vmess link"))?;

    let decoded = general_purpose::STANDARD
        .decode(encoded.trim())
        .or_else(|_| general_purpose::URL_SAFE.decode(encoded.trim()))
        .or_else(|_| general_purpose::URL_SAFE_NO_PAD.decode(encoded.trim()))?;

    let json: serde_json::Value = serde_json::from_slice(&decoded)?;

    let host = json_str(&json, "add")?;
    let port = json["port"]
        .as_u64()
        .or_else(|| json["port"].as_str()?.parse().ok())
        .unwrap_or(443) as u16;
    let uuid = json_str(&json, "id")?;
    let name = json["ps"].as_str().unwrap_or("VMess Server").to_string();
    let aid = json["aid"]
        .as_u64()
        .or_else(|| json["aid"].as_str()?.parse().ok())
        .unwrap_or(0) as u32;

    let net = json["net"].as_str().unwrap_or("tcp");
    let transport = match net {
        "tcp" => Transport::Tcp,
        "kcp" => Transport::Kcp,
        "ws" => Transport::Ws,
        "grpc" => Transport::Grpc,
        "h2" | "http" => Transport::Http,
        "quic" => Transport::Quic,
        "xhttp" => Transport::XHttp,
        "httpupgrade" => Transport::HttpUpgrade,
        _ => Transport::Tcp,
    };

    let ws = if transport == Transport::Ws {
        Some(WsSettings {
            path: json["path"].as_str().unwrap_or("/").to_string(),
            host: json["host"].as_str().map(String::from),
        })
    } else {
        None
    };

    let xhttp = if transport == Transport::XHttp {
        Some(XHttpSettings {
            host: json["host"].as_str().map(String::from),
            path: json["path"].as_str().unwrap_or("/").to_string(),
            mode: json["mode"].as_str().unwrap_or("auto").to_string(),
        })
    } else {
        None
    };

    let httpupgrade = if transport == Transport::HttpUpgrade {
        Some(HttpUpgradeSettings {
            host: json["host"].as_str().map(String::from),
            path: json["path"].as_str().unwrap_or("/").to_string(),
        })
    } else {
        None
    };

    let kcp = if transport == Transport::Kcp {
        Some(KcpSettings {
            header_type: json["type"].as_str().unwrap_or("none").to_string(),
            seed: json["path"].as_str().map(String::from),  // path is used for seed in kcp
        })
    } else {
        None
    };

    let quic = if transport == Transport::Quic {
        Some(QuicSettings {
            header_type: json["type"].as_str().unwrap_or("none").to_string(),
            quic_security: json["host"].as_str().unwrap_or("none").to_string(),  // host is used for quic security in vmess
            key: json["path"].as_str().unwrap_or("").to_string(),  // path is used for key in vmess quic
        })
    } else {
        None
    };

    let tls_val = json["tls"].as_str().unwrap_or("");
    let tls = TlsSettings {
        enabled: tls_val == "tls",
        server_name: json["sni"].as_str().map(String::from),
        insecure: json["allowInsecure"].as_str().unwrap_or("") == "true" || json["allowInsecure"].as_str().unwrap_or("") == "1",
        alpn: json["alpn"].as_array().map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect()).unwrap_or_default(),
        fingerprint: json["fp"].as_str().map(String::from),
        reality: None,
        // Additional TLS fields
        disable_sni: json["disable_sni"].as_str().unwrap_or("") == "true" || json["disable_sni"].as_str().unwrap_or("") == "1",
        min_version: json["min_version"].as_str().map(String::from),
        max_version: json["max_version"].as_str().map(String::from),
        cipher_suites: json["cipher_suites"].as_array().map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect()).unwrap_or_default(),
        curve_preferences: json["curve_preferences"].as_array().map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect()).unwrap_or_default(),
        certificate: json["certificate"].as_str().map(String::from),
        certificate_path: json["certificate_path"].as_str().map(String::from),
        certificate_public_key_sha256: json["certificate_public_key_sha256"].as_array().map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect()).unwrap_or_default(),
        client_certificate: json["client_certificate"].as_str().map(String::from),
        client_certificate_path: json["client_certificate_path"].as_str().map(String::from),
        client_key: json["client_key"].as_str().map(String::from),
        client_key_path: json["client_key_path"].as_str().map(String::from),
        utls_enabled: json["utls_enabled"].as_str().unwrap_or("") == "true" || json["utls_enabled"].as_str().unwrap_or("") == "1",
    };

    Ok(Server {
        id: Uuid::new_v4().to_string(),
        name,
        address: host,
        port,
        protocol: Protocol::Vmess,
        uuid: Some(uuid),
        password: None,
        method: None,
        flow: None,
        alter_id: Some(aid),
        transport,
        ws,
        grpc: None,
        xhttp,
        httpupgrade,
        kcp,
        quic,
        tls,
        ssh_settings: None,
        wireguard_settings: None,
        tun_settings: None,
        subscription_id: None,
        latency_ms: None,
        json_config: None,
    })
}

// ── Shadowsocks ────────────────────────────────────────

fn parse_shadowsocks(link: &str) -> Result<Server> {
    // Check if the link has query parameters (new format)
    let parsed_url = Url::parse(link).ok();
    
    if let Some(url) = parsed_url {
        // New format: ss://method:password@host:port/?params#name
        let without_prefix = link.strip_prefix("ss://").ok_or(anyhow!("Invalid ss link"))?;
        let (creds_and_host, name) = match without_prefix.split_once('#') {
            Some((m, n)) => (m, percent_decode(n)),
            None => (without_prefix, "SS Server".to_string()),
        };
        
        let params = QueryParams::from_url(&url);
        
        // Extract method:password from the credentials
        let mut creds_and_host_part = creds_and_host.split('?').next().unwrap_or(creds_and_host);
        if creds_and_host_part.contains('@') {
            let (creds, host_port) = creds_and_host_part.split_once('@').ok_or(anyhow!("Invalid ss link format"))?;
            let (method, password) = creds
                .split_once(':')
                .ok_or(anyhow!("Invalid method:password in ss link"))?;
            
            let (host, port) = parse_host_port(host_port)?;
            
            // Parse transport settings
            let transport = match params.get("type").as_deref() {
                Some("tcp") => Transport::Tcp,
                Some("kcp") => Transport::Kcp,
                Some("ws") => Transport::Ws,
                Some("grpc") => Transport::Grpc,
                Some("h2") | Some("http") => Transport::Http,
                Some("quic") => Transport::Quic,
                Some("xhttp") => Transport::XHttp,
                Some("httpupgrade") => Transport::HttpUpgrade,
                _ => Transport::Tcp,
            };
            
            let ws = if transport == Transport::Ws {
                Some(WsSettings {
                    path: params.get("path").unwrap_or_else(|| "/".to_string()),
                    host: params.get("host"),
                })
            } else {
                None
            };
            
            let grpc = if transport == Transport::Grpc {
                Some(GrpcSettings {
                    service_name: params.get("serviceName").unwrap_or_default(),
                })
            } else {
                None
            };
            
            let xhttp = if transport == Transport::XHttp {
                Some(XHttpSettings {
                    host: params.get("host"),
                    path: params.get("path").unwrap_or_else(|| "/".to_string()),
                    mode: params.get("mode").unwrap_or_else(|| "auto".to_string()),
                })
            } else {
                None
            };
            
            let httpupgrade = if transport == Transport::HttpUpgrade {
                Some(HttpUpgradeSettings {
                    host: params.get("host"),
                    path: params.get("path").unwrap_or_else(|| "/".to_string()),
                })
            } else {
                None
            };
            
            let kcp = if transport == Transport::Kcp {
                Some(KcpSettings {
                    header_type: params.get("headerType").unwrap_or_else(|| "none".to_string()),
                    seed: params.get("seed"),
                })
            } else {
                None
            };
            
            let quic = if transport == Transport::Quic {
                Some(QuicSettings {
                    header_type: params.get("headerType").unwrap_or_else(|| "none".to_string()),
                    quic_security: params.get("quicSecurity").unwrap_or_else(|| "none".to_string()),
                    key: params.get("key").unwrap_or_default(),
                })
            } else {
                None
            };
            
            // Handle TLS settings
            let security = params.get("security").unwrap_or_else(|| "".to_string());
            let reality = if security == "reality" {
                Some(RealitySettings {
                    public_key: params.get("pbk").unwrap_or_default(),
                    short_id: params.get("sid").unwrap_or_default(),
                })
            } else {
                None
            };
                
            let tls = TlsSettings {
                enabled: security == "tls" || security == "reality",
                server_name: params.get("sni"),
                insecure: params.get("allowInsecure").as_deref() == Some("1") || params.get("allowInsecure").as_deref() == Some("true"),
                alpn: params
                    .get("alpn")
                    .map(|a| a.split(',').map(String::from).collect())
                    .unwrap_or_default(),
                fingerprint: params.get("fp"),
                reality,
                // Additional TLS fields
                disable_sni: params.get("disable_sni").as_deref() == Some("1") || params.get("disable_sni").as_deref() == Some("true"),
                min_version: params.get("min_version"),
                max_version: params.get("max_version"),
                cipher_suites: params
                    .get("cipher_suites")
                    .map(|c| c.split(',').map(String::from).collect())
                    .unwrap_or_default(),
                curve_preferences: params
                    .get("curve_preferences")
                    .map(|c| c.split(',').map(String::from).collect())
                    .unwrap_or_default(),
                certificate: params.get("certificate"),
                certificate_path: params.get("certificate_path"),
                certificate_public_key_sha256: params
                    .get("certificate_public_key_sha256")
                    .map(|c| c.split(',').map(String::from).collect())
                    .unwrap_or_default(),
                client_certificate: params.get("client_certificate"),
                client_certificate_path: params.get("client_certificate_path"),
                client_key: params.get("client_key"),
                client_key_path: params.get("client_key_path"),
                utls_enabled: params.get("utls_enabled").as_deref() == Some("1") || params.get("utls_enabled").as_deref() == Some("true"),
            };
            
            return Ok(Server {
                id: Uuid::new_v4().to_string(),
                name,
                address: host,
                port,
                protocol: Protocol::Shadowsocks,
                uuid: None,
                password: Some(password.to_string()),
                method: Some(method.to_string()),
                flow: None,
                alter_id: None,
                transport,
                ws,
                grpc,
                xhttp,
                httpupgrade,
                kcp: None,
                quic: None,
                tls,
                ssh_settings: None,
                wireguard_settings: None,
                tun_settings: None,
                subscription_id: None,
                latency_ms: None,
                json_config: None,
            });
        }
    }
    
    // Original format: ss://base64(method:password)@host:port#name or ss://base64(method:password@host:port)#name
    let without_prefix = link.strip_prefix("ss://").ok_or(anyhow!("Invalid ss link"))?;

    let (main_part, name) = match without_prefix.split_once('#') {
        Some((m, n)) => (m, percent_decode(n)),
        None => (without_prefix, "SS Server".to_string()),
    };

    // Try format 1: base64@host:port
    if let Some((encoded, server_part)) = main_part.split_once('@') {
        let decoded = decode_b64(encoded)?;
        let (method, password) = decoded
            .split_once(':')
            .ok_or(anyhow!("Invalid ss userinfo"))?;

        let (host, port) = parse_host_port(server_part)?;

        return Ok(make_ss_server(
            name,
            host,
            port,
            method.to_string(),
            password.to_string(),
        ));
    }

    // Try format 2: everything is base64
    let decoded = decode_b64(main_part)?;
    if let Some((userinfo, server_part)) = decoded.split_once('@') {
        let (method, password) = userinfo
            .split_once(':')
            .ok_or(anyhow!("Invalid ss userinfo"))?;
        let (host, port) = parse_host_port(server_part)?;

        return Ok(make_ss_server(
            name,
            host,
            port,
            method.to_string(),
            password.to_string(),
        ));
    }

    Err(anyhow!("Could not parse ss link"))
}

fn make_ss_server(name: String, address: String, port: u16, method: String, password: String) -> Server {
    Server {
        id: Uuid::new_v4().to_string(),
        name,
        address,
        port,
        protocol: Protocol::Shadowsocks,
        uuid: None,
        password: Some(password),
        method: Some(method),
        flow: None,
        alter_id: None,
        transport: Transport::Tcp,
        ws: None,
        grpc: None,
        xhttp: None,
        httpupgrade: None,
        kcp: None,
        quic: None,
        tls: TlsSettings::default(),
        ssh_settings: None,
        wireguard_settings: None,
        tun_settings: None,
        subscription_id: None,
        latency_ms: None,
        json_config: None,
    }
}

// ── Trojan ─────────────────────────────────────────────

fn parse_trojan(link: &str) -> Result<Server> {
    // trojan://password@host:port?params#name
    let url = Url::parse(link)?;

    let password = url.username().to_string();
    let host = url.host_str().ok_or(anyhow!("No host"))?.to_string();
    let port = url.port().unwrap_or(443);
    let name = percent_decode(url.fragment().unwrap_or("Trojan Server"));

    let params = QueryParams::from_url(&url);

    let transport = match params.get("type").as_deref() {
        Some("tcp") => Transport::Tcp,
        Some("kcp") => Transport::Kcp,
        Some("ws") => Transport::Ws,
        Some("grpc") => Transport::Grpc,
        Some("h2") | Some("http") => Transport::Http,
        Some("quic") => Transport::Quic,
        Some("xhttp") => Transport::XHttp,
        Some("httpupgrade") => Transport::HttpUpgrade,
        _ => Transport::Tcp,
    };

    let ws = if transport == Transport::Ws {
        Some(WsSettings {
            path: params.get("path").unwrap_or_else(|| "/".to_string()),
            host: params.get("host"),
        })
    } else {
        None
    };

    let xhttp = if transport == Transport::XHttp {
        Some(XHttpSettings {
            host: params.get("host"),
            path: params.get("path").unwrap_or_else(|| "/".to_string()),
            mode: params.get("mode").unwrap_or_else(|| "auto".to_string()),
        })
    } else {
        None
    };

    let httpupgrade = if transport == Transport::HttpUpgrade {
        Some(HttpUpgradeSettings {
            host: params.get("host"),
            path: params.get("path").unwrap_or_else(|| "/".to_string()),
        })
    } else {
        None
    };

    let tls = TlsSettings {
        enabled: true,
        server_name: params.get("sni"),
        insecure: params.get("allowInsecure").as_deref() == Some("1"),
        alpn: params
            .get("alpn")
            .map(|a| a.split(',').map(String::from).collect())
            .unwrap_or_default(),
        fingerprint: params.get("fp"),
        reality: None,
        // Additional TLS fields
        disable_sni: params.get("disable_sni").as_deref() == Some("1") || params.get("disable_sni").as_deref() == Some("true"),
        min_version: params.get("min_version"),
        max_version: params.get("max_version"),
        cipher_suites: params
            .get("cipher_suites")
            .map(|c| c.split(',').map(String::from).collect())
            .unwrap_or_default(),
        curve_preferences: params
            .get("curve_preferences")
            .map(|c| c.split(',').map(String::from).collect())
            .unwrap_or_default(),
        certificate: params.get("certificate"),
        certificate_path: params.get("certificate_path"),
        certificate_public_key_sha256: params
            .get("certificate_public_key_sha256")
            .map(|c| c.split(',').map(String::from).collect())
            .unwrap_or_default(),
        client_certificate: params.get("client_certificate"),
        client_certificate_path: params.get("client_certificate_path"),
        client_key: params.get("client_key"),
        client_key_path: params.get("client_key_path"),
        utls_enabled: params.get("utls_enabled").as_deref() == Some("1") || params.get("utls_enabled").as_deref() == Some("true"),
    };

    Ok(Server {
        id: Uuid::new_v4().to_string(),
        name,
        address: host,
        port,
        protocol: Protocol::Trojan,
        uuid: None,
        password: Some(password),
        method: None,
        flow: None,
        alter_id: None,
        transport,
        ws,
        grpc: None,
        xhttp,
        httpupgrade,
        kcp: None,
        quic: None,
        tls,
        ssh_settings: None,
        wireguard_settings: None,
        tun_settings: None,
        subscription_id: None,
        latency_ms: None,
        json_config: None,
    })
}

// ── Hysteria2 ────────────────────────────────────────────

fn parse_hysteria2(link: &str) -> Result<Server> {
    // hy2://password@host:port/?upmbps=100&downmbps=100&obfs=salamander&obfs-password=cry_me_a_r1ver&sni=example.com&insecure=1#Remark
    let url = Url::parse(link)?;
    
    let password = url.username().to_string();
    let host = url.host_str().ok_or(anyhow!("No host"))?.to_string();
    let port = url.port().unwrap_or(443);
    let name = percent_decode(url.fragment().unwrap_or("Hysteria2 Server"));
    
    let params = QueryParams::from_url(&url);
    
    let up_mbps = params.get("upmbps").and_then(|s| s.parse::<u32>().ok()).unwrap_or(0);
    let down_mbps = params.get("downmbps").and_then(|s| s.parse::<u32>().ok()).unwrap_or(0);
    let obfs_type = params.get("obfs").unwrap_or_else(|| "none".to_string());
    let obfs_password = params.get("obfs-password");
    
    // TLS settings
    let tls = TlsSettings {
        enabled: true,
        server_name: params.get("sni").or(params.get("peer")),
        insecure: params.get("insecure").as_deref() == Some("1") || params.get("insecure").as_deref() == Some("true"),
        alpn: params
            .get("alpn")
            .map(|a| a.split(',').map(String::from).collect())
            .unwrap_or_default(),
        fingerprint: params.get("fp"),
        reality: None,
        // Additional TLS fields
        disable_sni: params.get("disable_sni").as_deref() == Some("1") || params.get("disable_sni").as_deref() == Some("true"),
        min_version: params.get("min_version"),
        max_version: params.get("max_version"),
        cipher_suites: params
            .get("cipher_suites")
            .map(|c| c.split(',').map(String::from).collect())
            .unwrap_or_default(),
        curve_preferences: params
            .get("curve_preferences")
            .map(|c| c.split(',').map(String::from).collect())
            .unwrap_or_default(),
        certificate: params.get("certificate"),
        certificate_path: params.get("certificate_path"),
        certificate_public_key_sha256: params
            .get("certificate_public_key_sha256")
            .map(|c| c.split(',').map(String::from).collect())
            .unwrap_or_default(),
        client_certificate: params.get("client_certificate"),
        client_certificate_path: params.get("client_certificate_path"),
        client_key: params.get("client_key"),
        client_key_path: params.get("client_key_path"),
        utls_enabled: params.get("utls_enabled").as_deref() == Some("1") || params.get("utls_enabled").as_deref() == Some("true"),
    };
    
    // Create a Hysteria2-specific configuration in a generic way
    // Store obfs information in method field
    Ok(Server {
        id: Uuid::new_v4().to_string(),
        name,
        address: host,
        port,
        protocol: Protocol::Hysteria2,
        uuid: None,
        password: Some(password),
        method: Some(obfs_type),  // Store obfs type in method field
        flow: Some(format!("upmbps:{};downmbps:{}", up_mbps, down_mbps)),  // Store bandwidth info in flow field
        alter_id: None,
        transport: Transport::Tcp,  // Default transport
        ws: None,
        grpc: None,
        xhttp: None,
        httpupgrade: None,
        kcp: None,
        quic: None,
        tls,
        ssh_settings: None,
        wireguard_settings: None,
        tun_settings: None,
        subscription_id: None,
        latency_ms: None,
        json_config: None,
    })
}

// ── TUIC ─────────────────────────────────────────────────

fn parse_tuic(link: &str) -> Result<Server> {
    // tuic://uuid:password@host:port/?sni=example.com&congestion_control=bbr&udp_relay_mode=native&udp_over_stream=true&zero_rtt_handshake=true&heartbeat=10s&alpn=h3&allow_insecure=1#Remark
    let url = Url::parse(link)?;
    
    // Extract UUID and password from user info
    let user_info = url.username();
    let password = url.password().unwrap_or("").to_string();
    let uuid = user_info.to_string();
    
    let host = url.host_str().ok_or(anyhow!("No host"))?.to_string();
    let port = url.port().unwrap_or(443);
    let name = percent_decode(url.fragment().unwrap_or("TUIC Server"));
    
    let params = QueryParams::from_url(&url);
    
    // Extract TUIC specific parameters
    let congestion_control = params.get("congestion_control").or(params.get("congestion-control")).unwrap_or_else(|| "cubic".to_string());
    let udp_relay_mode = params.get("udp_relay_mode");
    let udp_over_stream = params.get("udp_over_stream").as_deref() == Some("true") || params.get("udp_over_stream").as_deref() == Some("1");
    let zero_rtt_handshake = params.get("zero_rtt_handshake").as_deref() == Some("true") || params.get("zero_rtt_handshake").as_deref() == Some("1");
    let heartbeat = params.get("heartbeat");
    let network = params.get("network").unwrap_or_else(|| "tcp,udp".to_string());
    
    // TLS settings
    let tls = TlsSettings {
        enabled: true,
        server_name: params.get("sni").or(params.get("peer")),
        insecure: params.get("allow_insecure").as_deref() == Some("1") || params.get("allow_insecure").as_deref() == Some("true"),
        alpn: params
            .get("alpn")
            .map(|a| a.split(',').map(String::from).collect())
            .unwrap_or_default(),
        fingerprint: params.get("fp"),
        reality: None,
        // Additional TLS fields
        disable_sni: params.get("disable_sni").as_deref() == Some("1") || params.get("disable_sni").as_deref() == Some("true"),
        min_version: params.get("min_version"),
        max_version: params.get("max_version"),
        cipher_suites: params
            .get("cipher_suites")
            .map(|c| c.split(',').map(String::from).collect())
            .unwrap_or_default(),
        curve_preferences: params
            .get("curve_preferences")
            .map(|c| c.split(',').map(String::from).collect())
            .unwrap_or_default(),
        certificate: params.get("certificate"),
        certificate_path: params.get("certificate_path"),
        certificate_public_key_sha256: params
            .get("certificate_public_key_sha256")
            .map(|c| c.split(',').map(String::from).collect())
            .unwrap_or_default(),
        client_certificate: params.get("client_certificate"),
        client_certificate_path: params.get("client_certificate_path"),
        client_key: params.get("client_key"),
        client_key_path: params.get("client_key_path"),
        utls_enabled: params.get("utls_enabled").as_deref() == Some("1") || params.get("utls_enabled").as_deref() == Some("true"),
    };
    
    // Store TUIC-specific settings in flow field as JSON-like string
    let tuic_settings = format!(
        "{{\"congestion_control\":\"{}\",\"udp_relay_mode\":\"{}\",\"udp_over_stream\":{},\"zero_rtt_handshake\":{},\"heartbeat\":\"{}\",\"network\":\"{}\"}}",
        congestion_control,
        udp_relay_mode.as_deref().unwrap_or("native"),
        udp_over_stream,
        zero_rtt_handshake,
        heartbeat.as_deref().unwrap_or(""),
        network
    );
    
    Ok(Server {
        id: Uuid::new_v4().to_string(),
        name,
        address: host,
        port,
        protocol: Protocol::Tuic,
        uuid: Some(uuid),
        password: Some(password),
        method: Some(congestion_control),  // Store congestion control in method field
        flow: Some(tuic_settings),  // Store additional TUIC settings
        alter_id: None,
        transport: Transport::Tcp,  // Default transport
        ws: None,
        grpc: None,
        xhttp: None,
        httpupgrade: None,
        kcp: None,
        quic: None,
        tls,
        ssh_settings: None,
        wireguard_settings: None,
        tun_settings: None,
        subscription_id: None,
        latency_ms: None,
        json_config: None,
    })
}

// ── SSH ────────────────────────────────────────────────

fn parse_ssh(link: &str) -> Result<Server> {
    // ssh://user:password@host:port/?private_key_path=$HOME/.ssh/id_rsa&private_key_passphrase=passphrase&client_version=SSH-2.0-OpenSSH_7.4p1#Remark
    let url = Url::parse(link)?;
    
    // Extract user info (user:password)
    let user = url.username();
    let password = url.password().unwrap_or("").to_string();
    
    let host = url.host_str().ok_or(anyhow!("No host"))?.to_string();
    let port = url.port().unwrap_or(22);
    let name = percent_decode(url.fragment().unwrap_or("SSH Server"));
    
    let params = QueryParams::from_url(&url);
    
    // Extract SSH specific parameters
    let private_key = params.get("private_key");
    let private_key_path = params.get("private_key_path");
    let private_key_passphrase = params.get("private_key_passphrase");
    let client_version = params.get("client_version");
    
    // Handle host_key parameter (can be multiple)
    let host_keys_str = params.get("host_key");
    let host_key = if let Some(keys_str) = host_keys_str {
        Some(vec![keys_str])
    } else {
        None
    };
    
    // Handle host_key_algorithms parameter (can be multiple)
    let host_key_algorithms_str = params.get("host_key_algorithms");
    let host_key_algorithms = if let Some(algos_str) = host_key_algorithms_str {
        algos_str.split(',').map(|s| s.trim().to_string()).collect()
    } else {
        vec![]
    };
    
    Ok(Server {
        id: Uuid::new_v4().to_string(),
        name,
        address: host,
        port,
        protocol: Protocol::Ssh,
        uuid: None,
        password: if password.is_empty() { None } else { Some(password.clone()) },
        method: None,
        flow: None,
        alter_id: None,
        ssh_settings: Some(SshSettings {
            user: if user.is_empty() { None } else { Some(user.to_string()) },
            password: if password.is_empty() { None } else { Some(password.clone()) },
            private_key: private_key.clone(),
            private_key_path: private_key_path.clone(),
            private_key_passphrase: private_key_passphrase.clone(),
            host_key: host_key.clone(),
            host_key_algorithms: host_key_algorithms.clone(),
            client_version: client_version.clone(),
        }),
        transport: Transport::Tcp,  // Default transport
        ws: None,
        grpc: None,
        xhttp: None,
        httpupgrade: None,
        kcp: None,
        quic: None,
        tls: TlsSettings::default(),
        wireguard_settings: None,
        tun_settings: None,
        subscription_id: None,
        latency_ms: None,
        json_config: None,
    })
}

// ── WireGuard ────────────────────────────────────────────────

fn parse_wireguard(link: &str) -> Result<Server> {
    // wg://private_key@host:port/?address=192.168.1.2&port=12345&public_key=publickey&allowed_ips=0.0.0.0/0&pre_shared_key=presharedkey&persistent_keepalive_interval=25&mtu=1420&system=true&name=wg0#Remark
    let url = Url::parse(link)?;
    
    let private_key = url.username();
    let host = url.host_str().ok_or(anyhow!("No host"))?.to_string();
    let port = url.port().unwrap_or(10000);
    let name = percent_decode(url.fragment().unwrap_or("WireGuard Server"));
    
    let params = QueryParams::from_url(&url);
    
    // Parse WireGuard parameters
    let address = params.get("address").map(|addr| {
        addr.split(',').map(|s| s.trim().to_string()).collect()
    }).unwrap_or_default();
    let public_key = params.get("public_key").unwrap_or_default();
    let pre_shared_key = params.get("pre_shared_key");
    let allowed_ips = params.get("allowed_ips").map(|ips| {
        ips.split(',').map(|s| s.trim().to_string()).collect()
    }).unwrap_or_else(|| vec!["0.0.0.0/0".to_string()]);
    let persistent_keepalive_interval = params.get("persistent_keepalive_interval").and_then(|s| s.parse::<u32>().ok());
    let mtu = params.get("mtu").and_then(|s| s.parse::<u32>().ok());
    let system = params.get("system").as_deref() == Some("true");
    let wg_name = params.get("name");
    let listen_port = params.get("listen_port").and_then(|s| s.parse::<u16>().ok());
    let udp_timeout = params.get("udp_timeout");
    let workers = params.get("workers").and_then(|s| s.parse::<u32>().ok());
    
    // Parse reserved bytes if provided
    let reserved_param = params.get("reserved");
    let reserved = if let Some(res_str) = reserved_param {
        let mut res: Vec<u8> = Vec::new();
        for num in res_str.split(',') {
            if let Ok(val) = num.trim().parse::<u8>() {
                res.push(val);
            }
        }
        if !res.is_empty() { Some(res) } else { None }
    } else {
        None
    };
    
    // Create WireGuard peer
    let peer = WireGuardPeer {
        address: host,
        port: Some(port),
        public_key,
        pre_shared_key,
        allowed_ips,
        persistent_keepalive_interval,
        reserved,
    };
    
    Ok(Server {
        id: Uuid::new_v4().to_string(),
        name,
        address: "127.0.0.1".to_string(), // Placeholder, actual address is in WireGuard settings
        port: 0, // Placeholder, actual port is in WireGuard settings
        protocol: Protocol::WireGuard,
        uuid: None,
        password: None,
        method: None,
        flow: None,
        alter_id: None,
        ssh_settings: None,
        wireguard_settings: Some(WireGuardSettings {
            system: if system { Some(true) } else { None },
            name: wg_name,
            mtu,
            address,
            private_key: private_key.to_string(),
            listen_port,
            peers: vec![peer],
            udp_timeout,
            workers,
        }),
        transport: Transport::Tcp,  // Default transport
        ws: None,
        grpc: None,
        xhttp: None,
        httpupgrade: None,
        kcp: None,
        quic: None,
        tls: TlsSettings::default(),
        tun_settings: None,
        subscription_id: None,
        latency_ms: None,
        json_config: None,
    })
}

// ── Helpers ────────────────────────────────────────────

struct QueryParams(Vec<(String, String)>);

impl QueryParams {
    fn from_url(url: &Url) -> Self {
        Self(url.query_pairs().map(|(k, v)| (k.to_string(), v.to_string())).collect())
    }

    fn get(&self, key: &str) -> Option<String> {
        self.0.iter().find(|(k, _)| k == key).map(|(_, v)| v.clone())
    }
}

fn json_str(json: &serde_json::Value, key: &str) -> Result<String> {
    json[key]
        .as_str()
        .map(String::from)
        .ok_or_else(|| anyhow!("Missing field: {}", key))
}

fn percent_decode(s: &str) -> String {
    percent_encoding::percent_decode_str(s)
        .decode_utf8_lossy()
        .to_string()
}

fn decode_b64(s: &str) -> Result<String> {
    let bytes = general_purpose::STANDARD
        .decode(s.trim())
        .or_else(|_| general_purpose::URL_SAFE.decode(s.trim()))
        .or_else(|_| general_purpose::URL_SAFE_NO_PAD.decode(s.trim()))?;
    Ok(String::from_utf8(bytes)?)
}

fn parse_host_port(s: &str) -> Result<(String, u16)> {
    let s = s.trim();
    if let Some(idx) = s.rfind(':') {
        let host = s[..idx].to_string();
        let port: u16 = s[idx + 1..].parse()?;
        Ok((host, port))
    } else {
        Err(anyhow!("Cannot parse host:port from '{}'", s))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_vless() {
        let link = "vless://uuid-here@example.com:443?security=tls&type=ws&path=%2Fws&sni=example.com#Test%20Server";
        let server = parse_link(link).unwrap();
        assert_eq!(server.protocol, Protocol::Vless);
        assert_eq!(server.address, "example.com");
        assert_eq!(server.port, 443);
        assert_eq!(server.name, "Test Server");
        assert!(server.tls.enabled);
    }

    #[test]
    fn test_parse_trojan() {
        let link = "trojan://password123@example.com:443?sni=example.com#Trojan";
        let server = parse_link(link).unwrap();
        assert_eq!(server.protocol, Protocol::Trojan);
        assert_eq!(server.password, Some("password123".to_string()));
    }
}

/// Check if a string is valid JSON
fn is_valid_json(s: &str) -> bool {
    serde_json::from_str::<serde_json::Value>(s.trim()).is_ok()
}

/// Parse a JSON config as a Custom server
fn parse_json_config(json_str: &str) -> Result<Server> {
    // Validate that it's valid JSON
    let json_value: serde_json::Value = serde_json::from_str(json_str.trim())
        .map_err(|e| anyhow!("Invalid JSON: {}", e))?;
    
    // For JSON configs, we'll use the JSON content itself as the "address"
    // and assign a default name
    let name = extract_name_from_json(&json_value).unwrap_or_else(|| "Custom Config".to_string());
    
    Ok(Server {
        id: Uuid::new_v4().to_string(),
        name,
        address: "custom-json".to_string(), // Placeholder address
        port: 0, // Placeholder port
        protocol: Protocol::Custom,
        uuid: None,
        password: None,
        method: Some("json".to_string()), // Indicate this is a JSON config
        flow: None,
        alter_id: None,
        transport: Transport::Tcp, // Default transport
        ws: None,
        grpc: None,
        xhttp: None,
        httpupgrade: None,
        kcp: None,
        quic: None,
        tls: TlsSettings::default(),
        ssh_settings: None,
        wireguard_settings: None,
        tun_settings: None,
        subscription_id: None,
        latency_ms: None,
        json_config: Some(json_str.trim().to_string()), // Store the original JSON config
    })
}

/// Try to extract a name from the JSON config if possible
fn extract_name_from_json(json: &serde_json::Value) -> Option<String> {
    // Try common fields that might contain a name
    if let Some(name) = json.get("remarks").and_then(|v| v.as_str()) {
        return Some(name.to_string());
    }
    if let Some(name) = json.get("name").and_then(|v| v.as_str()) {
        return Some(name.to_string());
    }
    if let Some(name) = json.get("remark").and_then(|v| v.as_str()) {
        return Some(name.to_string());
    }
    None
}
