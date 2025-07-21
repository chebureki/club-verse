pub mod client {
    use anyhow::{anyhow, Result};
    use quick_xml::{events::Event, Reader};

    #[derive(Debug, Clone, PartialEq)]
    pub enum Packet {
        VersionCheck { expected: String },
        // TODO: not sure whether fields relevant!
        RandomKey,
        Login { username: String, password: String },
    }

    // TODO: having a fullfletched xml parser as a
    pub fn deserialize(raw: &str) -> Result<Packet> {
        let mut reader = Reader::from_str(raw);
        let mut buf = Vec::new();

        let mut action = String::new();
        let mut version = String::new();
        let mut username = String::new();
        let mut password = String::new();
        let mut in_nick = false;
        let mut in_pword = false;

        loop {
            match reader.read_event_into(&mut buf)? {
                Event::Start(e) => match e.name().as_ref() {
                    b"body" => {
                        for attr in e.attributes() {
                            let attr = attr?;
                            if attr.key.as_ref() == b"action" {
                                action = std::str::from_utf8(&attr.value)?.to_string();
                            }
                        }
                    }
                    b"nick" => in_nick = true,
                    b"pword" => in_pword = true,
                    _ => {}
                },
                Event::Empty(e) => {
                    if e.name().as_ref() == b"ver" {
                        for attr in e.attributes() {
                            let attr = attr?;
                            if attr.key.as_ref() == b"v" {
                                version = std::str::from_utf8(&attr.value)?.to_string();
                            }
                        }
                    }
                }
                Event::End(e) => match e.name().as_ref() {
                    b"nick" => in_nick = false,
                    b"pword" => in_pword = false,
                    _ => {}
                },
                Event::CData(e) => {
                    let text = std::str::from_utf8(&e)?.to_string();
                    if in_nick {
                        username = text;
                    } else if in_pword {
                        password = text;
                    }
                }
                Event::Eof => break,
                _ => {}
            }
            buf.clear();
        }

        match action.as_str() {
            "verChk" => Ok(Packet::VersionCheck { expected: version }),
            "rndK" => Ok(Packet::RandomKey),
            "login" => Ok(Packet::Login { username, password }),
            _ => Err(anyhow::anyhow!("Unknown action: {}", action)),
        }
    }

    impl TryFrom<String> for Packet{
        type Error = anyhow::Error;

        fn try_from(value: String) -> std::result::Result<Self, Self::Error> {
            deserialize(&value)
        }

    }
}

pub mod server {
    use anyhow::Result;

    #[derive(Debug, Clone, PartialEq)]
    pub enum Packet {
        //<msg t="sys"><body action="apiOK" r="0" /></msg>
        ApiOK,
        //<msg t="sys"><body action="rndK" r="-1"><k>houdini</k></body></msg>
        RandomKey(String),
    }

    // super lazy lmaooooo
    pub fn serialize(packet: Packet) -> String {
        match packet {
            Packet::ApiOK => r#"<msg t="sys"><body action="apiOK" r="0" /></msg>"#.to_string(),
            Packet::RandomKey(key) => {
                format!(
                    r#"<msg t="sys"><body action="rndK" r="-1"><k>{}</k></body></msg>"#,
                    key
                )
            }
        }
    }

    impl Into<String> for Packet{
        fn into(self) -> String {
            serialize(self)
        }

    }
}

#[cfg(test)]
mod tests {
    use assert_matches::assert_matches;

    use super::*;

    #[test]
    fn basic_serialization() {
        assert_eq!(server::serialize(server::Packet::ApiOK), r#"<msg t="sys"><body action="apiOK" r="0" /></msg>"#);
        assert_eq!(server::serialize(server::Packet::RandomKey("foo".to_owned())), r#"<msg t="sys"><body action="rndK" r="-1"><k>foo</k></body></msg>"#);
    }

    #[test]
    fn basic_deserialization() {
        {
            let raw = r"<msg t='sys'><body action='verChk' r='0'><ver v='153' /></body></msg>";
            let pkt: client::Packet = client::deserialize(raw).expect("failed to deserialize");
            assert_matches!(
                pkt,
                client::Packet::VersionCheck { expected} if expected == "153"
            )
        }
        {
            let raw = r"<msg t='sys'><body action='rndK' r='-1'></body></msg>";
            let pkt: client::Packet = client::deserialize(raw).expect("failed to deserialize");
            assert_matches!(pkt, client::Packet::RandomKey)
        }

        {
            let raw = r"<msg t='sys'><body action='login' r='0'><login z='w1'><nick><![CDATA[kirill]]></nick><pword><![CDATA[foo]]></pword></login></body></msg>";
            let pkt: client::Packet = client::deserialize(raw).expect("failed to deserialize");
            assert_matches!(pkt, client::Packet::Login { username, password }
            if username == "kirill" && password == "foo"
            )
        }
    }
}
