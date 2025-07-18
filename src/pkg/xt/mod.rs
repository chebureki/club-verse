pub mod decl;

use anyhow::{anyhow, bail, Context, Error};
use std::fmt;

const XT_MAX_ARG_LEN: usize = 32;

pub struct XTPackage<'raw> {
    pub handler_id: &'raw str,
    // TODO: only signed since houdini uses signum, check whether actually needed!
    pub packet_id: &'raw str,
    pub internal_id: isize,
    pub(crate) data_len: usize,
    pub(crate) data: [&'raw str; XT_MAX_ARG_LEN],
}

impl<'raw> XTPackage<'raw> {
    pub fn data_slice(&'raw self) -> &'raw [&'raw str] {
        &self.data[0..self.data_len]
    }
}

pub trait IntoXT {
    fn into_xt<'raw>(&'raw self) -> XTPackage<'raw>;
}

pub trait FromXT {
    fn from_xt<'raw>(xt: &'raw XTPackage<'raw>) -> Self;
}

impl<'raw> fmt::Debug for XTPackage<'raw> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("XTPackage")
            .field("handler_id", &self.handler_id)
            .field("packet_id", &self.packet_id)
            .field("internal_id", &self.internal_id)
            .field("data_slice", &self.data_slice())
            .finish()
    }
}

pub fn deserialize<'raw>(raw: &'raw str) -> Result<XTPackage, Error> {
    // we avoid allocations extensively here
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

    let handler_id = match iter.next() {
        Some(hi) if hi.len() > 0 => hi,
        _ => bail!("bad extension"),
    };

    let packet_id = match iter.next() {
        Some(pi) if pi.len() > 0 => pi,
        _ => bail!("bad packet id"),
    };

    let internal_id = match iter.next().map(|s| s.parse()) {
        Some(Ok(ii)) => ii,
        _ => bail!("bad internal id"),
    };

    let mut data_len = 0;
    // ugly, but that's rust's safety lol
    let mut data: [&'raw str; XT_MAX_ARG_LEN] = [""; XT_MAX_ARG_LEN];

    while let Some(val) = iter.next() {
        if data_len >= data.len() {
            bail!("arg len exceeded internal maximum");
        }
        data[data_len] = val;
        data_len += 1;
    }

    Ok(XTPackage {
        handler_id,
        packet_id,
        internal_id,
        data_len,
        data,
    })
}

pub fn serialize(xt: XTPackage<'_>) -> String {
    // TODO: enough capacity?
    let mut s = String::with_capacity(128);

    //let raw_pkg = "%xt%s%u#sp%1%395%384%";
    s.push('%');
    s.push_str("xt");
    s.push('%');
    s.push_str(xt.handler_id);
    s.push('%');
    s.push_str(xt.packet_id);
    s.push('%');
    // TODO: whack ... internal id is mostly hard coded anyway, const str may be enough
    s.push_str(&xt.internal_id.to_string());
    s.push('%');

    let mut slice_iter = xt.data_slice().iter();
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
        let xt_pkg = deserialize(raw_pkg).expect("failed to parse");
        assert_matches!(
            xt_pkg,
            XTPackage {
                handler_id: "s",
                packet_id: "u#sp",
                internal_id: 1,
                data_len: 2,
                data: ["395", "384", ..],
                ..
            }
        );
    }

    #[test]
    fn basic_serialize() {
        let mut data = [""; XT_MAX_ARG_LEN];
        data[0] = "395";
        data[1] = "384";
        let xt = XTPackage {
            handler_id: "s",
            packet_id: "u#sp",
            internal_id: 1,
            data_len: 2,
            data,
        };
        assert_eq!(serialize(xt), "%xt%s%u#sp%1%395%384%");
    }
}
