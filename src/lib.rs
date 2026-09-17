use core::panic;
use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub enum JsonType {
    Null,
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
            let mut str: String = String::new();
            while end < len && json[end] != '"' {
                if json[end] == '\\' {
                    end += 1;
                    if end >= len {
                        println!("Error, unexpected EOF after \\ in unclosed string");
                        panic!()
                    }
                    match json[end] {
                        '"' | '\\' | '/' => {
                            str.push(json[end]);
                            end += 1;
                        },
                        'b' => {
                            str.push('\x08');
                            end += 1;
                        },
                        'f' => {
                            str.push('\x0C');
                            end += 1;
                        },
                        'n' => {
                            str.push('\n');
                            end += 1;
                        },
                        'r' => {
                            str.push('\r');
                            end += 1;
                        },
                        't' => {
                            str.push('\t');
                            end += 1;
                        },
                        'u' => {
                            const UNICODE_CODE_POINT_HEX_DIGITS: usize = 4;
                            end += 1;
                            if end + UNICODE_CODE_POINT_HEX_DIGITS >= len {
                                println!("Error, unexpected EOF after \\u in unclosed string, expected {UNICODE_CODE_POINT_HEX_DIGITS} hex digits");
                                panic!()
                            }
                            let hex: [char; UNICODE_CODE_POINT_HEX_DIGITS] = unsafe { json[end..end + UNICODE_CODE_POINT_HEX_DIGITS].try_into().unwrap_unchecked() }; // NOTE[Safety]: The size [end..end+4[ is 4 elements. Assuming no usize addition overflow.
                            let mut val = 0;
                            for char in hex {
                                val *= 16;
                                val += match char {
                                    '0'..'9' => char as u32 - '0' as u32,
                                    'a'..'f' => 10 + char as u32 - 'a' as u32,
                                    'A'..'F' => 10 + char as u32 - 'A' as u32,
                                    _ => {
                                        println!("Error, unexpected character {char} after \\u, expected {UNICODE_CODE_POINT_HEX_DIGITS} hex digits");
                                        panic!()
                                    },
                                }
                            }
                            str.push(char::from_u32(val).expect("Unicode code point specified by \\u{hex:?} not valid char"));
                            end += UNICODE_CODE_POINT_HEX_DIGITS;
                        },
                        c => {
                            println!("Error, unexpected escape character \\{c} in string");
                            panic!()
                        },
                    }
                } else {
                    str.push(json[end]);
                    end += 1;
                }
            }
            if json[end] != '"' {
                println!("Error, unexpected EOF in unclosed string");
                panic!()
            }
            (JsonType::String(str), &json[end + 1..]) //. Eat the end quote
        },
        ['t', 'r', 'u', 'e', ..] => (JsonType::Bool(true), &json[4..]),
        ['f', 'a', 'l', 's', 'e', ..] => (JsonType::Bool(false), &json[5..]),
        ['n', 'u', 'l', 'l', ..] => (JsonType::Null, &json[4..]),
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::assert_eq;
    macro_rules! hashmap {
    ($($key:expr => $value:expr),* $(,)?) => {
            std::collections::HashMap::from([
                $(($key, $value)),*
            ])
        };
    }

    #[test]
    fn escape_example() {
        let input = r#"{ "escaped": "\" \\ \/ \b \f \n \r \t \u0041" }"#.chars().collect::<Vec<char>>();

        let (parsed, remainder) = parse(&input);

        let expected = JsonType::Object(hashmap! {
            "escaped".to_string() => JsonType::String(
               "\" \\ / \x08 \x0C \n \r \t A".to_string()
            ),
        });

        assert_eq!(parsed, expected);
        assert_eq!(remainder, []);

        println!("{parsed:?}");
    }

    #[test]
    fn gltf_example() {
        let input = str::from_utf8(include_bytes!("gltf_example.json")).expect("Could not parse as utf8").chars().collect::<Vec<char>>();
        let (parsed, remainder) = parse(&input);

        let expected = JsonType::Object(hashmap! {
            "asset".into() => JsonType::Object(hashmap! {
                "generator".into() => JsonType::String("COLLADA2GLTF".into()),
                "version".into() => JsonType::String("2.0".into()),
            }),

            "scene".into() => JsonType::Int(0),

            "scenes".into() => JsonType::Array(vec![
                JsonType::Object(hashmap! {
                    "nodes".into() => JsonType::Array(vec![
                        JsonType::Int(3),
                        JsonType::Int(0),
                    ]),
                }),
            ]),

            "nodes".into() => JsonType::Array(vec![
                JsonType::Object(hashmap! {
                    "children".into() => JsonType::Array(vec![
                        JsonType::Int(1),
                    ]),
                    "rotation".into() => JsonType::Array(vec![
                        JsonType::Float(-0.0),
                        JsonType::Float(-0.0),
                        JsonType::Float(-0.0),
                        JsonType::Float(-1.0),
                    ]),
                }),
                JsonType::Object(hashmap! {
                    "children".into() => JsonType::Array(vec![
                        JsonType::Int(2),
                    ]),
                }),
                JsonType::Object(hashmap! {
                    "mesh".into() => JsonType::Int(0),
                    "rotation".into() => JsonType::Array(vec![
                        JsonType::Float(-0.0),
                        JsonType::Float(-0.0),
                        JsonType::Float(-0.0),
                        JsonType::Float(-1.0),
                    ]),
                }),
                JsonType::Object(hashmap! {
                    "mesh".into() => JsonType::Int(1),
                }),
            ]),

            "meshes".into() => JsonType::Array(vec![
                JsonType::Object(hashmap! {
                    "primitives".into() => JsonType::Array(vec![
                        JsonType::Object(hashmap! {
                            "attributes".into() => JsonType::Object(hashmap! {
                                "NORMAL".into() => JsonType::Int(1),
                                "POSITION".into() => JsonType::Int(2),
                            }),
                            "indices".into() => JsonType::Int(0),
                            "mode".into() => JsonType::Int(4),
                            "material".into() => JsonType::Int(0),
                        }),
                    ]),
                    "name".into() => JsonType::String("inner_box".into()),
                }),
                JsonType::Object(hashmap! {
                    "primitives".into() => JsonType::Array(vec![
                        JsonType::Object(hashmap! {
                            "attributes".into() => JsonType::Object(hashmap! {
                                "NORMAL".into() => JsonType::Int(4),
                                "POSITION".into() => JsonType::Int(5),
                            }),
                            "indices".into() => JsonType::Int(3),
                            "mode".into() => JsonType::Int(4),
                            "material".into() => JsonType::Int(1),
                        }),
                    ]),
                    "name".into() => JsonType::String("outer_box".into()),
                }),
            ]),

            "animations".into() => JsonType::Array(vec![
                JsonType::Object(hashmap! {
                    "channels".into() => JsonType::Array(vec![
                        JsonType::Object(hashmap! {
                            "sampler".into() => JsonType::Int(0),
                            "target".into() => JsonType::Object(hashmap! {
                                "node".into() => JsonType::Int(2),
                                "path".into() => JsonType::String("rotation".into()),
                            }),
                        }),
                        JsonType::Object(hashmap! {
                            "sampler".into() => JsonType::Int(1),
                            "target".into() => JsonType::Object(hashmap! {
                                "node".into() => JsonType::Int(0),
                                "path".into() => JsonType::String("translation".into()),
                            }),
                        }),
                    ]),
                    "samplers".into() => JsonType::Array(vec![
                        JsonType::Object(hashmap! {
                            "input".into() => JsonType::Int(6),
                            "interpolation".into() => JsonType::String("LINEAR".into()),
                            "output".into() => JsonType::Int(7),
                        }),
                        JsonType::Object(hashmap! {
                            "input".into() => JsonType::Int(8),
                            "interpolation".into() => JsonType::String("LINEAR".into()),
                            "output".into() => JsonType::Int(9),
                        }),
                    ]),
                }),
            ]),

            "accessors".into() => JsonType::Array(vec![
                JsonType::Object(hashmap! {
                    "bufferView".into() => JsonType::Int(0),
                    "byteOffset".into() => JsonType::Int(0),
                    "componentType".into() => JsonType::Int(5123),
                    "count".into() => JsonType::Int(186),
                    "max".into() => JsonType::Array(vec![
                        JsonType::Int(95),
                    ]),
                    "min".into() => JsonType::Array(vec![
                        JsonType::Int(0),
                    ]),
                    "type".into() => JsonType::String("SCALAR".into()),
                }),
                JsonType::Object(hashmap! {
                    "bufferView".into() => JsonType::Int(1),
                    "byteOffset".into() => JsonType::Int(0),
                    "componentType".into() => JsonType::Int(5126),
                    "count".into() => JsonType::Int(96),
                    "max".into() => JsonType::Array(vec![
                        JsonType::Float(1.0),
                        JsonType::Float(1.0),
                        JsonType::Float(1.0),
                    ]),
                    "min".into() => JsonType::Array(vec![
                        JsonType::Float(-1.0),
                        JsonType::Float(-1.0),
                        JsonType::Float(-1.0),
                    ]),
                    "type".into() => JsonType::String("VEC3".into()),
                }),
                JsonType::Object(hashmap! {
                    "bufferView".into() => JsonType::Int(1),
                    "byteOffset".into() => JsonType::Int(1152),
                    "componentType".into() => JsonType::Int(5126),
                    "count".into() => JsonType::Int(96),
                    "max".into() => JsonType::Array(vec![
                        JsonType::Float(0.33504000306129458),
                        JsonType::Float(0.5),
                        JsonType::Float(0.33504000306129458),
                    ]),
                    "min".into() => JsonType::Array(vec![
                        JsonType::Float(-0.33504000306129458),
                        JsonType::Float(-0.5),
                        JsonType::Float(-0.33504000306129458),
                    ]),
                    "type".into() => JsonType::String("VEC3".into()),
                }),
                JsonType::Object(hashmap! {
                    "bufferView".into() => JsonType::Int(0),
                    "byteOffset".into() => JsonType::Int(372),
                    "componentType".into() => JsonType::Int(5123),
                    "count".into() => JsonType::Int(576),
                    "max".into() => JsonType::Array(vec![
                        JsonType::Int(223),
                    ]),
                    "min".into() => JsonType::Array(vec![
                        JsonType::Int(0),
                    ]),
                    "type".into() => JsonType::String("SCALAR".into()),
                }),
                JsonType::Object(hashmap! {
                    "bufferView".into() => JsonType::Int(1),
                    "byteOffset".into() => JsonType::Int(2304),
                    "componentType".into() => JsonType::Int(5126),
                    "count".into() => JsonType::Int(224),
                    "max".into() => JsonType::Array(vec![
                        JsonType::Float(1.0),
                        JsonType::Float(1.0),
                        JsonType::Float(1.0),
                    ]),
                    "min".into() => JsonType::Array(vec![
                        JsonType::Float(-1.0),
                        JsonType::Float(-1.0),
                        JsonType::Float(-1.0),
                    ]),
                    "type".into() => JsonType::String("VEC3".into()),
                }),
                JsonType::Object(hashmap! {
                    "bufferView".into() => JsonType::Int(1),
                    "byteOffset".into() => JsonType::Int(4992),
                    "componentType".into() => JsonType::Int(5126),
                    "count".into() => JsonType::Int(224),
                    "max".into() => JsonType::Array(vec![
                        JsonType::Float(0.5),
                        JsonType::Float(0.5),
                        JsonType::Float(0.5),
                    ]),
                    "min".into() => JsonType::Array(vec![
                        JsonType::Float(-0.5),
                        JsonType::Float(-0.5),
                        JsonType::Float(-0.5),
                    ]),
                    "type".into() => JsonType::String("VEC3".into()),
                }),
                JsonType::Object(hashmap! {
                    "bufferView".into() => JsonType::Int(2),
                    "byteOffset".into() => JsonType::Int(0),
                    "componentType".into() => JsonType::Int(5126),
                    "count".into() => JsonType::Int(2),
                    "max".into() => JsonType::Array(vec![
                        JsonType::Float(2.5),
                    ]),
                    "min".into() => JsonType::Array(vec![
                        JsonType::Float(1.25),
                    ]),
                    "type".into() => JsonType::String("SCALAR".into()),
                }),
                JsonType::Object(hashmap! {
                    "bufferView".into() => JsonType::Int(3),
                    "byteOffset".into() => JsonType::Int(0),
                    "componentType".into() => JsonType::Int(5126),
                    "count".into() => JsonType::Int(2),
                    "max".into() => JsonType::Array(vec![
                        JsonType::Float(1.0),
                        JsonType::Float(0.0),
                        JsonType::Float(0.0),
                        JsonType::Float(4.4896593387466768e-11),
                    ]),
                    "min".into() => JsonType::Array(vec![
                        JsonType::Float(-0.0),
                        JsonType::Float(0.0),
                        JsonType::Float(0.0),
                        JsonType::Float(-1.0),
                    ]),
                    "type".into() => JsonType::String("VEC4".into()),
                }),
                JsonType::Object(hashmap! {
                    "bufferView".into() => JsonType::Int(2),
                    "byteOffset".into() => JsonType::Int(8),
                    "componentType".into() => JsonType::Int(5126),
                    "count".into() => JsonType::Int(4),
                    "max".into() => JsonType::Array(vec![
                        JsonType::Float(3.708329916000366),
                    ]),
                    "min".into() => JsonType::Array(vec![
                        JsonType::Float(0.0),
                    ]),
                    "type".into() => JsonType::String("SCALAR".into()),
                }),
                JsonType::Object(hashmap! {
                    "bufferView".into() => JsonType::Int(4),
                    "byteOffset".into() => JsonType::Int(0),
                    "componentType".into() => JsonType::Int(5126),
                    "count".into() => JsonType::Int(4),
                    "max".into() => JsonType::Array(vec![
                        JsonType::Float(0.0),
                        JsonType::Float(2.5199999809265138),
                        JsonType::Float(0.0),
                    ]),
                    "min".into() => JsonType::Array(vec![
                        JsonType::Float(0.0),
                        JsonType::Float(0.0),
                        JsonType::Float(0.0),
                    ]),
                    "type".into() => JsonType::String("VEC3".into()),
                }),
            ]),

            "materials".into() => JsonType::Array(vec![
                JsonType::Object(hashmap! {
                    "pbrMetallicRoughness".into() => JsonType::Object(hashmap! {
                        "baseColorFactor".into() => JsonType::Array(vec![
                            JsonType::Float(0.800000011920929),
                            JsonType::Float(0.4159420132637024),
                            JsonType::Float(0.7952920198440552),
                            JsonType::Float(1.0),
                        ]),
                        "metallicFactor".into() => JsonType::Float(0.0),
                    }),
                    "name".into() => JsonType::String("inner".into()),
                }),
                JsonType::Object(hashmap! {
                    "pbrMetallicRoughness".into() => JsonType::Object(hashmap! {
                        "baseColorFactor".into() => JsonType::Array(vec![
                            JsonType::Float(0.3016040027141571),
                            JsonType::Float(0.5335419774055481),
                            JsonType::Float(0.800000011920929),
                            JsonType::Float(1.0),
                        ]),
                        "metallicFactor".into() => JsonType::Float(0.0),
                    }),
                    "name".into() => JsonType::String("outer".into()),
                }),
            ]),

            "bufferViews".into() => JsonType::Array(vec![
                JsonType::Object(hashmap! {
                    "buffer".into() => JsonType::Int(0),
                    "byteOffset".into() => JsonType::Int(7784),
                    "byteLength".into() => JsonType::Int(1524),
                    "target".into() => JsonType::Int(34963),
                }),
                JsonType::Object(hashmap! {
                    "buffer".into() => JsonType::Int(0),
                    "byteOffset".into() => JsonType::Int(80),
                    "byteLength".into() => JsonType::Int(7680),
                    "byteStride".into() => JsonType::Int(12),
                    "target".into() => JsonType::Int(34962),
                }),
                JsonType::Object(hashmap! {
                    "buffer".into() => JsonType::Int(0),
                    "byteOffset".into() => JsonType::Int(7760),
                    "byteLength".into() => JsonType::Int(24),
                }),
                JsonType::Object(hashmap! {
                    "buffer".into() => JsonType::Int(0),
                    "byteOffset".into() => JsonType::Int(0),
                    "byteLength".into() => JsonType::Int(32),
                }),
                JsonType::Object(hashmap! {
                    "buffer".into() => JsonType::Int(0),
                    "byteOffset".into() => JsonType::Int(32),
                    "byteLength".into() => JsonType::Int(48),
                }),
            ]),

            "buffers".into() => JsonType::Array(vec![
                JsonType::Object(hashmap! {
                    "byteLength".into() => JsonType::Int(9308),
                }),
            ]),
        });

        assert_eq!(parsed, expected);
        assert_eq!(remainder, []);

        println!("{parsed:?}");
    }
}
