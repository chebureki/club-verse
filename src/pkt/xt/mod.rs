pub mod decl;

use anyhow::{bail, Error, Result};

const XT_DEFAULT_INT_ID: isize = -1;

#[derive(Debug, Clone, PartialEq)]
pub enum XTVariant {
    Server,
    Client,
}
#[derive(Debug, Clone, PartialEq)]
pub struct XTPacket {
    // only used in client requests ...TODO: why tf does it exist???
    pub handler_id: Option<String>,

    pub packet_id: String,
    // TODO: only signed since houdini uses signum, check whether actually needed!
    pub internal_id: isize,
    pub(crate) data: Vec<String>,
}

pub fn deserialize<'raw>(raw: &'raw str, variant: XTVariant) -> Result<XTPacket, Error> {
    let raw = match raw {
        raw if !raw.starts_with("%") => bail!("bad leading %"),
        raw if !raw.ends_with("%") => bail!("bad trailing %"),
        raw => &raw[1..raw.len() - 1],
    };

    let mut iter = raw.split('%');
    match iter.next() {
        Some("xt") => {}
        _ => bail!("bad xt prefix"),
    };

    let handler_id = match variant {
        XTVariant::Client => match iter.next() {
            Some(hi) if hi.len() > 0 => Some(hi),
            _ => bail!("bad extension"),
        },
        XTVariant::Server => None,
    };

    let packet_id = match iter.next() {
        Some(pi) if pi.len() > 0 => pi,
        _ => bail!("bad packet id"),
    };

    let internal_id: isize = match iter.next().map(|s| s.parse()) {
        Some(Ok(ii)) => ii,
        _ => bail!("bad internal id"),
    };

    let mut data: Vec<String> = Vec::with_capacity(16);

    while let Some(val) = iter.next() {
        data.push(val.to_owned());
    }

    Ok(XTPacket {
        handler_id: handler_id.map(|s| s.to_owned()),
        packet_id: packet_id.to_owned(),
        internal_id: internal_id.to_owned(),
        data,
    })
}

pub fn serialize(xt: XTPacket) -> String {
    // TODO: enough capacity?
    let mut s = String::with_capacity(128);

    //let raw_pkg = "%xt%s%u#sp%1%395%384%";
    s.push('%');
    s.push_str("xt");
    s.push('%');
    if let Some(handler_id) = &xt.handler_id {
        s.push_str(handler_id);
        s.push('%');
    }
    s.push_str(&xt.packet_id);
    s.push('%');
    // TODO: whack ... internal id is mostly hard coded anyway, const str may be enough
    s.push_str(&xt.internal_id.to_string());
    s.push('%');

    let mut slice_iter = xt.data.iter();
    while let Some(val) = slice_iter.next() {
        s.push_str(val);
        s.push('%');
    }
    // loop creates the trailing %
    s
}

#[cfg(test)]
mod xt_parse_tests {
    use assert_matches::assert_matches;

    use super::*;

    #[test]
    fn basic_deserialize() {
        let raw_pkg = "%xt%s%u#sp%1%395%384%";
        let xt = deserialize(raw_pkg, XTVariant::Client).expect("failed to parse");
        assert_matches!(xt, XTPacket { handler_id: Some(handler_id),packet_id, internal_id, data }
        if
        handler_id == "s" &&
        packet_id == "u#sp" &&
        internal_id == 1 &&
        data == ["395", "384"]
        );
    }

    #[test]
    fn basic_serialize_client() {
        let xt = XTPacket {
            handler_id: Some("s".to_owned()),
            packet_id: "u#sp".to_owned(),
            internal_id: 1,
            data: vec!["395".to_owned(), "384".to_owned()],
        };
        assert_eq!(serialize(xt), "%xt%s%u#sp%1%395%384%");
    }


    #[test]
    fn basic_serialize_server() {
        let xt = XTPacket {
            handler_id: None,
            packet_id: "sp".to_owned(),
            internal_id: 1,
            data: vec!["5".to_owned(), "395".to_owned(), "384".to_owned()],
        };
        assert_eq!(serialize(xt), "%xt%sp%1%5%395%384%");
    }
}
