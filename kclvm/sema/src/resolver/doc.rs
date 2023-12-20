use kclvm_ast::ast::SchemaStmt;
use pcre2::bytes::Regex;
use std::collections::{HashMap, HashSet};
use std::iter::Iterator;
use std::str;

lazy_static::lazy_static! {
    static ref RE: Regex = Regex::new(r#"(?s)^(['\"]{3})(.*?)(['\"]{3})$"#).unwrap();
}

/// strip leading and trailing triple quotes from the original docstring content
fn strip_quotes(original: &mut String) {
    let quote = original.chars().next().unwrap();
    if quote != '"' && quote != '\'' {
        return;
    }
    if let Ok(Some(mat)) = RE.find(original.as_bytes()) {
        let content = str::from_utf8(&original.as_bytes()[mat.start() + 3..mat.end() - 3])
            .unwrap()
            .to_owned();
        *original = content;
    }
}

fn expand_tabs(s: &str, spaces_per_tab: usize) -> String {
    s.replace("\t", &" ".repeat(spaces_per_tab))
}

/// Clean up indentation by removing any common leading whitespace on all lines after the first line.
fn clean_doc(doc: &mut String) {
    let tab_expanded = expand_tabs(&doc, 4);
    let mut lines: Vec<&str> = tab_expanded.split('\n').collect();
    // Find minimum indentation of any non-blank lines after first line.
    // Skip first line since it's not indented.
    if !lines.is_empty() {
        let margin = lines[1..] // skip first line
            .iter()
            .filter(|line| !line.trim().is_empty()) // skip empty lines
            .map(|line| line.chars().take_while(|c| c.is_whitespace()).count()) // count leading whitespaces of each line
            .min() // return the minimum indentation
            .unwrap_or(0);

        lines[1..].iter_mut().for_each(|line| {
            *line = if line.trim().len() > 0 {
                if let Some(sub) = line.get(margin..) {
                    sub
                } else {
                    line.trim()
                }
            } else {
                line.trim()
            }; // remove command indentation
        });

        // Remove trailing and leading blank lines.
        while !lines.is_empty() && lines.last().unwrap().trim().is_empty() {
            lines.pop();
        }
        while !lines.is_empty() && lines[0].trim().is_empty() {
            lines.remove(0);
        }
    }
    *doc = lines.join("\n");
}

/// A line-based string reader.
struct Reader {
    data: Vec<String>,
    l: usize,
}

impl Reader {
    fn new(data: String) -> Self {
        let data_vec: Vec<String> = data.split('\n').map(|s| s.to_string()).collect();
        Self {
            data: data_vec,
            l: 0,
        }
    }
    fn reset(&mut self) {
        self.l = 0;
    }

    fn read(&mut self) -> String {
        if !self.eof() {
            let out = self.data[self.l].clone();
            self.l += 1;
            return out;
        } else {
            return "".to_string();
        }
    }

    fn seek_next_non_empty_line(&mut self) {
        for l in self.data[self.l..].iter() {
            if l.trim().len() > 0 {
                break;
            } else {
                self.l += 1;
            }
        }
    }

    fn eof(&self) -> bool {
        self.l >= self.data.len()
    }

    fn read_to_condition(&mut self, condition_func: &dyn Fn(&str) -> bool) -> Vec<String> {
        let start = self.l;
        for line in self.data[start..].iter() {
            if condition_func(line) {
                return self.data[start..self.l].to_vec();
            }
            self.l += 1;
            if self.eof() {
                return self.data[start..self.l].to_vec();
            }
        }
        return vec![];
    }

    fn read_to_next_empty_line(&mut self) -> Vec<String> {
        self.seek_next_non_empty_line();

        fn is_empty(line: &str) -> bool {
            return line.trim().len() == 0;
        }

        return self.read_to_condition(&is_empty);
    }

    fn read_to_next_unindented_line(&mut self) -> Vec<String> {
        fn is_unindented(line: &str) -> bool {
            return line.trim().len() > 0 && line.trim_start().len() == line.len();
        }

        return self.read_to_condition(&is_unindented);
    }

    fn peek(&self, n: usize, positive: bool) -> String {
        if positive {
            if self.l + n < self.data.len() {
                return self.data[self.l + n].clone();
            } else {
                return "".to_string();
            }
        } else {
            if self.l >= n {
                return self.data[self.l - n].clone();
            } else {
                return "".to_string();
            }
        }
    }

    fn _is_empty(&self) -> bool {
        return self.data.iter().all(|x| x.trim().len() == 0);
    }
}

/// remove the leading and trailing empty lines
fn _strip(doc: Vec<String>) -> Vec<String> {
    let mut i = 0;
    let mut j = 0;
    for (line_num, line) in doc.iter().enumerate() {
        if !line.trim().is_empty() {
            i = line_num;
            break;
        }
    }

    for (line_num, line) in doc.iter().enumerate().rev() {
        if !line.trim().is_empty() {
            j = line_num;
            break;
        }
    }

    doc[i..j + 1].to_vec()
}

/// Checks if current line is at the beginning of a section
fn is_at_section(doc: &mut Reader) -> bool {
    doc.seek_next_non_empty_line();
    if doc.eof() {
        return false;
    }
    let l1 = doc.peek(0, true);
    let l1 = l1.trim();
    let l2 = doc.peek(1, true);
    let l2 = l2.trim(); // ---------- or ==========
    let l2_char_set = l2.chars().collect::<HashSet<char>>();

    if l2.len() >= 3
        && l2_char_set.len() == 1
        && (l2.contains('-') || l2.contains('='))
        && l1.len() != l1.len()
    {
        // todo: when line2 is conformed with "-" or "=", but the number of the "-/=" mismatch the section title length, mark as a section and return a warning
        return false;
    }
    l2.starts_with(&"-".repeat(l1.len())) || l2.starts_with(&"=".repeat(l1.len()))
}

/// read lines before next section beginning, continuous empty lines will be merged to one
fn read_to_next_section(doc: &mut Reader) -> Vec<String> {
    let mut section = doc.read_to_next_empty_line();

    while !is_at_section(doc) && !doc.eof() {
        if doc.peek(1, false).trim().is_empty() {
            section.push(doc.peek(1, false));
        }
        section.append(&mut doc.read_to_next_empty_line());
    }
    section
}

/// parse the Attribute Section of the docstring to list of Attribute
fn parse_attr_list(content: String) -> Vec<Attribute> {
    let mut r = Reader::new(content);
    let mut attrs = vec![];
    while !r.eof() {
        let header = r.read();
        let header = header.trim();
        if header.contains(": ") {
            let parts: Vec<&str> = header.split(": ").collect();
            let arg_name = parts[0].trim();

            let desc_lines = r
                .read_to_next_unindented_line()
                .iter()
                .map(|s| s.trim().to_string())
                .collect();
            attrs.push(Attribute::new(arg_name.to_string(), desc_lines));
        } else {
            r.read_to_next_unindented_line();
        }
    }
    attrs
}

/// parse the summary of the schema. The final summary content will be a concat of lines in the original summary with whitespace.
fn parse_summary(doc: &mut Reader) -> String {
    if is_at_section(doc) {
        // no summary provided
        return "".to_string();
    }
    let lines = read_to_next_section(doc);
    lines
        .iter()
        .map(|s| s.trim())
        .collect::<Vec<_>>()
        .join(" ")
        .trim()
        .to_string()
}

/// parse the schema docstring to Doc.
/// The summary of the schema content will be concatenated to a single line string by whitespaces.
/// The description of each attribute will be returned as separate lines.
pub fn parse_doc_string(ori: &String) -> Doc {
    if ori.is_empty() {
        return Doc::new("".to_string(), vec![], HashMap::new());
    }
    let mut ori = ori.clone();
    strip_quotes(&mut ori);
    clean_doc(&mut ori);
    let mut doc = Reader::new(ori);
    doc.reset();
    let summary = parse_summary(&mut doc);

    let attr_section = read_to_next_section(&mut doc);

    let attr_content = attr_section.join("\n");

    let attrs = parse_attr_list(attr_content);

    let mut examples = HashMap::new();
    let example_section = read_to_next_section(&mut doc);
    if !example_section.is_empty() {
        let default_example_content = match example_section.len() {
            0 | 1 | 2 => "".to_string(),
            _ => example_section[2..].join("\n"),
        };
        examples.insert(
            "Default example".to_string(),
            Example::new("".to_string(), "".to_string(), default_example_content),
        );
    }
    Doc::new(summary, attrs, examples)
}

/// The Doc struct contains a summary of schema and all the attributes described in the the docstring.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Doc {
    pub summary: String,
    pub attrs: Vec<Attribute>,
    pub examples: HashMap<String, Example>,
}

impl Doc {
    pub fn new(summary: String, attrs: Vec<Attribute>, examples: HashMap<String, Example>) -> Self {
        Self {
            summary,
            attrs,
            examples,
        }
    }
    pub fn new_from_schema_stmt(schema: &SchemaStmt) -> Self {
        let attrs = schema
            .get_left_identifier_list()
            .iter()
            .map(|(_, _, attr_name)| attr_name.clone())
            .collect::<Vec<String>>();
        Self {
            summary: "".to_string(),
            attrs: attrs
                .iter()
                .map(|name| Attribute::new(name.clone(), vec![]))
                .collect(),
            examples: HashMap::new(),
        }
    }

    pub fn to_doc_string(self) -> String {
        let summary = self.summary;
        let attrs_string = self
            .attrs
            .iter()
            .map(|attr| format!("{}: {}", attr.name, attr.desc.join("\n")))
            .collect::<Vec<String>>()
            .join("\n");
        let examples_string = self
            .examples
            .values()
            .map(|example| {
                format!(
                    "{}\n{}\n{}",
                    example.summary, example.description, example.value
                )
            })
            .collect::<Vec<String>>()
            .join("\n");
        format!("{summary}\n\nAttributes\n----------\n{attrs_string}\n\nExamples\n--------{examples_string}\n")
    }
}

/// The Attribute struct contains the attribute name and the corresponding description.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Attribute {
    pub name: String,
    pub desc: Vec<String>,
}

impl Attribute {
    fn new(name: String, desc: Vec<String>) -> Self {
        Self { name, desc }
    }
}

/// The Example struct contains the example summary and the literal content
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Example {
    pub summary: String,
    pub description: String,
    pub value: String,
}

impl Example {
    fn new(summary: String, description: String, value: String) -> Self {
        Self {
            summary,
            description,
            value,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{clean_doc, is_at_section, read_to_next_section, strip_quotes, Reader};
    use crate::resolver::doc::{parse_doc_string, Example};
    use std::fs::File;
    use std::io::prelude::*;
    use std::path::PathBuf;

    fn read_doc_content() -> String {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("src/resolver/test_data/doc.txt");
        let mut file = File::open(path).expect("Unable to open file");

        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .expect("Unable to read file");

        if cfg!(windows) {
            contents = contents.replace("\r\n", "\n")
        }
        contents
    }

    #[test]
    fn test_strip_quotes() {
        let ori_from_file = read_doc_content();

        let oris = [
            r#""""abcde""""#,
            r#"'''abc
de'''"#,
            ori_from_file.as_str(),
        ];
        let results = [
            "abcde",
            "abc
de",
            r#"
    Server is the common user interface for long-running
    services adopting the best practice of Kubernetes.

    Attributes
    ----------
    workloadType : str, default is "Deployment", required
        Use this attribute to specify which kind of long-running service you want.
        Valid values: Deployment, CafeDeployment.
        See also: kusion_models/core/v1/workload_metadata.k.
    name : str, required
        A Server-level attribute.
        The name of the long-running service.
        See also: kusion_models/core/v1/metadata.k.
    labels: {str:str}, optional
        A Server-level attribute.
        The labels of the long-running service.
        See also: kusion_models/core/v1/metadata.k.
  
    Examples
    ----------------------
    myCustomApp = AppConfiguration {
        name = "componentName"
    }
    "#,
        ];

        for (ori, res) in oris.iter().zip(results.iter()) {
            let from = &mut ori.to_string();
            strip_quotes(from);
            assert_eq!(from.to_string(), res.to_string());
        }
    }

    #[test]
    fn test_clean_doc() {
        let mut ori = read_doc_content();
        strip_quotes(&mut ori);
        clean_doc(&mut ori);
        let expect_cleaned = r#"Server is the common user interface for long-running
services adopting the best practice of Kubernetes.

Attributes
----------
workloadType : str, default is "Deployment", required
    Use this attribute to specify which kind of long-running service you want.
    Valid values: Deployment, CafeDeployment.
    See also: kusion_models/core/v1/workload_metadata.k.
name : str, required
    A Server-level attribute.
    The name of the long-running service.
    See also: kusion_models/core/v1/metadata.k.
labels: {str:str}, optional
    A Server-level attribute.
    The labels of the long-running service.
    See also: kusion_models/core/v1/metadata.k.

Examples
----------------------
myCustomApp = AppConfiguration {
    name = "componentName"
}"#;
        assert_eq!(ori.to_string(), expect_cleaned.to_string());
    }

    #[test]
    fn test_seek_next_non_empty_line() {
        let data = "line1
    line2

    
    line3

    line4

    ";
        let mut reader = Reader::new(data.to_string());

        // Test initial position
        assert_eq!(reader.l, 0);

        // Test seek to next non-empty line
        reader.seek_next_non_empty_line();
        assert_eq!(reader.l, 0); // line1
        assert_eq!(reader.read(), "line1");
        reader.seek_next_non_empty_line();
        assert_eq!(reader.l, 1); // line2
        assert_eq!(reader.read(), "    line2");
        reader.seek_next_non_empty_line();
        assert_eq!(reader.l, 4); // line3
        assert_eq!(reader.read(), "    line3");
        reader.seek_next_non_empty_line();
        assert_eq!(reader.l, 6); // line4
        assert_eq!(reader.read(), "    line4");
        // Test seek at the end of the data
        reader.seek_next_non_empty_line();
        assert_eq!(reader.l, 9); // end of data
        assert_eq!(reader.read(), "");
        assert!(reader.eof());
    }

    #[test]
    fn test_read_to_next_empty_line() {
        let data = "hello
    world

    foo
    bar

abc
        ";
        let mut reader = Reader::new(data.to_string());

        let output = reader.read_to_next_empty_line();
        assert_eq!(output, vec!["hello", "    world"]);

        let output = reader.read_to_next_empty_line();
        assert_eq!(output, vec!["    foo", "    bar"]);

        let output = reader.read_to_next_empty_line();
        assert_eq!(output, vec!["abc"]);

        let output = reader.read_to_next_empty_line();
        assert_eq!(output.len(), 0);
    }

    #[test]
    fn test_read_to_next_unindented_line() {
        let data = "
    indented line
    indented line
        indented line
    indented line

unindented line
        ";
        let mut reader = Reader::new(data.to_string());
        let result = reader.read_to_next_unindented_line();
        assert_eq!(
            result,
            vec![
                "",
                "    indented line",
                "    indented line",
                "        indented line",
                "    indented line",
                ""
            ]
        );
    }

    #[test]
    fn test_at_section() {
        let mut data = "Summary
    Attribute
    ---------
    description"
            .to_string();

        clean_doc(&mut data);

        let mut doc = Reader::new(data);
        assert!(!is_at_section(&mut doc));

        assert_eq!(doc.read(), "Summary");
        assert!(is_at_section(&mut doc));

        assert_eq!(doc.read(), "Attribute");
        assert!(!is_at_section(&mut doc));
    }

    #[test]
    fn test_read_to_next_section() {
        let mut data = "Summary
    

    SummaryContinue


    Attribute
    ---------
    attr1
        description
    
    attr2
        description
    
    Example
    -------
    content
    
    content
    
    See Also
    --------
    content"
            .to_string();
        clean_doc(&mut data);

        let mut doc = Reader::new(data);
        assert_eq!(
            read_to_next_section(&mut doc),
            vec!["Summary", "", "SummaryContinue"]
        );
    }

    #[test]
    fn test_parse_doc() {
        let mut content = read_doc_content();
        let doc = parse_doc_string(&mut content);
        assert_eq!(
            doc.summary,
            "Server is the common user interface for long-running services adopting the best practice of Kubernetes."
        );

        assert_eq!(doc.attrs.len(), 3);
        assert_eq!(doc.attrs[0].name, "workloadType".to_string());
        assert_eq!(
            doc.attrs[0].desc,
            vec![
                "Use this attribute to specify which kind of long-running service you want."
                    .to_string(),
                "Valid values: Deployment, CafeDeployment.".to_string(),
                "See also: kusion_models/core/v1/workload_metadata.k.".to_string()
            ]
        );

        assert_eq!(doc.attrs[1].name, "name".to_string());
        assert_eq!(
            doc.attrs[1].desc,
            vec![
                "A Server-level attribute.".to_string(),
                "The name of the long-running service.".to_string(),
                "See also: kusion_models/core/v1/metadata.k.".to_string(),
            ]
        );

        assert_eq!(doc.attrs[2].name, "labels".to_string());
        assert_eq!(
            doc.attrs[2].desc,
            vec![
                "A Server-level attribute.".to_string(),
                "The labels of the long-running service.".to_string(),
                "See also: kusion_models/core/v1/metadata.k.".to_string(),
            ]
        );
        assert!(doc.examples.contains_key("Default example"));
        assert_eq!(
            doc.examples.get("Default example"),
            Some(&Example::new(
                "".to_string(),
                "".to_string(),
                "myCustomApp = AppConfiguration {
    name = \"componentName\"
}"
                .to_string()
            ))
        );
    }
}
