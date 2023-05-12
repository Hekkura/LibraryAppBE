// ? TODO: Create bulk function
// pub async fn convert_to_bulk(operation_type: BulkTypes, data: &Value) -> {

use actix_web::HttpResponse;
// }
use reqwest::StatusCode;
use serde_json::{Value, json};

use crate::{handlers::{errors::ErrorTypes, libs::index_name_builder}, actions::EClientTesting};

pub async fn get_document(index: &str, document_id: &str, retrieve_fields: &Option<String>, client: &EClientTesting) -> Result<(StatusCode, Value), (StatusCode, ErrorTypes)>{
    let resp = client.get_document(index, document_id, retrieve_fields).await.unwrap();

    let status_code = resp.status_code();
    
    if !status_code.is_success() {
        let error = match status_code{
            StatusCode::NOT_FOUND => ErrorTypes::DocumentNotFound(document_id.to_string()),
            _ => ErrorTypes::Unknown
        };
        return Err((status_code, error));
    }

    let json_resp = resp.json::<Value>().await.unwrap();

    Ok((status_code, json_resp))
}

pub async fn document_search(app_id: &str, index: &str, body: &Value, from: &Option<i64>, count: &Option<i64>, client: &EClientTesting) -> Result<(StatusCode, Value), HttpResponse> {

    let name = index_name_builder(app_id, index);
    
    let time_now = std::time::Instant::now();
    // This takes a while to get a response on large "count" value, perhaps there is a better way?
    // Benchmarking with returning 10000 (worst case) of movie_records yields 300ms to 400ms on release mode, 400ms to 600ms on debug mode
    let resp = client.search_index(&name, body, from, count).await.unwrap();
    println!("Initial Request elapsed: {:#?}ms", time_now.elapsed().as_millis());

    let status = resp.status_code();

    if !status.is_success() {
        let error = match status {
            StatusCode::NOT_FOUND => ErrorTypes::IndexNotFound(index.to_owned()).to_string(),
            StatusCode::BAD_REQUEST => ErrorTypes::BadDataRequest.to_string(),
            _ => ErrorTypes::Unknown.to_string()
        };

        return Err(HttpResponse::build(status).json(json!({"error": error})));
    };

    let receive = std::time::Instant::now();
    // This takes a while to get a response on large "count" value, perhaps there is a better way?
    // Benchmarking with returning 10000 (worst case) of movie_records yields 500ms to 600ms on release mode, 1300ms to 1400ms on debug mode
    // 1. This takes in the body response because the connection is still active
    // 2. This parses the input into json
    let text_resp = resp.text().await.unwrap();
    println!("Body Response elapsed {:#?}ms", receive.elapsed().as_millis());

    let convert = std::time::Instant::now();
    let json_resp = serde_json::from_str(&text_resp).unwrap();
    println!("Conversion elapsed {:#?}ms", convert.elapsed().as_millis());

    Ok((status, json_resp))
}

pub fn search_body_builder(search_term: &Option<String>, search_in: &Option<Vec<String>>, retrieve_field: &Option<String>) -> Value {
    let fields_to_search = search_in.to_owned().unwrap_or(vec!["*".to_string()]);

    let fields_to_return = match retrieve_field {
        Some(val) => val.split(',').map(|x| x.trim().to_string()).collect(),
        None => vec!["*".to_string()],
    };

    
    // Returns everything
    let mut body = json!({
        "_source": {
            "includes": fields_to_return
        },
        "query": {
            "match_all": {} 
        },
    });
        if let Some(term) = search_term {

            body = json!({
                "_source": {
                    "includes": fields_to_return
                },
                "query": {
                        "query_string": {
                            "query": term,
                            "type": "cross_fields",
                            "fields": fields_to_search,
                            "minimum_should_match": "75%"
                        }
                    }
                })
        }
    body
}