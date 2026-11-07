//! XML attribute extraction (port of `util.extract_xml_attr`).

use std::collections::HashMap;

use crate::error::{Error, Result};

const ROOT_ELEMENT: &str = "IPMI";

/// Attributes of a single XML element.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Attrs {
    element: String,
    attrs: HashMap<String, String>,
}

impl Attrs {
    /// Fetch a required attribute.
    pub(crate) fn get(&self, attribute: &'static str) -> Result<&str> {
        self.attrs
            .get(attribute)
            .map(String::as_str)
            .ok_or_else(|| Error::MissingAttribute {
                element: self.element.clone(),
                attribute,
            })
    }
}

/// Collect the attributes of every descendant element named `tag`
/// (equivalent to ElementTree's `findall(".//TAG")`).
///
/// The document element must be `<IPMI>`: anything else (typically the
/// BMC's HTML login page) is reported as [`Error::UnexpectedResponse`].
/// DTDs are rejected, so entity-expansion attacks are not possible.
pub(crate) fn extract(xml: &str, tag: &str) -> Result<Vec<Attrs>> {
    let doc = roxmltree::Document::parse(xml.trim())?;
    let root = doc.root_element().tag_name().name();
    if root != ROOT_ELEMENT {
        return Err(Error::UnexpectedResponse(root.to_owned()));
    }
    Ok(doc
        .root_element()
        .descendants()
        .skip(1) // `.//` excludes the context node itself
        .filter(|node| node.is_element() && node.tag_name().name() == tag)
        .map(|node| Attrs {
            element: tag.to_owned(),
            attrs: node
                .attributes()
                .map(|a| (a.name().to_owned(), a.value().to_owned()))
                .collect(),
        })
        .collect())
}

#[cfg(test)]
pub(crate) fn attrs(element: &str, pairs: &[(&str, &str)]) -> Attrs {
    Attrs {
        element: element.to_owned(),
        attrs: pairs
            .iter()
            .map(|(k, v)| ((*k).to_owned(), (*v).to_owned()))
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture(name: &str) -> String {
        let path = format!("{}/tests/fixtures/{name}.xml", env!("CARGO_MANIFEST_DIR"));
        std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"))
    }

    #[test]
    fn extract_counts() {
        for (file, tag, expected) in [
            ("ipmi_response_sensors", "SENSOR", 28),
            ("ipmi_response_sensors", "NOT_A_SENSOR", 0),
            ("ipmi_response_sel", "SEL", 6),
            ("ipmi_response_pmbus", "PSItem", 4),
            ("ipmi_response_pmbus", "PSInfo", 1),
        ] {
            assert_eq!(
                extract(&fixture(file), tag).unwrap().len(),
                expected,
                "{file} {tag}"
            );
        }
    }

    #[test]
    fn rejects_non_ipmi_documents() {
        let err = extract("<html><body/></html>", "SENSOR").unwrap_err();
        assert!(
            matches!(&err, Error::UnexpectedResponse(root) if root == "html"),
            "{err:?}"
        );
    }

    #[test]
    fn root_element_is_not_matched() {
        assert!(extract("<IPMI/>", "IPMI").unwrap().is_empty());
    }

    #[test]
    fn missing_attribute_is_reported() {
        let items = extract("<IPMI><B x=\"1\"/></IPMI>", "B").unwrap();
        assert_eq!(items[0].get("x").unwrap(), "1");
        let err = items[0].get("y").unwrap_err();
        assert_eq!(err.to_string(), "<B> is missing the `y` attribute");
    }

    #[test]
    fn rejects_malformed_and_dtd() {
        assert!(matches!(extract("<IPMI>", "A"), Err(Error::Xml(_))));
        let bomb = r#"<?xml version="1.0"?><!DOCTYPE l [<!ENTITY a "aaaa">]><l>&a;</l>"#;
        assert!(matches!(extract(bomb, "l"), Err(Error::Xml(_))));
    }
}
