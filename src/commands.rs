
use lazy_static::lazy_static;
use resp::Value;
use std::{collections::HashMap, sync::Mutex};

type STORE = Mutex<HashMap<String, String>>;
// static and const only can be initialized with primitives, lazy_static lift the restrictions
lazy_static! {
    static ref RUDIS_DB: STORE = Mutex::new(HashMap::new());
}


pub fn process_client_request(decoded_msg: Value) -> Vec<u8> {
    let reply = if let Value::Array(v) = decoded_msg {
        // if decoded_msg contains RESP::Value, and Value is Array, we match agains it
        // v = [Value::Bulk("SET/GET"), Value::Bulk("KEY")]
        match &v[0] {
            Value::Bulk(s) => {
                if s == "GET" || s == "get" || s == "fetch" {
                    println!("GETTER => {}", s);
                    handle_get(v)
                } else if s == "SET" || s == "set" {
                    handle_set(v)
                } else {
                    Err(Value::Error(format!(
                        "'{:?}' is not supported as of now",
                        s
                    )))
                }
            }
            other => Err(Value::Error(format!(
                "'{:?}' is not supported as of now",
                other
            ))),
        }
    } else {
        Err(Value::Error("Invalid Command".to_string()))
    };

    match reply {
        // returns Vec<u8>
        Ok(r) | Err(r) => r.encode(),
    }
}

// Result<Value, Value> == Ok<Value> | Err<Value>
pub fn handle_get(v: Vec<Value>) -> Result<Value, Value> {
    // v = [Value::Bulk("SET/GET"), Value::Bulk("KEY")]
    let v: Vec<_> = v.iter().skip(1).collect();
    // if QUERY KEY is not provided as first arguent, return with error
    if v.is_empty() {
        return Err(Value::Error(
            "Expected 1 argument for GET command".to_string(),
        ));
    }
    // if key is provided, find the key in DB
    let redis_db = RUDIS_DB.lock().unwrap();
    // If KEY found in DB, return value of that key
    let reply = if let Value::Bulk(s) = &v[0] {
        redis_db
            .get(s)
            .map(|e| Value::Bulk(e.to_string()))
            .unwrap_or(Value::Null)
    } else {
        // If value not found agains a key, return a Null value
        Value::Null
    };
    Ok(reply)
}

pub fn handle_set(v: Vec<Value>) -> Result<Value, Value> {
    let v = v.iter().skip(1).collect::<Vec<_>>();
    // first arg SET, second arg a key, both mandatory, if not provided returnerr
    if v.is_empty() || v.len() < 2 {
        return Err(Value::Error(
            "Expected 2 arguments for SET command".to_string(),
        ));
    }
    match (&v[0], &v[1]) {
        (Value::Bulk(k), Value::Bulk(v)) => {
            let _ = RUDIS_DB
                .lock()
                .unwrap()
                .insert(k.to_string(), v.to_string());
        }
        _ => unimplemented!("SET not implemented for {:?}", v),
    }

    Ok(Value::String("OK".to_string()))
}