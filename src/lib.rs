use core::panic;
use std::collections::HashMap;

#[derive(Debug)]
pub enum JsonType {
    Int(i32),
    Float(f32),
    String(String),
    Bool(bool),
    Array(Vec<JsonType>),
    Object(HashMap<String, JsonType>),
}

pub fn parse(mut json: &[char]) -> (JsonType, &[char]) {
    fn left_trim(mut str: &[char]) -> &[char] {
        while !str.is_empty() && str[0].is_whitespace() {
            str = &str[1..];
        }
        str
    }

    let len = json.len();
    let mut end = 0;
    let (result, remainder) = match json {
        [c, ..] if c.is_whitespace() => parse(left_trim(&json[1..])), //. Skip whitespaces
        [c, ..] if ['-', '+', '0', '1', '2', '3', '4', '5', '6', '7', '8', '9'].contains(&c) => {
            let start = end;
            if ['-', '+'].contains(&c) {
                end += 1;
            }
            while end < len && json[end].is_ascii_digit() {
                end += 1;
            }
            if end < len && json[end] == '.' {
                end += 1;
                while end < len && json[end].is_ascii_digit() {
                    end += 1;
                }
                if end < len && (json[end] == 'e' || json[end] == 'E') {
                    //. Scientific notation
                    end += 1;
                    while end < len && ['-', '+', '0', '1', '2', '3', '4', '5', '6', '7', '8', '9'].contains(&json[end]) {
                        end += 1;
                    }
                }

                (
                    JsonType::Float({
                        let s = json[start..end].iter().collect::<String>();
                        s.parse().unwrap_or_else(|_| panic!("Could not parse number {s}"))
                    }),
                    &json[end..],
                )
            } else {
                (
                    JsonType::Int({
                        let s = json[start..end].iter().collect::<String>();
                        s.parse().unwrap_or_else(|_| panic!("Could not parse number {s}"))
                    }),
                    &json[end..],
                )
            }
        },
        [c, ..] if *c == '"' => {
            end += 1;
            let start = end;
            while end < len && json[end] != '"' {
                // TODO: Handle escape charso
                end += 1;
            }
            let s = json[start..end].iter().collect::<String>();
            (JsonType::String(s), &json[end + 1..]) //. Eat the end quote
        },
        ['t', 'r', 'u', 'e', ..] => (JsonType::Bool(true), &json[4..]),
        ['f', 'a', 'l', 's', 'e', ..] => (JsonType::Bool(false), &json[5..]),
        ['{', ..] => {
            let mut map: HashMap<String, JsonType> = Default::default();
            json = &json[1..];
            loop {
                match json {
                    [c, ..] if c.is_whitespace() => json = left_trim(json),
                    [',', ..] => json = &json[1..],
                    ['}', ..] => {
                        json = &json[1..];
                        break;
                    },
                    ['"', ..] => {
                        //. Don't "pop" the start quote
                        if let (JsonType::String(key), js) = parse(json) {
                            json = left_trim(js);
                            assert!(json[0] == ':');
                            let val;
                            (val, json) = parse(&json[1..]);
                            map.insert(key, val);
                        } else {
                            panic!()
                        }
                    },
                    [c, ..] => {
                        panic!("Error, unexpected {c} in object")
                    },
                    [] => {
                        panic!("Error, unexpected EOF in object")
                    },
                }
            }
            (JsonType::Object(map), json)
        },
        ['[', ..] => {
            let mut vec: Vec<JsonType> = vec![];
            json = &json[1..];
            loop {
                // TODO: assert elements are same type
                match json {
                    [c, ..] if c.is_whitespace() => json = left_trim(json),
                    [',', ..] => json = &json[1..],
                    [']', ..] => {
                        json = &json[1..];
                        break;
                    },
                    _ => {
                        let val;
                        (val, json) = parse(json);
                        vec.push(val);
                    },
                }
            }
            (JsonType::Array(vec), json)
        },
        [c, ..] => panic!("Unexpected character {c}, in {}", json.iter().collect::<String>()),
        [] => panic!("Unexpected end"),
    };
    return (result, left_trim(remainder)); //. Trim trailing whitespace
}

pub fn print_json_str(json: &str) {
    let mut indent: usize = 0;
    let indent_base = "  ";
    for char in json.chars() {
        match char {
            c if ['{', '['].contains(&c) => {
                indent += 1;
                let istr = indent_base.repeat(indent);
                print!("{c}\n{istr}");
            },
            c if ['}', ']'].contains(&c) => {
                indent -= 1;
                let istr = indent_base.repeat(indent);
                print!("\n{istr}{c}");
            },
            ',' => {
                let istr = indent_base.repeat(indent);
                print!(",\n{istr}");
            },
            c => {
                print!("{c}");
            },
        }
    }
}

// fn main() {
//     let json = str::from_utf8(include_bytes!("test.json")).expect("Could not parse as utf8");
//     let json_chars = json.chars().collect::<Vec<char>>();
//     let result = parse(&json_chars);
//     println!("{result:?}");
// }
