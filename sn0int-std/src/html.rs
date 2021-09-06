use crate::engine::structs::LuaMap;
use crate::errors::*;
use crate::hlua::AnyLuaValue;
use kuchiki::traits::TendrilSink;
use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub struct Element {
    pub attrs: HashMap<String, String>,
    pub text: String,
    pub html: String,
}

impl From<Element> for AnyLuaValue {
    fn from(elem: Element) -> AnyLuaValue {
        let mut map = LuaMap::new();

        map.insert_str("text", elem.text);
        map.insert("attrs", LuaMap::from(elem.attrs));
        map.insert_str("html", elem.html);

        map.into()
    }
}

fn transform_element(entry: &kuchiki::NodeDataRef<kuchiki::ElementData>) -> Element {
    let text = entry.text_contents();
    let as_node = entry.as_node();

    let mut attrs: HashMap<String, String> = HashMap::new();

    if let Some(element) = as_node.as_element() {
        for (k, v) in &element.attributes.borrow().map {
            attrs.insert(k.local.to_string(), v.value.clone());
        }
    }

    let mut html = Vec::new();
    let html = match as_node.serialize(&mut html) {
        Ok(_) => String::from_utf8_lossy(&html).to_string(),
        Err(_) => {
            debug!("html serialize failed");
            String::new()
        },
    };

    Element {
        attrs,
        text,
        html,
    }
}

pub fn html_select(html: &str, selector: &str) -> Result<Element> {
    let doc = kuchiki::parse_html().one(html);
    match doc.select_first(selector) {
        Ok(x) => Ok(transform_element(&x)),
        Err(_) => bail!("css selector failed"),
    }
}

pub fn html_select_list(html: &str, selector: &str) -> Result<Vec<Element>> {
    let doc = kuchiki::parse_html().one(html);

    match doc.select(selector) {
        Ok(x) => Ok(x.map(|x| transform_element(&x)).collect()),
        Err(_) => bail!("css selector failed"),
    }
}

pub fn html_form(html: &str) -> Result<HashMap<String, String>> {
    let inputs = html_select_list(html, "input")?;

    let mut form = HashMap::new();

    for input in inputs {
        let name = match input.attrs.get("name") {
            Some(x) => x.to_string(),
            None => continue,
        };

        let value = input.attrs.get("value").map(|x| x.to_string());

        let value = match input.attrs.get("type").map(|x| x.as_str()) {
            Some("hidden") => value,
            Some("submit") => value,
            _ => continue,
        };

        if let Some(value) = value {
            form.insert(name, value);
        }
    }

    Ok(form)
}


#[cfg(test)]
mod tests {
    use super::*;
    use maplit::hashmap;

    #[test]
    fn test_html_select() {
        let elems = html_select(r#"<html><div id="yey">content</div></html>"#, "#yey").unwrap();
        assert_eq!(elems,
            Element {
                attrs: hashmap!{
                    "id".into() => "yey".into(),
                },
                text: "content".into(),
                html: r#"<div id="yey">content</div>"#.into(),
            }
        );
    }

    #[test]
    fn test_html_select_list() {
        let elems = html_select_list(r#"<html><div id="yey">content</div></html>"#, "#yey").unwrap();
        assert_eq!(elems, vec![
            Element {
                attrs: hashmap!{
                    "id".into() => "yey".into(),
                },
                text: "content".into(),
                html: r#"<div id="yey">content</div>"#.into(),
            }
        ]);
    }
}
