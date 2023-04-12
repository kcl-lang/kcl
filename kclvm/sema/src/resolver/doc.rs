use std::fs::File;
use std::io::prelude::*;
use std::iter::Iterator;
use std::collections::HashSet;
use regex::Regex;

fn read_doc_content() -> String {
    let mut file = File::open("/Users/amy/Documents/practice/rust/doc_parser/docstring.txt").expect("Unable to open file");
    let mut contents = String::new();
    file.read_to_string(&mut contents).expect("Unable to read file");
    contents
}

// strip leading and trailing triple quotes in the original docstring content
fn strip_quotes(original: String) -> String {
    let quote = original.chars().next().unwrap();
    let pattern = format!("(?s)^{char}{{3}}(.*?){char}{{3}}$", char=quote);
    let re = Regex::new(&pattern).unwrap();
    let caps = re.captures(&original);
    let result = match caps {
        Some(caps) => caps,
        None => return original,
    };
    let content = &result[1];
    content.to_owned()
}

fn expand_tabs(s: &str, spaces_per_tab: usize) -> String {
    s.replace("\t", &" ".repeat(spaces_per_tab))
}

// Clean up indentation by removing any common leading whitespace
// on all lines after the first line.
pub fn clean_doc(doc: &mut String) -> &mut String{
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
            *line = if line.len() > 0 {&line[margin..]} else {line}; // remove command indentation
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
    doc
}


// A line-based string reader.
struct Reader<'a> {
    data: Vec<&'a str>,
    l: usize,
}

impl<'a> Reader<'a> {
    fn new(data: &'a str) -> Self {
        let data_vec: Vec<&str> = data.split('\n').collect();
        Self {
            data: data_vec,
            l: 0,
        }
    }
    fn reset(&mut self) {
        self.l = 0;
    }
    
    fn read(&mut self) -> &'a str {
        if !self.eof() {
            let out = self.data[self.l];
            self.l += 1;
            return out;
        } else {
            return "";
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
    
    fn read_to_condition(&mut self, condition_func: &dyn Fn(&str) -> bool) -> Vec<&'a str> {
        let start = self.l;
        for line in self.data[start..].iter() {
            if condition_func(line) {
                return self.data[start..self.l].to_vec();
            }
            self.l += 1;
            if self.eof() {
                return self.data[start..self.l + 1].to_vec();
            }
        }
        return vec![];
    }
    
    fn read_to_next_empty_line(&mut self) -> Vec<&'a str> {
        self.seek_next_non_empty_line();
    
        fn is_empty(line: &str) -> bool {
            return line.trim().len() == 0;
        }
    
        return self.read_to_condition(&is_empty);
    }
    
    fn read_to_next_unindented_line(&mut self) -> Vec<&'a str> {
        fn is_unindented(line: &str) -> bool {
            return line.trim().len() > 0 && line.trim_start().len() == line.len();
        }
    
        return self.read_to_condition(&is_unindented);
    }
    
    fn peek(&self, n: usize, positive: bool) -> &'a str {
        if positive {
            if self.l + n < self.data.len() {
                return self.data[self.l + n];
            } else {
                return "";
            }
        } else {
            if self.l >= n {
                return self.data[self.l - n];
            } else {
                return "";
            }
        }
    }
    
    fn is_empty(&self) -> bool {
        return self.data.iter().all(|&x| x.trim().len() == 0);
    }    
}

// remove the leading and trailing empty lines
fn _strip(doc: Vec<&str>) -> Vec<&str> {
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

// Checks if current line is at the beginning of a section
fn is_at_section(doc: &mut Reader) -> bool {
    doc.seek_next_non_empty_line();
    if doc.eof() {
        return false;
    }
    let l1 = doc.peek(0, true).trim();
    let l2 = doc.peek(1, true).trim(); // ---------- or ==========
    let l2_char_set = l2.chars().collect::<HashSet<char>>();

    if l2.len() >= 3 && l2_char_set.len() == 1  && (l2.contains('-') || l2.contains('=')) && l1.len() != l1.len() {
        // todo: when line2 is conformed with "-" or "=", but the number of the "-/=" mismatch the section title length, mark as a section and return a warning
        return false;
    }
    l2.starts_with(&"-".repeat(l1.len())) || l2.starts_with(&"=".repeat(l1.len()))
}

// read lines before next section beginning, continuous empty lines will be merged to one
fn read_to_next_section<'a>(doc: &'a mut Reader<'a>) -> Vec<&'a str> {
    let mut section = doc.read_to_next_empty_line();

    while !is_at_section(doc) && !doc.eof() {
        if doc.peek(1, false).trim().is_empty() {
            section.push(doc.peek(1, false));
        }
        section.append(&mut doc.read_to_next_empty_line());
    }
    section
}

// read all sections, returns list of each section Title and the section content
// For following docstring lines, the extracted sections will be: 
// [("Attribute", ["content", "content"]), ("Examples", ["content"])]
//
// Attribute
// ----------
// content
// content

// Examples
// --------
// content
// fn read_sections<'a>(doc: &'a mut Reader<'a>) ->  Vec<Result<(String, Vec<&str>), &'static str>>{
//     let mut sections = vec![];


//     while !doc.eof() {
//         let data = read_to_next_section(doc);
//         let name = data[0].trim().to_owned();

//         if data.len() < 2 {
//             sections.push(Ok((name, vec![])))
//         } else {
//             sections.push(Ok((name, _strip(data[2..].to_vec()))))
//         }
//     }
//     sections
// }

// parse 
fn parse_attr_list(content: &str) -> Vec<Attribute> {
    let mut r = Reader::new(content);
    let mut attrs = vec![];
    while !r.eof() {
        let header = r.read().trim();
        // 
        if header.contains(" : ") {
            let parts: Vec<&str> = header.split(" : ").collect();
            let arg_name = parts[0];
            let desc_lines: Vec<String> = r.read_to_next_unindented_line().iter().map(|&s| s.to_string()).collect();
            attrs.push(Attribute::new(arg_name, desc_lines));
        } else {
            let arg_name = header;
            let desc_lines: Vec<String> = r.read_to_next_unindented_line().iter().map(|&s| s.to_string()).collect();
            attrs.push(Attribute::new(arg_name, desc_lines));
        }
    }
    attrs
}

// parse the summary of the schema. The final summary content will be a concat of lines in the original summary with whitespace.
fn parse_summary<'a>(doc: &'a mut Reader<'a>) -> Option<String> {
    if is_at_section(doc){
        // no summary provided
        return None
    }
    let lines = read_to_next_section(doc);
    return Some(lines.iter().map(|s| s.trim()).collect::<Vec<_>>().join(" ").trim().to_string());
}

// the main logic of parsing the schema docstring
// fn parse_doc_string(ori: &mut String) -> Doc {
//     let cleaned = clean_doc(&mut ori);
//     let doc = &mut Reader::new(&cleaned.as_str());
//     doc.reset();
//     let summary = parse_summary(doc);
//     let attr_section = read_to_next_section(doc);
//     let attr_content_cleaned = attr_section.iter().map(|s| s.trim()).collect::<Vec<_>>().join(" ").trim();
//     let attrs = parse_attr_list(attr_content_cleaned);

//     Doc::new(summary, attrs)
// }

#[derive(Debug)]
struct Doc<'a> {
    summary: Option<String>,
    attrs: Vec<Attribute<'a>>,
}

impl<'a> Doc<'a> {
    fn new(summary: Option<String>, attrs: Vec<Attribute<'a>>) -> Self {
        Self {
            summary,
            attrs: attrs,
        }
    }
}

#[derive(Debug)]
struct Attribute<'a> {
    name: &'a str,
    desc: Vec<String>,
}

impl<'a> Attribute<'a> {
    fn new(name: &'a str, desc: Vec<String>,) -> Self {
        Self {
            name,
            desc,
        }
    }
}


#[test]
fn test_strip_quotes() {
    let ori_from_file = read_doc_content();

    let oris = [r#""""abcde""""#, r#"'''abc
de'''"#, ori_from_file.as_str()];
    let results = ["abcde", "abc
de", r#"Server is the common user interface for long-running
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
    labels : {str:str}, optional
        A Server-level attribute.
        The labels of the long-running service.
        See also: kusion_models/core/v1/metadata.k.

    Examples
    ----------------------
    myCustomApp = AppConfiguration {
        name = "componentName"
    }


    "#];

    for (ori, res) in oris.iter().zip(results.iter()) {
        assert_eq!(strip_quotes(ori.to_string()), res.to_string());
    }
    
}

#[test]
fn test_clean_doc() {
    let mut ori = strip_quotes(read_doc_content());
    let cleaned = clean_doc(&mut ori);
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
labels : {str:str}, optional
    A Server-level attribute.
    The labels of the long-running service.
    See also: kusion_models/core/v1/metadata.k.

Examples
----------------------
myCustomApp = AppConfiguration {
    name = "componentName"
}"#;
assert_eq!(cleaned, &expect_cleaned.to_string());
}


#[test]
fn test_seek_next_non_empty_line() {
    let data = "line1
    line2

    
    line3

    line4

    ";
    let mut reader = Reader::new(data);

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
    let input_str = "hello
    world

    foo
    bar

abc
    ";
    let mut reader = Reader::new(input_str);

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
    let mut reader = Reader::new(data);
    let result = reader.read_to_next_unindented_line();
    assert_eq!(result, vec!["", "    indented line", "    indented line", "        indented line", "    indented line", ""]);
}

#[test]
fn test_at_section() {
    let data = "Summary
    Attribute
    ---------
    description";
    
    let mut doc = Reader::new(data);
    assert!(!is_at_section(&mut doc));


    assert_eq!(doc.read(), "Summary");
    assert!(is_at_section(&mut doc));

    assert_eq!(doc.read(), "Attribute");
    assert!(!is_at_section(&mut doc));
}

#[test]
fn test_read_to_next_section() {
    let data = "Summary
    

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
    content";
    let mut doc = Reader::new(data);
    assert_eq!(read_to_next_section(&mut doc), vec!["Summary", "", "SummaryContinue"]);
}

// #[test]
// fn test_parse_doc() {
//     let mut content = read_doc_content();
//     parse_doc_string(&mut content);
// }
