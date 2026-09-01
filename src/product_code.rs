//! 54-FZ tag 1162 helper: builds a product code from a DataMatrix scan string.
//!
//! Port of `yookassa.domain.helpers.product_code.ProductCode` from the
//! official Python SDK (default `use_prefix=True` behavior).

/// Parsed DataMatrix code.
#[derive(Debug, Clone)]
pub struct ProductCode {
    /// `gs_1m`, `gs_10`, `short`, `fur`, `egais_20`, `egais_30`, `itf_14`,
    /// `ean_13`, `ean_8` or `unknown`.
    pub type_: &'static str,
    /// Tag 1162: hex pairs separated by spaces, e.g. `04 36 03 BE ...`.
    /// Empty when the code could not be encoded.
    pub product_code: String,
    /// Tag 1163 raw value.
    pub mark_code_value: String,
}

impl ProductCode {
    /// Build tag 1162/1163 from a scanner (DataMatrix/QR) string.
    pub fn new(code: &str) -> Self {
        // The Python SDK skips parsing entirely for an empty string.
        if code.is_empty() {
            return Self {
                type_: "unknown",
                product_code: String::new(),
                mark_code_value: String::new(),
            };
        }

        let (type_, gtin, serial, app_identifiers) = parse_scan_string(code)
            .map(|(t, m1, m2, split)| {
                let mut gtin = None;
                let mut serial = None;
                let mut app_identifiers: Vec<String> = Vec::new();
                match t {
                    "ean_8" | "ean_13" | "itf_14" | "fur" | "egais_30" => gtin = m1,
                    "egais_20" => gtin = m2,
                    "short" => {
                        gtin = m1;
                        serial = m2;
                    }
                    "gs_1m" | "gs_10" => {
                        gtin = m1;
                        match split {
                            Some(mut parts) if parts.len() > 1 => {
                                serial = Some(parts.remove(0));
                                app_identifiers = parts;
                            }
                            _ => serial = m2,
                        }
                    }
                    _ => {}
                }
                (t, gtin, serial, app_identifiers)
            })
            .unwrap_or(("unknown", None, None, Vec::new()));

        let (gtin, serial) = if type_ == "unknown" {
            let cut = |s: &str, max: usize| -> String {
                if s.len() > max {
                    s[..max].to_string()
                } else {
                    s.to_string()
                }
            };
            (Some(cut(code, 30)), Some(cut(code, 32)))
        } else {
            (gtin, serial)
        };

        let valid = if type_ == "gs_10" || type_ == "gs_1m" || type_ == "short" {
            gtin.is_some() && serial.is_some()
        } else {
            gtin.is_some()
        };

        let gtin = gtin.unwrap_or_default();
        let serial = serial.unwrap_or_default();
        let product_code = if !valid {
            String::new()
        } else {
            let mut result = prefix_for(type_).to_string();
            match type_ {
                "ean_8" | "ean_13" | "itf_14" => result.push_str(&num_to_hex(&gtin)),
                "fur" | "egais_20" | "egais_30" | "unknown" => result.push_str(&str_to_hex(&gtin)),
                "short" => {
                    result.push_str(&num_to_hex(&gtin));
                    result.push_str(&str_to_hex(&serial));
                }
                "gs_1m" | "gs_10" => {
                    result.push_str(&num_to_hex(&gtin));
                    result.push_str(&str_to_hex(&serial));
                    if let Some(sum) = app_identifiers
                        .iter()
                        .find(|a| a.starts_with("8005"))
                        .map(|a| &a[4..])
                    {
                        result.push_str(&str_to_hex(sum));
                    }
                }
                _ => {}
            }
            chunk(result)
        };

        let mark_code_value = match type_ {
            "short" | "gs_1m" | "gs_10" => format!("01{gtin}21{serial}"),
            "unknown" => serial.to_string(),
            _ => gtin.to_string(),
        };

        Self {
            type_,
            product_code,
            mark_code_value,
        }
    }
}

fn prefix_for(type_: &str) -> &'static str {
    match type_ {
        "unknown" => "0000",
        "ean_8" => "4508",
        "ean_13" => "450D",
        "itf_14" => "4909",
        "gs_10" | "gs_1m" | "short" => "444D",
        "fur" => "5246",
        "egais_20" => "C514",
        "egais_30" => "C51E",
        _ => "444D",
    }
}

type ScanMatch = (
    &'static str,
    Option<String>,
    Option<String>,
    Option<Vec<String>>,
);

/// Scanner-string rules in the Python SDK's order.
fn parse_scan_string(code: &str) -> Option<ScanMatch> {
    use std::sync::LazyLock;

    static GS1M: LazyLock<regex::Regex> = LazyLock::new(|| {
        regex::Regex::new(
            r#"^01(\d{14})21(.+)((91(.+)92(.+))|(93[\w!"%&'()*+,-./_:;=<>?]{4}(.*)))$"#,
        )
        .unwrap()
    });
    static GS10: LazyLock<regex::Regex> =
        LazyLock::new(|| regex::Regex::new(r"^01(\d{14})21(.+)$").unwrap());
    static SHORT: LazyLock<regex::Regex> =
        LazyLock::new(|| regex::Regex::new(r"^(\d{14})(.+)$").unwrap());
    static EGAIS20: LazyLock<regex::Regex> =
        LazyLock::new(|| regex::Regex::new(r"^(.{8})(.{33})(.+)$").unwrap());
    static EGAIS30: LazyLock<regex::Regex> =
        LazyLock::new(|| regex::Regex::new(r"^(.{14})(.+)$").unwrap());
    static ITF14: LazyLock<regex::Regex> =
        LazyLock::new(|| regex::Regex::new(r"^(0\d{13})$").unwrap());
    static EAN13: LazyLock<regex::Regex> =
        LazyLock::new(|| regex::Regex::new(r"^(\d{13})$").unwrap());
    static EAN8: LazyLock<regex::Regex> =
        LazyLock::new(|| regex::Regex::new(r"^(\d{8})$").unwrap());
    static FUR: LazyLock<regex::Regex> =
        LazyLock::new(|| regex::Regex::new(r"^((\w{2})-(\d{6})-(\w{10}))$").unwrap());

    if let Some(c) = GS1M.captures(code) {
        let split = c
            .get(2)
            .map(|m| m.as_str().split('\u{1d}').map(str::to_string).collect());
        return Some((
            "gs_1m",
            Some(c[1].to_string()),
            Some(c[2].to_string()),
            split,
        ));
    }
    if let Some(c) = GS10.captures(code) {
        let split = c
            .get(2)
            .map(|m| m.as_str().split('\u{1d}').map(str::to_string).collect());
        return Some((
            "gs_10",
            Some(c[1].to_string()),
            Some(c[2].to_string()),
            split,
        ));
    }
    if code.len() == 29
        && let Some(c) = SHORT.captures(code)
    {
        return Some((
            "short",
            Some(c[1].to_string()),
            Some(c[2].to_string()),
            None,
        ));
    }
    if code.len() == 68
        && let Some(c) = EGAIS20.captures(code)
    {
        return Some(("egais_20", None, Some(c[2].to_string()), None));
    }
    if code.len() == 150
        && let Some(c) = EGAIS30.captures(code)
    {
        return Some(("egais_30", Some(c[1].to_string()), None, None));
    }
    if code.len() == 14
        && let Some(c) = ITF14.captures(code)
    {
        return Some(("itf_14", Some(c[1].to_string()), None, None));
    }
    if code.len() == 13
        && let Some(c) = EAN13.captures(code)
    {
        return Some(("ean_13", Some(c[1].to_string()), None, None));
    }
    if code.len() == 8
        && let Some(c) = EAN8.captures(code)
    {
        return Some(("ean_8", Some(c[1].to_string()), None, None));
    }
    if code.len() == 20
        && let Some(c) = FUR.captures(code)
    {
        return Some(("fur", Some(c[1].to_string()), None, None));
    }
    None
}

fn num_to_hex(number: &str) -> String {
    let value = number.parse::<u64>().unwrap_or(0);
    format!("{value:012x}")
}

fn str_to_hex(s: &str) -> String {
    s.chars().map(|c| format!("{:02x}", c as u32)).collect()
}

fn chunk(s: String) -> String {
    let chars: Vec<char> = s.chars().collect();
    chars
        .chunks(2)
        .map(|c| c.iter().collect::<String>())
        .collect::<Vec<_>>()
        .join(" ")
        .to_uppercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Values verified against the official Python SDK's test suite.
    #[test]
    fn gs10_result() {
        let code = ProductCode::new("010460406000590021N4N57RTCBUZTQ");
        assert_eq!(code.type_, "gs_10");
        assert_eq!(
            code.product_code,
            "44 4D 04 2F F7 5C 76 0C 4E 34 4E 35 37 52 54 43 42 55 5A 54 51"
        );
        assert_eq!(code.mark_code_value, "010460406000590021N4N57RTCBUZTQ");
    }

    #[test]
    fn gs1m_type_detection() {
        let code = ProductCode::new("0104860019002077215'u0E0AeTToec93L6Wj");
        assert_eq!(code.type_, "gs_1m");
    }

    #[test]
    fn ean13_result() {
        let code = ProductCode::new("4006670128002");
        assert_eq!(code.type_, "ean_13");
        assert_eq!(code.product_code, "45 0D 03 A4 E0 26 53 82");
        assert_eq!(code.mark_code_value, "4006670128002");
    }

    #[test]
    fn itf14_and_egais20_detection() {
        assert_eq!(ProductCode::new("01460123456789").type_, "itf_14");
        assert_eq!(
            ProductCode::new(
                "22N00002NU5DBKYDOT17ID980726019019608CW1A4XR5EJ7JKFX50FHHGV92ZR2GZRZ"
            )
            .type_,
            "egais_20"
        );
    }

    #[test]
    fn unknown_code_cuts_to_limit() {
        let code = ProductCode::new("some_unknown_code_123");
        assert_eq!(code.type_, "unknown");
        assert_eq!(code.mark_code_value, "some_unknown_code_123");
        let long = ProductCode::new(&"a".repeat(40));
        assert_eq!(long.mark_code_value.len(), 32);
        assert_eq!(long.product_code.split(' ').count(), 32); // 0000 + 30 bytes
    }

    #[test]
    fn empty_code_is_invalid() {
        let code = ProductCode::new("");
        assert_eq!(code.type_, "unknown");
        assert_eq!(code.product_code, "");
    }
}
