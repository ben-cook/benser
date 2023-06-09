use std::collections::HashMap;

use crate::{
    dom::{AttrMap, Node},
    encoding::{Confidence, Encoding},
    parse_state::ParseState,
};

pub struct Parser<'a> {
    // Properties from the specification
    encoding: Encoding,
    confidence: Confidence,
    parse_state: ParseState<'a>,
    /// Parsers have a script nesting level, which must be initially set to zero.
    ///
    /// https://html.spec.whatwg.org/multipage/parsing.html#overview-of-the-parsing-model
    script_nesting_level: u32,
    /// Parsers have a parser pause flag, which must be initially set to false.
    ///
    /// https://html.spec.whatwg.org/multipage/parsing.html#overview-of-the-parsing-model
    pause_flag: bool,

    // Properties for internal use
    input: String,
    pos: usize,
}

impl Parser<'_> {
    pub fn from_string(input: String) -> Self {
        Parser {
            input,
            pos: 0,
            encoding: Encoding::Utf8,
            confidence: Confidence::Certain,
            parse_state: ParseState::default(),
            script_nesting_level: 0,
            pause_flag: false,
        }
    }

    pub fn from_bytes_utf8(input: Vec<u8>) -> Self {
        Parser {
            input: String::from_utf8(input).unwrap(),
            pos: 0,
            encoding: Encoding::Utf8,
            confidence: Confidence::Certain,
            parse_state: ParseState::default(),
            script_nesting_level: 0,
            pause_flag: false,
        }
    }

    pub fn from_bytes_utf16(input: Vec<u16>) -> Self {
        Parser {
            input: String::from_utf16(input.as_slice()).unwrap(),
            pos: 0,
            encoding: Encoding::Utf16,
            confidence: Confidence::Certain,
            parse_state: ParseState::default(),
            script_nesting_level: 0,
            pause_flag: false,
        }
    }

    /// Parse an HTML document and return the root element.
    pub fn run(&mut self) -> Node {
        // https://html.spec.whatwg.org/multipage/parsing.html#overview-of-the-parsing-model

        let mut nodes = self.parse_nodes();

        // If the document contains a root element, just return it. Otherwise, create one.
        if nodes.len() == 1 {
            nodes.swap_remove(0)
        } else {
            Node::elem("html".to_string(), HashMap::new(), nodes)
        }
    }

    /// Read the current character without consuming it.
    fn next_char(&self) -> char {
        self.input[self.pos..].chars().next().unwrap()
    }

    /// Do the next characters start with the given string?
    fn starts_with(&self, s: &str) -> bool {
        self.input[self.pos..].starts_with(s)
    }

    /// Return true if all input is consumed.
    fn eof(&self) -> bool {
        self.pos >= self.input.len()
    }

    /// Return the current character, and advance self.pos to the next character.
    fn consume_char(&mut self) -> char {
        let mut iter = self.input[self.pos..].char_indices();
        let (_, cur_char) = iter.next().unwrap();
        let (next_pos, _) = iter.next().unwrap_or((1, ' '));
        self.pos += next_pos;
        cur_char
    }

    /// Consume characters until `test` returns false.
    fn consume_while<F>(&mut self, test: F) -> String
    where
        F: Fn(char) -> bool,
    {
        let mut result = String::new();
        while !self.eof() && test(self.next_char()) {
            result.push(self.consume_char());
        }
        result
    }

    /// Consume and discard zero or more whitespace characters.
    fn consume_whitespace(&mut self) {
        self.consume_while(char::is_whitespace);
    }

    /// Parse a tag or attribute name.
    fn parse_tag_name(&mut self) -> String {
        self.consume_while(|c| matches!(c, 'a'..='z' | 'A'..='Z' | '0'..='9'))
    }

    /// Parse a single node.
    fn parse_node(&mut self) -> Node {
        match self.next_char() {
            '<' => self.parse_element(),
            _ => self.parse_text(),
        }
    }

    /// Parse a text node.
    fn parse_text(&mut self) -> Node {
        Node::text(self.consume_while(|c| c != '<'))
    }

    /// Parse a single element, including its open tag, contents, and closing tag.
    fn parse_element(&mut self) -> Node {
        // Opening tag.
        assert!(self.consume_char() == '<');
        let tag_name = self.parse_tag_name();
        let attrs = self.parse_attributes();
        assert!(self.consume_char() == '>');

        // Contents.
        let children = self.parse_nodes();

        // Closing tag.
        assert!(self.consume_char() == '<');
        assert!(self.consume_char() == '/');
        assert!(self.parse_tag_name() == tag_name);
        assert!(self.consume_char() == '>');

        Node::elem(tag_name, attrs, children)
    }

    /// Parse a single name="value" pair.
    fn parse_attr(&mut self) -> (String, String) {
        let name = self.parse_tag_name();
        assert!(self.consume_char() == '=');
        let value = self.parse_attr_value();
        (name, value)
    }

    /// Parse a quoted value.
    fn parse_attr_value(&mut self) -> String {
        let open_quote = self.consume_char();
        assert!(open_quote == '"' || open_quote == '\'');
        let value = self.consume_while(|c| c != open_quote);
        assert!(self.consume_char() == open_quote);
        value
    }

    /// Parse a list of name="value" pairs, separated by whitespace.
    fn parse_attributes(&mut self) -> AttrMap {
        let mut attributes = HashMap::new();
        loop {
            self.consume_whitespace();
            if self.next_char() == '>' {
                break;
            }
            let (name, value) = self.parse_attr();
            attributes.insert(name, value);
        }
        attributes
    }

    /// Parse a sequence of sibling nodes.
    fn parse_nodes(&mut self) -> Vec<Node> {
        let mut nodes = Vec::new();
        loop {
            self.consume_whitespace();
            if self.eof() || self.starts_with("</") {
                break;
            }
            nodes.push(self.parse_node());
        }
        nodes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_tests() {
        assert_eq!(
            Parser::from_string("<div></div>".to_string()).run(),
            Node::elem("div".to_string(), HashMap::new(), Vec::new())
        );

        assert_eq!(
            Parser::from_string("<html><body>Hello, world!</body></html>".to_string()).run(),
            Node::elem(
                "html".to_string(),
                HashMap::new(),
                vec![Node::elem(
                    "body".to_string(),
                    HashMap::new(),
                    vec![Node::text("Hello, world!".to_string())]
                )]
            )
        );
    }

    #[test]
    fn attributes() {
        let mut attribute_map = HashMap::new();
        attribute_map.insert("height".to_string(), "3".to_string());
        attribute_map.insert("width".to_string(), "100%".to_string());

        assert_eq!(
            Parser::from_string(r#"<div height="3" width="100%"></div>"#.to_string()).run(),
            Node::elem("div".to_string(), attribute_map, Vec::new())
        );
    }

    #[test]
    fn adds_root_node() {
        assert_eq!(
            Parser::from_string("<h1>Heading 1</h1> <h2>Heading 2</h2>".to_string()).run(),
            Node::elem(
                "html".to_string(),
                HashMap::new(),
                vec![
                    Node::elem(
                        "h1".to_string(),
                        HashMap::new(),
                        vec![Node::text("Heading 1".to_string())]
                    ),
                    Node::elem(
                        "h2".to_string(),
                        HashMap::new(),
                        vec![Node::text("Heading 2".to_string())]
                    )
                ]
            )
        );
    }
}
