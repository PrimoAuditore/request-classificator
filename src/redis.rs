
mod classification {

}

pub mod part_register {
    use std::error::Error;
    use std::time::{SystemTime, UNIX_EPOCH};
    use fizzy_commons::redis::client::create_client;
    use log::error;
    use redis::{Commands, RedisResult, Value};
    use crate::structs::part_request::PartRequest;

    pub fn create_part_request(origin: &str, reference:&str) -> Result<PartRequest, String>{

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

        let part_request = PartRequest::new(&request_id, origin, reference,timestamp.as_str());
        let redis_fields = part_request.get_redis_list();


        let res: RedisResult<Value> = con.hset_multiple(request_id, &redis_fields);

        if res.is_err() {
            error!("{}", res.as_ref().unwrap_err());
            return Err(format!("{}", res.as_ref().unwrap_err().to_string()))
        }

        Ok(part_request)

    }
}


#[cfg(test)]
mod classification_test{

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
