mod classification {}

pub mod part_register {
    use crate::structs::part_request::{PartRequest, VehicleData, RequestDetails};
    use fizzy_commons::redis::client::create_client;
    use log::{error, debug};
    use redis::{Commands, RedisResult, Value};
    use std::error::Error;
    use std::time::{SystemTime, UNIX_EPOCH};

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
        vehicle_data: VehicleData,
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
        details: RequestDetails,
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
}

#[cfg(test)]
mod classification_test {

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
