use core::panic;

use itertools::Itertools;

pub struct Html(HtmlNode);

enum HtmlNode {
    Element {
        tag: String,
        children: Vec<HtmlNode>,
        attributes: Vec<(String, String)>,
        void: bool,
    },
    Raw(String),
    Document(Vec<HtmlNode>),
}

impl HtmlNode {
    fn new(tag: &str) -> Self {
        Self::Element {
            tag: tag.to_string(),
            children: vec![],
            attributes: vec![],
            void: false,
        }
    }

    fn new_void(tag: &str) -> Self {
        Self::Element {
            tag: tag.to_string(),
            children: vec![],
            attributes: vec![],
            void: true,
        }
    }

    fn document() -> Self {
        Self::Document(vec![])
    }

    fn push_child(&mut self, child: HtmlNode) {
        match self {
            Self::Element {
                tag: _,
                children,
                attributes: _,
                void: _,
            } => {
                children.push(child);
            }
            Self::Document(children) => {
                children.push(child);
            }
            _ => panic!(),
        }
    }

    fn push_text(&mut self, text: String) {
        self.push_child(HtmlNode::Raw(HtmlNode::escape(text)));
    }
    fn push_raw(&mut self, text: String) {
        self.push_child(HtmlNode::Raw(text));
    }

    fn escape(s: String) -> String {
        html_escape::encode_text(&s).to_string()
    }

    fn render(&self, pretty: bool, indent_level: usize) -> String {
        let indent = if pretty {
            " ".repeat(indent_level * 2)
        } else {
            "".to_string()
        };
        match self {
            HtmlNode::Element {
                tag,
                children,
                attributes,
                void,
            } => {
                let attribute_list = if attributes.is_empty() {
                    "".to_string()
                } else {
                    attributes
                        .iter()
                        .map(|a| format!(" {}=\"{}\"", a.0, a.1))
                        .collect::<String>()
                };
                if *void {
                    return format!("{indent}<{tag}{attribute_list} />");
                }
                let inner = children
                    .iter()
                    .map(|c| c.render(pretty, indent_level + 1))
                    .join("\n");

                if pretty {
                    let inner_nl = if inner.is_empty() { "" } else { "\n" };
                    let inner_indent = if inner.is_empty() {
                        ""
                    } else {
                        indent.as_str()
                    };
                    format!(
                        "{indent}<{tag}{attribute_list}>{inner_nl}{inner}{inner_nl}{inner_indent}</{tag}>"
                    )
                } else {
                    format!("<{tag}{attribute_list}>{inner}</{tag}>")
                }
            }
            HtmlNode::Raw(s) => format!("{indent}{s}"),
            HtmlNode::Document(children) => children.iter().map(|c| c.render(pretty, 0)).join("\n"),
        }
    }

    fn set_attribute(&mut self, k: String, v: String, push: bool) {
        if let Self::Element {
            tag: _,
            children: _,
            attributes,
            void: _,
        } = self
        {
            if let Some(attr) = attributes.iter_mut().find(|a| a.0 == k) {
                if push {
                    attr.1.push(' ');
                    attr.1.push_str(v.as_str());
                } else {
                    attr.1 = v;
                }
            } else {
                attributes.push((k, v));
            }
        } else {
            panic!()
        }
    }
}

#[allow(dead_code)]
impl Html {
    pub fn new(tag: &str) -> Self {
        Self(HtmlNode::new(tag))
    }

    pub fn div_with_class(class: &str) -> Self {
        let elem = Self::new("div").with_class(class);
        elem
    }

    pub fn new_void(tag: &str) -> Self {
        Self(HtmlNode::new_void(tag))
    }

    pub fn document() -> Self {
        Self(HtmlNode::document())
    }

    pub fn push_child(&mut self, child: Html) {
        self.0.push_child(child.0);
    }

    pub fn div_with_class_and_text(class: &str, text: String) -> Html {
        Self::div_with_class(class).with_text(&text)
    }
    
    pub fn push_child_div_with_class_and_text(&mut self, child_class: &str, child_text: String) {
        let child = Self::div_with_class(child_class).with_text(&child_text);
        self.push_child(child);
    }

    pub fn with_child(mut self, child: Html) -> Self {
        self.push_child(child);
        self
    }

    pub fn push_text(&mut self, text: String) {
        self.0.push_text(text);
    }

    pub fn with_text(mut self, text: &str) -> Self {
        self.push_text(text.to_string());
        self
    }

    pub fn with_string(self, text: String) -> Self {
        self.with_text(&text)
    }

    pub fn push_raw(&mut self, text: &str) {
        self.0.push_raw(text.to_string());
    }

    pub fn with_raw(mut self, text: &str) -> Self {
        self.push_raw(text);
        self
    }

    pub fn set_attribute(&mut self, k: &str, v: &str) {
        self.0.set_attribute(k.to_string(), v.to_string(), false);
    }

    pub fn push_attribute(&mut self, k: &str, v: &str) {
        self.0.set_attribute(k.to_string(), v.to_string(), true);
    }

    pub fn with_attribute(mut self, k: &str, v: &str) -> Self {
        self.set_attribute(k, v);
        self
    }

    pub fn set_class(mut self, c: &str) {
        self.set_attribute("class", c);
    }
    pub fn with_class(mut self, c: &str) -> Self {
        self.push_attribute("class", c);
        self
    }

    pub fn render(&self) -> String {
        self.0.render(false, 0)
    }

    pub fn pretty(&self) -> String {
        self.0.render(true, 0)
    }
}

impl From<&Html> for Html {
    fn from(val: &Html) -> Self {
        val.into()
    }
}
