use kdl::{KdlDocument, KdlNode, KdlValue};
use serde_json::{Map, Value, json};

//
// --- JSON → KDL ---
//
pub  fn json_to_kdl(value: &Value, key: Option<&str>) -> KdlNode {
    match value {
        Value::Object(map) => {
            let mut node = KdlNode::new(key.unwrap_or("object"));
            let mut children = KdlDocument::new();
            for (k, v) in map {
                children.nodes_mut().push(json_to_kdl(v, Some(k)));
            }
            node.set_children(children);
            node
        }
        Value::Array(arr) => {
            let mut node = KdlNode::new(key.unwrap_or("array"));
            let mut children = KdlDocument::new();
            for v in arr {
                children.nodes_mut().push(json_to_kdl(v, Some("item")));
            }
            node.set_children(children);
            node
        }
        Value::String(s) => {
            let mut node = KdlNode::new(key.unwrap_or("value"));
            node.push(KdlValue::String(s.clone()));
            node
        }
        Value::Number(num) => {
            let mut node = KdlNode::new(key.unwrap_or("value"));
            if let Some(i) = num.as_i64() {
                node.push(KdlValue::Integer(i.into()));
            } else if let Some(f) = num.as_f64() {
                node.push(KdlValue::Float(f));
            }
            node
        }
        Value::Bool(b) => {
            let mut node = KdlNode::new(key.unwrap_or("value"));
            node.push(KdlValue::Bool(*b));
            node
        }
        Value::Null => {
            let mut node = KdlNode::new(key.unwrap_or("value"));
            node.push(KdlValue::Null);
            node
        }
    }
}

//
// --- KDL → JSON ---
//
pub fn kdl_to_json(node: &KdlNode) -> Value {
    // If the node has children, it’s either an object or an array.
    if let Some(children) = node.children() {
        let nodes = children.nodes();

        // Check if all child nodes are named "item"
        let all_items = nodes.iter().all(|n| n.name().value() == "item");

        if all_items {
            // Convert to a JSON array
            let arr: Vec<Value> = nodes.iter().map(kdl_to_json).collect();
            return Value::Array(arr);
        } else {
            // Convert to a JSON object
            let mut map = Map::new();
            for child in nodes {
                map.insert(child.name().to_string(), kdl_to_json(child));
            }
            return Value::Object(map);
        }
    }

    // No children → use arguments as values
    let args = node.entries();
    if args.len() == 1 {
        match args[0].value() {
            KdlValue::String(s) => Value::String(s.clone()),
            KdlValue::Bool(b) => Value::Bool(*b),
            KdlValue::Integer(num) => Value::Number((*num as i64).into()),
            KdlValue::Float(f) => json!(f),
            KdlValue::Null => Value::Null,
        }
    } else {
        Value::Null
    }
}

#[cfg(test)]
mod tests {
    use kdl::KdlDocument;
    use serde_json::Value;

    use crate::mapping::kdl::{json_to_kdl, kdl_to_json};


    #[test]
    fn test_json_to_kdl_roundtrip() {
        let json_str = r#"
    {
        "name": "Alice",
        "age": 30,
        "hobbies": ["reading", "hiking"],
        "favorites": {
            "colors": ["blue", "green"],
            "food": "pizza"
        },
        "is_admin": false
    }
    "#;

        let json_value: Value = serde_json::from_str(json_str).unwrap();
        let kdl_root = json_to_kdl(&json_value, Some("root"));
        let mut kdl_doc = KdlDocument::new();
        kdl_doc.nodes_mut().push(kdl_root);

        assert_eq!(kdl_doc.to_string(), "root{\nage 30\nfavorites{\ncolors{\nitem blue\nitem green\n        }\nfood pizza\n    }\nhobbies{\nitem reading\nitem hiking\n    }\nis_admin #false\nname Alice\n}\n".to_string());

        let parsed_doc = KdlDocument::parse(&kdl_doc.to_string()).unwrap();
        let root_node = parsed_doc.get("root").unwrap();
        let back_to_json = kdl_to_json(root_node);

        assert_eq!(serde_json::to_string_pretty(&back_to_json).unwrap(), "{\n  \"age\": 30,\n  \"favorites\": {\n    \"colors\": [\n      \"blue\",\n      \"green\"\n    ],\n    \"food\": \"pizza\"\n  },\n  \"hobbies\": [\n    \"reading\",\n    \"hiking\"\n  ],\n  \"is_admin\": false,\n  \"name\": \"Alice\"\n}".to_string())
    }
}
