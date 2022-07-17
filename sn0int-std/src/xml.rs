use crate::errors::*;

use crate::json::LuaJsonValue;
use crate::hlua::AnyLuaValue;
use serde::Serialize;
use std::collections::HashMap;
use xml::attribute::OwnedAttribute;
use xml::name::OwnedName;
use xml::reader::{EventReader, ParserConfig, XmlEvent};


#[derive(Debug, PartialEq, Serialize)]
pub struct XmlDocument {
    pub children: Vec<XmlElement>,
}

impl Default for XmlDocument {
    fn default() -> Self {
        Self::new()
    }
}

impl XmlDocument {
    #[inline(always)]
    pub fn new() -> XmlDocument {
        XmlDocument {
            children: Vec::new(),
        }
    }
}

#[derive(Debug, PartialEq, Clone, Serialize)]
pub struct XmlElement {
    pub name: String,
    pub attrs: HashMap<String, String>,
    pub text: Option<String>,
    pub children: Vec<XmlElement>,
}

impl XmlElement {
    #[inline]
    fn from(name: OwnedName, attributes: Vec<OwnedAttribute>) -> XmlElement {
        let name = name.local_name;
        let attrs = attributes.into_iter()
            .map(|attr| (attr.name.local_name, attr.value))
            .collect();
        XmlElement {
            name,
            attrs,
            text: None,
            children: Vec::new(),
        }
    }
}

#[inline]
pub fn decode(x: &str) -> Result<AnyLuaValue> {
    let v = decode_raw(x)?;
    let v = serde_json::to_value(v)?;
    let v: LuaJsonValue = v.into();
    Ok(v.into())
}

#[inline]
fn append_text(stack: &mut [XmlElement], text: String) {
    if let Some(tail) = stack.last_mut() {
        if let Some(prev) = tail.text.as_mut() {
            prev.push_str(&text);
        } else {
            tail.text = Some(text);
        }
    }
}

fn decode_raw(x: &str) -> Result<XmlDocument> {
    let config = ParserConfig::new()
        .trim_whitespace(true)
        .whitespace_to_characters(true)
        .cdata_to_characters(true)
        .ignore_comments(true)
        .coalesce_characters(true);

    let parser = EventReader::new_with_config(x.as_bytes(), config);
    let mut stack = Vec::new();

    let mut doc = XmlDocument::new();

    for next in parser {
        let next = next?;
        debug!("xml element: {:?}", next);

        match next {
            XmlEvent::StartElement {
                name,
                attributes,
                ..
            } => {
                stack.push(XmlElement::from(name, attributes));
            },
            XmlEvent::EndElement {
                name,
            } => {
                let child = stack.pop()
                    .ok_or_else(|| format_err!("end element has no matching start element"))?;

                let name = name.local_name;
                if child.name != name {
                    bail!("end element name doesn't match start element name")
                }

                if let Some(tail) = stack.last_mut() {
                    tail.children.push(child);
                } else {
                    doc.children.push(child);
                }
            },
            XmlEvent::CData(text) => append_text(&mut stack, text),
            XmlEvent::Characters(text) => append_text(&mut stack, text),
            _ => (),
        }
    }

    // TODO: consider ignoring this?
    if !stack.is_empty() {
        bail!("end of document but still open elements remaining")
    }

    Ok(doc)
}

#[cfg(test)]
mod tests {
    use super::*;
    use maplit::hashmap;

    #[test]
    fn verify_xml_decode_empty() {
        let result = decode_raw("");
        assert!(result.is_err());
    }

    #[test]
    fn verify_xml_decode_empty_body() {
        let doc = decode_raw("<body></body>").unwrap();
        assert_eq!(doc, XmlDocument {
            children: vec![
                XmlElement {
                    name: String::from("body"),
                    attrs: HashMap::new(),
                    text: None,
                    children: vec![],
                }
            ]
        });
    }

    #[test]
    fn verify_xml_decode_single_tag() {
        let doc = decode_raw("<body><foo x=\"1\" /></body>").unwrap();
        assert_eq!(doc, XmlDocument {
            children: vec![
                XmlElement {
                    name: String::from("body"),
                    attrs: HashMap::new(),
                    text: None,
                    children: vec![
                        XmlElement {
                            name: String::from("foo"),
                            attrs: hashmap!{
                                String::from("x") => String::from("1"),
                            },
                            text: None,
                            children: vec![],
                        }
                    ],
                }
            ]
        });
    }

    #[test]
    fn verify_xml_decode_single_tag_text() {
        let doc = decode_raw("<body><foo x=\"1\">hello world</foo></body>").unwrap();
        assert_eq!(doc, XmlDocument {
            children: vec![
                XmlElement {
                    name: String::from("body"),
                    attrs: HashMap::new(),
                    text: None,
                    children: vec![
                        XmlElement {
                            name: String::from("foo"),
                            attrs: hashmap!{
                                String::from("x") => String::from("1"),
                            },
                            text: Some(String::from("hello world")),
                            children: vec![],
                        }
                    ],
                }
            ]
        });
    }
}
