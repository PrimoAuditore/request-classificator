pub mod classification {
    use std::fmt::format;

    use crate::structs::classification::Label;
    use crate::structs::part_request::PartRequest;
    use fizzy_commons::redis::client::create_client;
    use fizzy_commons::redis::search::QueryBuilder;
    use log::{debug, error};
    use redis::{Commands, RedisResult, Value};

    pub fn get_all_labels() -> Result<Vec<Label>, String>{
        
        debug!("Getting all labels");
        let mut query: QueryBuilder<Vec<Label>> = QueryBuilder::default();

        query
            .index("label-parent-search".to_string());

        let mut client = create_client().unwrap();
        let res = query.all(&client);

        if res.is_err() {
            return Err(res.unwrap_err().to_string());
        }

        Ok(res.unwrap())
    }

    pub fn get_pending_classification_requests() -> Result<Vec<PartRequest>, String> {
        // TODO: Think on a concurrent solution
        let mut query: QueryBuilder<Vec<PartRequest>> = QueryBuilder::default();

        query
            .index("request-search".to_string())
            .arg("classified".to_string(), "PENDING".to_string());

        let mut client = create_client().unwrap();
        let res = query.search(&client);

        if res.is_err() {
            let error = format!(
                "Error obtaining pending requests: {}",
                res.as_ref().unwrap_err().to_string()
            );
            return Err(error);
        }

        Ok(res.unwrap())
    }

    pub fn append_label(request_id: &str, label_code: &str) -> Result<(), String> {
        let client = create_client().unwrap();
        let mut con = client.get_connection().unwrap();

        let key = format!("part-request:{request_id}:labels");

        let res: RedisResult<Value> = con.sadd(key, label_code);

        if res.is_err() {
            let err_msg = format!("Error appending label: {}", res.as_ref().unwrap_err());
            error!("{}", &err_msg);
            return Err(err_msg);
        }

        Ok(())
    }

    pub fn remove_label(request_id: &str, label_code: &str) -> Result<(), String> {
        // Id 0 is a symbolic id for base labels
        if label_code == "0" {
            return Ok(());
        }

        let client = create_client().unwrap();
        let mut con = client.get_connection().unwrap();

        let key = format!("part-request:{request_id}:labels");

        debug!("Removing label {label_code} on key {key}");

        let res: RedisResult<Value> = con.srem(key, label_code);

        if res.is_err() {
            let err_msg = format!("Error removing label: {}", res.as_ref().unwrap_err());
            error!("{}", &err_msg);
            return Err(err_msg);
        }

        let res2 = res.unwrap();
        if let Value::Int(code) = res2 {
            if code != 1 {
                let err_msg =
                    format!("Code {code}, error removing label, verify that label is assigned to request");
                error!("{}", &err_msg);
                return Err(err_msg);
            }
        };

        Ok(())
    }

    pub fn get_label_childs(label_id: &str) -> Result<Vec<Label>, String> {
        let mut query: QueryBuilder<Vec<Label>> = QueryBuilder::default();

        let base_parent = String::from(label_id);
        let base_parent_field = String::from("parent");

        query
            .index("label-parent-search".to_string())
            .arg(base_parent_field, base_parent);

        let mut client = create_client().unwrap();
        let res = query.search(&client);

        if res.is_err() {
            return Err(res.unwrap_err().to_string());
        }

        Ok(res.unwrap())
    }

    pub fn get_request_labels(request_id: &str) -> Result<Vec<Label>, String>{
        let mut client = create_client().unwrap();
        let mut con = client.get_connection().unwrap();
        let mut list: Vec<Label> = vec![];


        let key = format!("part-request:{request_id}:labels");

        debug!("Gettings labels for {key}");

        let res: RedisResult<Vec<String>> = con.smembers(key);
        if res.is_err() {
            let err = format!("Error getting request labels: {}", res.unwrap_err());
            error!("{}", err);
            return Err(err)
        }

        debug!("Found labels ids: {:?}", res.as_ref().unwrap());

        for label in res.unwrap(){
            list.push(Label::get(&label).expect("Failed to get label: "))
        }

        Ok(list)
    }
    pub fn get_label(id: &str) -> Result<Label, String> {
        let mut client = create_client().unwrap();
        let mut con = client.get_connection().unwrap();

        let key = format!("part-label:{id}");

        let res: RedisResult<Label> = con.hgetall(&key);

        if res.is_err() {
            let err = format!(
                "err getting label {}: {}",
                &key,
                res.as_ref().unwrap_err().to_string()
            );
            error!("{}",err);
            return Err(err) 
        }

       Ok(res.unwrap()) 
    }
}

pub mod common {
    use fizzy_commons::redis::client::create_client;
    use log::{debug, error};
    use redis::Commands;
    use redis::RedisError;
    use redis::RedisResult;
    use redis::Value;

    pub fn key_exists(key: &str) -> bool {
        let client = create_client().unwrap();
        let mut con = client.get_connection().unwrap();

        let res: RedisResult<Value> = con.exists(key);

        match res.unwrap() {
            Value::Int(response_code) => response_code == 1,
            _ => {
                error!("Unexpected response type received");
                panic!("Unexpected response type received")
            }
        }
    }

    pub fn get_user_mode(phone_number: &str) -> Result<u16, String> {
        let client = create_client().unwrap();
        let mut con = client.get_connection().unwrap();

        let mode: RedisResult<String> = con.hget(format!("selected-mode:{}", phone_number), "mode");

        if mode.is_err() {
            let is_nil = is_nil(mode.as_ref().unwrap_err());

            if is_nil {
                error!("Key has no value");
                return Err("Key has no value".to_string());
            } else {
                error!("{}", mode.as_ref().unwrap_err().to_string());
                return Err(String::from("Other error"));
            };
        }

        let parsed_mode = mode.unwrap().parse::<u16>().unwrap();
        Ok(parsed_mode)
    }

    pub fn is_nil(error: &RedisError) -> bool {
        if error.to_string().contains("response was nil") {
            true
        } else {
            false
        }
    }
}

pub mod part_register {
    use crate::structs::part_request::{PartRequest, RequestDetails, Requestor, VehicleData};
    use fizzy_commons::redis::client::create_client;
    use log::{debug, error};
    use redis::{Commands, RedisResult, Value};
    use std::time::{SystemTime, UNIX_EPOCH};

    pub fn append_label(part_request_id: &str, label_id: &str) -> Result<Value, String> {
        let client = create_client().unwrap();
        let mut con = client.get_connection().unwrap();

        // Key
        let key = format!("part-request:{}:labels", part_request_id);

        let res: RedisResult<Value> = con.sadd(key, label_id);

        if res.is_err() {
            // TODO
        }

        Ok(res.unwrap())
    }

    pub fn create_part_request(origin: &str, reference: &str) -> Result<PartRequest, String> {
        let client = create_client().unwrap();
        let mut con = client.get_connection().unwrap();

        // Request ID
        let uuid = uuid::Uuid::new_v4().to_string();
        let request_id = format!("part-request:{}", uuid);

        // Timestamp
        let timestamp = match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(n) => n.as_millis().to_string(),
            Err(_) => panic!("SystemTime before UNIX EPOCH!"),
        };

        let part_request =
            PartRequest::new(&uuid, origin, reference, timestamp.as_str(), "PENDING");
        let redis_fields = part_request.get_redis_list();

        let res: RedisResult<Value> = con.hset_multiple(request_id, &redis_fields);

        if res.is_err() {
            error!("{}", res.as_ref().unwrap_err());
            return Err(format!("{}", res.as_ref().unwrap_err().to_string()));
        }

        Ok(part_request)
    }

    pub fn set_request_vehicle_information(
        part_request_id: &str,
        vehicle_data: &VehicleData,
    ) -> Result<(), String> {
        // Get Client
        let client = create_client().unwrap();
        let mut con = client.get_connection().unwrap();

        // get redis field tuples from struct
        let redis_fields = vehicle_data.get_redis_fields();

        // Key
        let key = format!("part-request:{}:vehicle", part_request_id);

        let res: RedisResult<Value> = con.hset_multiple(&key, &redis_fields);

        if res.is_err() {
            return Err(res.as_ref().unwrap_err().to_string());
        }

        debug!("Vehicle data added succesfully {}", &key);
        Ok(())
    }

    pub fn set_request_details(
        part_request_id: &str,
        details: &RequestDetails,
    ) -> Result<(), String> {
        // Get Client
        let client = create_client().unwrap();
        let mut con = client.get_connection().unwrap();

        // get redis field tuples from struct
        let redis_fields = details.get_redis_fields();

        // Key
        let key = format!("part-request:{}:request-details", part_request_id);

        let res: RedisResult<Value> = con.hset_multiple(&key, &redis_fields);

        if res.is_err() {
            return Err(res.as_ref().unwrap_err().to_string());
        }

        debug!("Request details added succesfully {}", &key);
        Ok(())
    }

    pub fn set_request_requestor(
        part_request_id: &str,
        requestor: &Requestor,
    ) -> Result<(), String> {
        let client = create_client().unwrap();
        let mut con = client.get_connection().unwrap();

        let redis_fields = requestor.get_redis_fields();

        // Key
        let key = format!("part-request:{}:requestor", part_request_id);

        let res: RedisResult<Value> = con.hset_multiple(&key, &redis_fields);

        if res.is_err() {
            error!(
                "Error adding requestor to part request: {}",
                res.as_ref().unwrap_err().to_string()
            );
            return Err(res.as_ref().unwrap_err().to_string());
        }

        debug!("Requestor added succesfully {}", &key);
        Ok(())
    }
}

#[cfg(test)]
mod classification_test {

    // Passes if fails when trying to add a label that is already added.
    #[test]
    fn already_added_label() {
        assert!(false)
    }

    // Checks the index search returns a parseable label struct
    #[test]
    fn search_returns_label() {
        assert!(false)
    }

    // Passes if given a label that is parent of other labels, return those labels.
    #[test]
    fn sublabel_found() {
        assert!(false)
    }

    // Passes if given a label that is not parent of other labels, return an empty vec.
    #[test]
    fn sublabel_not_found() {
        assert!(false)
    }
}
