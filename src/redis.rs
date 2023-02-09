pub mod classification {
    use crate::structs::classification::Label;
    use fizzy_commons::redis::client::create_client;
    use fizzy_commons::redis::search::QueryBuilder;

    pub fn get_label_childs() {
        let mut query: QueryBuilder<Vec<Label>> = QueryBuilder::default();

        let base_parent = String::from("1");
        let base_parent_field = String::from("parent");

        query
            .index("label-parent-search".to_string())
            .arg(base_parent_field, base_parent);

        let mut client = create_client().unwrap();
        let res = query.search(&client);

        if res.is_ok() {
            println!("{res:?}");
        } else {
            println!("{}", res.unwrap_err().to_string())
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

        if res.is_err(){
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

        let part_request = PartRequest::new(&uuid, origin, reference, timestamp.as_str());
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
    fn already_added_label(){
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
