
use std::collections::BTreeSet;

use crate::hash::Hasher;

pub struct DepParser {
    deps: Vec<String>,
    target: String,
}
impl DepParser {
    pub fn new(content: &str) -> Self {
        let mut dep_parser = DepParser {deps: Vec::new(), target: "".to_owned()};
        dep_parser.parse(content);
        dep_parser
    }

    fn parse(&mut self, content: &str) {
        let patched_lines_str = content.replace("\\\r\n", " ").replace("\\\n", " ");
        let mut tartet = None;
        let mut map: BTreeSet<String> = BTreeSet::new();
        for line in patched_lines_str.split("\n") {
            let mut column=0;
            let split = line.split('"');
            let mut is_prereq = false;
            for (i, el) in split.enumerate() {
                if i % 2 == 0 {
                    // not escaped
                    let mut val:String = "".to_owned();
                    let split2 = el.split_whitespace();
                    for el2 in split2 {
                        // println!("{}", el2);
                        column += el2.len();
                         if el2.ends_with(":") {
                            // found it
                            if tartet.is_none() {
                                tartet = Some(line[..column-1].trim().replace("\"", ""));
                            }
                            is_prereq = true;
                        } else if is_prereq {
                            val += el2;
                            if el2.ends_with('\\') {
                                val += " ";
                            } else  if !map.contains(&val.to_lowercase()) {
                                self.deps.push(val.to_owned());
                                map.insert(val.to_lowercase());
                                val = "".to_owned();
                            } else {
                                val = "".to_owned();
                            }
                        }
                        column += 1;
                    }
                } else {
                    // escaped
                    if is_prereq && !map.contains(&el.to_lowercase()) {
                        self.deps.push(el.to_owned());
                        map.insert(el.to_lowercase());
                    }
                    column += el.len();
                }
                column += 1;
            }
        }
        self.target = tartet.unwrap();
        // println!("{}", self.deps.get(0).unwrap());
        // println!("{}", self.target);
    }

    pub fn update_hash(&self, hasher: &mut Hasher) -> Result<(), Box<dyn std::error::Error + 'static>> {
        for dep in &self.deps {
            // dbg!("create hash for: {}", dep);
            hasher.update(&std::fs::read(dep.replace("\\ ", " "))?);
        }
        Ok(())
    }

    // create make dependency file from deps
    pub fn get_dep_file_string(&self) -> String {
        let mut dep_string = String::new();
        dep_string.push_str(&self.target);
        dep_string.push_str(":");
        for dep in &self.deps {
            dep_string.push_str(" \\\n");
            // this would create relative paths
            // it is not used because of the base_dir option
            // but kept for reference
            // if Path::new(dep).is_absolute() {
            //     dep_string.push_str(&diff_paths(dep, env::current_dir().unwrap()).unwrap().to_str().unwrap().replace("\\", "/"));
            // } else {
                dep_string.push_str(dep);
            // }
        }
        dep_string
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let content = "a:\\\r\n\
            b\r\n\
            c:\\\r\n\
            d\r\n\
            e:\r\n\
            f\r\n\
            g:\r\n\
            h";
        let dep_parser = DepParser::new(content);
        let expected = vec!["b", "d"];
        assert_eq!(dep_parser.deps, expected);
        assert_eq!(dep_parser.target, "a");
    }

    #[test]
    fn test_escape() {
        let content = "\"a\":\\\n\
            \"b\"\n\
            \"c\":\\\n\
            \"d\"\n\
            \"e\":\n\
            \"f\"\n\
            \"g\":\n\
            h";
        let dep_parser = DepParser::new(content);
        let expected = vec!["b", "d"];
        assert_eq!(dep_parser.deps, expected);
        assert_eq!(dep_parser.target, "a");
    }

    #[test]
    fn test_escape2() {
        let content = "\"a\":	\"b c\" d";
        let dep_parser = DepParser::new(content);
        let expected = vec!["b c", "d"];
        assert_eq!(dep_parser.deps, expected);
        assert_eq!(dep_parser.target, "a");
    }

    #[test]
    fn test_escape3() {
        let content = "\"a:\\bla\": \"b:/test c\" \"d\"";
        let dep_parser = DepParser::new(content);
        let expected = vec!["b:/test c", "d"];
        assert_eq!(dep_parser.deps, expected);
        assert_eq!(dep_parser.target, "a:\\bla");
    }

    #[test]
    fn test_escape4() {
        let content = "a: b\\ c\\ d e f";
        let dep_parser = DepParser::new(content);
        let expected = vec!["b\\ c\\ d", "e", "f"];
        assert_eq!(dep_parser.deps, expected);
    }

    #[test]
    fn test_escape5() {
        let content = "a\\ b: c d";
        let dep_parser = DepParser::new(content);
        let expected = vec!["c", "d"];
        assert_eq!(dep_parser.deps, expected);
        assert_eq!(dep_parser.target, "a\\ b");
    }

    #[test]
    fn test_duplicates() {
        let content = "a: b c b d c";
        let dep_parser = DepParser::new(content);
        let expected = vec!["b", "c", "d"];
        assert_eq!(dep_parser.deps, expected);
        assert_eq!(dep_parser.target, "a");
    }
}