use crate::structs::constants::get_year_encodings;
use fizzy_commons::redis::client::create_client;
use fizzy_commons::shared_structs::user_management::User;
use log::{debug, info};
use redis::Value::Bulk;
use redis::{RedisResult, Value};
use std::collections::HashMap;

const MAKE_STATUS_ID: i32 = 3;
const DESCRIPTION_STATUS_ID: i32 = 9;
const VIN_STATUS_ID: i32 = 7;
const MODEL_STATUS_ID: i32 = 5;
const VIN_YEAR_DIGIT: i32 = 9; // 10th digit of the VIN

pub struct VinInformation {
    make: String,
    model: String,
    year: String,
    plant: String,
}

#[derive(Clone, Debug)]
pub struct TrackerStep {
    pub(crate) tracker_id: String,
    pub(crate) timestamp: String,
    pub(crate) id: String,
    pub(crate) status: String,
    pub(crate) value: String,
    pub(crate) attached_files: String,
    pub(crate) message_reference: String,
}

pub struct TrackerParam {
    pub tracker_id: String,
}

impl TrackerStep {
    pub fn parse_from_redis(&mut self, register: &Vec<Value>) -> TrackerStep {
        let mut values: HashMap<String, String> = HashMap::new();

        // Parse bulk into key-val hashmap
        let mut param_name = "".to_string();
        for (index, elem) in register.iter().enumerate() {
            let string_val = match elem {
                Value::Data(val) => String::from_utf8(val.clone()),
                _ => {
                    panic!("Unexpected value")
                }
            }
            .unwrap();

            if index % 2 == 0 {
                param_name = string_val;
            } else {
                values.insert(param_name.to_string(), string_val);
            }
        }

        // Add parsed values to struct
        self.value = String::from(
            values
                .get("value")
                .expect("Expected parameter value wasn't found"),
        );
        values.remove("value");
        self.status = String::from(
            values
                .get("status")
                .expect("Expected parameter status wasn't found"),
        );
        values.remove("status");
        self.tracker_id = String::from(
            values
                .get("tracker_id")
                .expect("Expected parameter tracker_id wasn't found"),
        );
        values.remove("tracker_id");
        self.timestamp = String::from(
            values
                .get("timestamp")
                .expect("Expected parameter timestamp wasn't found"),
        );
        values.remove("timestamp");
        self.attached_files = String::from(
            values
                .get("attached_files")
                .expect("Expected parameter attached_files wasn't found"),
        );
        values.remove("attached_files");
        self.message_reference = String::from(
            values
                .get("message_reference")
                .expect("Expected parameter message_reference wasn't found"),
        );
        values.remove("message_reference");

        // Fails it there are values in the hashmap that are not parsed into the tracker step struct
        if values.iter().len() > 0 {
            panic!("Found more values than expected");
        }

        self.clone()
    }
}

impl Default for TrackerStep {
    fn default() -> Self {
        TrackerStep {
            tracker_id: "".to_string(),
            timestamp: "".to_string(),
            id: "".to_string(),
            status: "".to_string(),
            value: "".to_string(),
            attached_files: "".to_string(),
            message_reference: "".to_string(),
        }
    }
}

pub trait Source {
    fn get_vin(reference: &str) -> Option<String>;
    fn get_description(reference: &str) -> Option<String>;
    fn get_attached_files(reference: &str) -> Option<String>;

    fn decode_year(vin: &str) -> (String, String) {
        let digit_value = vin
            .chars()
            .nth(VIN_YEAR_DIGIT as usize)
            .unwrap()
            .to_string();
        debug!("Year digit is {digit_value}");
        let year_table = get_year_encodings();
        let possible_years = year_table.get(&digit_value.to_ascii_uppercase());

        if possible_years.is_none() {
            panic!("Year digit is not a valid digit");
        }

        let clone = possible_years.unwrap().clone();
        debug!("clone: {clone:?}");
        clone
    }

    fn get_make(tracker_id: &str) -> Option<String>;

    fn get_requestor(reference: &str) -> Result<User, String>;

    fn get_model(tracker_id: &str) -> Option<String>;
}

#[derive(Clone)]
pub struct WhatsappSource {}
impl Source for WhatsappSource {
    fn get_requestor(reference: &str) -> Result<User, String> {
        let user = User::from_phone_number(reference);
        user
    }

    fn get_vin(tracker_id: &str) -> Option<String> {
        println!("Searching vin");
        let client = create_client().expect("Redis client couldnt be created.");
        let mut con = client.get_connection().unwrap();

        let res: RedisResult<Value> = redis::cmd("FT.SEARCH")
            .arg("trackerSteps")
            .arg(format!(
                "@tracker_id:{} @status:{}",
                tracker_id, VIN_STATUS_ID
            ))
            .query(&mut con);

        if res.is_err() {
            if res
                .as_ref()
                .unwrap_err()
                .to_string()
                .contains("response was [int(0)]")
            {
                // No record found
                // return Err("No records found".to_string())
            } else {
                // Any other error
                // return Err(res.unwrap_err().to_string())
            }
        }

        println!("Result: {:?}", res.as_ref().unwrap());

        // Find 'value' field on search
        let mut vin_value: Option<String> = None;
        for x in res.unwrap().as_sequence().unwrap() {
            if let Bulk(register) = x {
                let mut param_name = "".to_string();
                for (index, elem) in register.iter().enumerate() {
                    let string_val = match elem {
                        Value::Data(val) => String::from_utf8(val.clone()),
                        _ => {
                            panic!("Unexpected value")
                        }
                    }
                    .unwrap();

                    if index % 2 == 0 {
                        param_name = string_val;
                    } else {
                        if param_name == "value" {
                            println!("Found value param");
                            println!("{}", string_val);
                            vin_value = Some(string_val);
                            break;
                        };
                    }
                }
            }
        }

        // Fails if no field value is found, or value is empty
        vin_value
    }

    fn get_make(tracker_id: &str) -> Option<String> {
        let client = create_client().expect("Redis client couldnt be created.");
        let mut con = client.get_connection().unwrap();

        let res: RedisResult<Value> = redis::cmd("FT.SEARCH")
            .arg("trackerSteps")
            .arg(format!(
                "@tracker_id:{} @status:{}",
                tracker_id, MAKE_STATUS_ID
            ))
            .query(&mut con);

        debug!("{:?}", res.as_ref().unwrap());

        let mut make: Option<String> = None;
        for val in res.unwrap().as_sequence().unwrap() {
            if let Bulk(register) = val {
                let step = TrackerStep::default().parse_from_redis(register);

                debug!("step value: {}", &step.value);
                make = Some(step.value);
            }
        }
        make
    }

    fn get_description(tracker_id: &str) -> Option<String> {
        let client = create_client().expect("Redis client couldnt be created.");
        let mut con = client.get_connection().unwrap();

        let res: RedisResult<Value> = redis::cmd("FT.SEARCH")
            .arg("trackerSteps")
            .arg(format!(
                "@tracker_id:{} @status:{}",
                tracker_id, DESCRIPTION_STATUS_ID
            ))
            .query(&mut con);

        debug!("{:?}", res.as_ref().unwrap());

        let mut description: Option<String> = None;
        for val in res.unwrap().as_sequence().unwrap() {
            if let Bulk(register) = val {
                let step = TrackerStep::default().parse_from_redis(register);

                debug!("step value: {}", &step.value);
                description = Some(step.value);
            }
        }
        description
    }

    fn get_attached_files(tracker_id: &str) -> Option<String> {
        let client = create_client().expect("Redis client couldnt be created.");
        let mut con = client.get_connection().unwrap();

        let res: RedisResult<Value> = redis::cmd("FT.SEARCH")
            .arg("trackerSteps")
            .arg(format!(
                "@tracker_id:{} @status:{}",
                tracker_id, DESCRIPTION_STATUS_ID
            ))
            .query(&mut con);

        debug!("{:?}", res.as_ref().unwrap());

        let mut attached_files: Option<String> = None;
        for val in res.unwrap().as_sequence().unwrap() {
            if let Bulk(register) = val {
                let step = TrackerStep::default().parse_from_redis(register);

                debug!("step value: {}", &step.attached_files);
                attached_files = Some(step.attached_files);
            }
        }
        attached_files
    }

    fn get_model(tracker_id: &str) -> Option<String> {
        let client = create_client().expect("Redis client couldnt be created.");
        let mut con = client.get_connection().unwrap();

        let res: RedisResult<Value> = redis::cmd("FT.SEARCH")
            .arg("trackerSteps")
            .arg(format!(
                "@tracker_id:{} @status:{}",
                tracker_id, MAKE_STATUS_ID
            ))
            .query(&mut con);

        debug!("{:?}", res.as_ref().unwrap());

        let mut model: Option<String> = None;
        for val in res.unwrap().as_sequence().unwrap() {
            if let Bulk(register) = val {
                let step = TrackerStep::default().parse_from_redis(register);

                debug!("step value: {}", &step.value);
                model = Some(step.value);
            }
        }
        model
    }
}

pub mod classification {
    use redis::{from_redis_value, FromRedisValue, RedisResult};
    use redis::{RedisError, Value};
    use std::collections::{HashMap, HashSet};
    use std::io::ErrorKind;

    #[derive(Debug)]
    pub struct Label {
        pub id: String,
        pub name: String,
        pub parent: String,
    }

    impl Label {
        pub fn new(id: &str, name: &str, parent: &str) -> Label {
            Label {
                id: String::from(id),
                name: String::from(name),
                parent: String::from(parent),
            }
        }
    }

    impl FromRedisValue for Label {
        fn from_redis_values(items: &[Value]) -> redis::RedisResult<Vec<Self>> {
            let mut parsed_values: Vec<Self> = vec![];

            for item in items {
                if let Value::Bulk(val) = item {
                    let value: RedisResult<Label> = from_redis_value(&Value::Bulk(vec![item.clone()]));

                    if value.is_err() {
                        return Err(RedisError::from(std::io::Error::new(
                            ErrorKind::Other,
                            "Value couldn't be parsed",
                        )));
                    }

                    parsed_values.push(value.unwrap())
                } else {
                    // TODO: implement logging for library
                }
            }

            if parsed_values.is_empty() {
                return Err(RedisError::from(std::io::Error::new(
                    ErrorKind::Other,
                    "No values to parse",
                )));
            }

            Ok(parsed_values)
        }

        fn from_redis_value(v: &redis::Value) -> redis::RedisResult<Self> {
            if let Value::Bulk(bulk) = v {
                for value in bulk {
                    if let Value::Bulk(register) = value {
                        let mut fields: HashMap<String, String> = HashMap::new();
                        let mut verification_set: HashSet<String> = HashSet::new();

                        // Parse bulk into key-val hashmap
                        let mut param_name = "".to_string();
                        for (index, elem) in register.iter().enumerate() {
                            let string_val = match elem {
                                Value::Data(val) => String::from_utf8(val.clone()),
                                _ => {
                                    return Err(RedisError::from(std::io::Error::new(
                                        ErrorKind::Other,
                                        "Unexpected Value type received",
                                    )));
                                }
                            }
                            .unwrap();

                            if index % 2 == 0 {
                                param_name = string_val;
                                verification_set.insert(param_name.clone());
                            } else {
                                fields.insert(param_name.to_string(), string_val);
                            }
                        }

                        let mut label: Label = Label::new("", "", "");

                        // Verifiy set is empty
                        label.id = fields.get("id").expect("Id field not found").to_string();
                        verification_set.remove("id");

                        label.name = fields
                            .get("name")
                            .expect("name field not found")
                            .to_string();
                        verification_set.remove("name");

                        label.parent = fields
                            .get("parent")
                            .expect("last_name field not found")
                            .to_string();
                        verification_set.remove("parent");

                        if !verification_set.is_empty() {
                            return Err(RedisError::from(std::io::Error::new(
                                ErrorKind::Other,
                                "Aditional unexpected values found",
                            )));
                        }

                        return Ok(label);
                    }
                }
            } else {
                // TODO: find better way to represent conversion error
                return Err(RedisError::from(std::io::Error::new(
                    ErrorKind::Other,
                    "Error converting struct from redis value",
                )));
            }
            Err(RedisError::from(std::io::Error::new(
                ErrorKind::Other,
                "Not found",
            )))
        }
    }
}

pub mod part_request {
    use crate::redis::part_register::{
        set_request_details, set_request_requestor, set_request_vehicle_information,
    };
    use crate::structs::Source;
    use fizzy_commons::shared_structs::user_management::User;
    use log::error;

    pub struct VehicleData {
        make: Option<String>,
        model: Option<String>,
        vin: Option<String>,
        year: Option<String>,
    }

    impl VehicleData {
        pub fn get_redis_fields(&self) -> Vec<(String, String)> {
            let mut vec: Vec<(String, String)> = vec![];

            vec.push((
                String::from("make"),
                String::from(self.make.as_ref().unwrap()),
            ));

            vec.push((
                String::from("model"),
                String::from(self.model.as_ref().unwrap()),
            ));

            vec.push((
                String::from("year"),
                String::from(self.year.as_ref().unwrap()),
            ));

            vec.push((
                String::from("vin"),
                String::from(self.vin.as_ref().unwrap()),
            ));
            vec
        }
    }

    // VEHICLE DATA BUILDER
    #[derive(Clone)]
    pub struct VehicleDataBuilder<T> {
        pub make: Option<String>,
        pub model: Option<String>,
        pub vin: Option<String>,
        pub year: Option<String>,
        pub source: Option<T>,
    }

    impl<T> Default for VehicleDataBuilder<T> {
        fn default() -> Self {
            VehicleDataBuilder {
                make: None,
                model: None,
                vin: None,
                year: None,
                source: None,
            }
        }
    }

    impl<T: Source> VehicleDataBuilder<T> {
        pub fn build(self) -> VehicleData {
            VehicleData {
                make: self.make,
                model: self.model,
                vin: self.vin,
                year: self.year,
            }
        }

        pub fn vin(&mut self, reference: &str) -> &mut Self {
            let vin = T::get_vin(reference);
            if vin.is_none() {
                error!("Vin wasnt found");
            }
            self.vin = vin;
            self
        }

        pub fn make(&mut self, tracker_id: &str) -> &mut Self {
            let make = T::get_make(tracker_id);
            if make.is_none() {
                error!("make wasnt found");
            }
            self.make = make;
            self
        }

        pub fn model(&mut self, tracker_id: &str) -> &mut Self {
            let model = T::get_model(tracker_id);
            if model.is_none() {
                error!("model wasnt found");
            }
            self.model = model;
            self
        }

        pub fn year(&mut self) -> &mut Self {
            if self.vin.is_none() {
                panic!("Vin has to be defined before decoding it");
            }
            let possible_years = T::decode_year(&self.vin.as_ref().unwrap());
            self.year = Some(format!("{},{}", possible_years.0, possible_years.1));

            self
        }
    }

    // PART REQUEST
    pub struct PartRequest {
        pub id: String,
        pub origin: String,
        pub origin_reference: String,
        pub timestamp: String,
        pub vehicle: Option<VehicleData>,
        pub requestor: Option<Requestor>,
        pub details: Option<RequestDetails>,
    }

    impl PartRequest {
        pub fn new(id: &str, origin: &str, origin_reference: &str, timestamp: &str) -> PartRequest {
            PartRequest {
                id: id.to_string(),
                origin: origin.to_string(),
                origin_reference: origin_reference.to_string(),
                timestamp: timestamp.to_string(),
                vehicle: None,
                requestor: None,
                details: None,
            }
        }

        pub fn get_redis_list(&self) -> Vec<(String, String)> {
            vec![
                (String::from("id"), String::from(&self.id)),
                (String::from("origin"), String::from(&self.origin)),
                (
                    String::from("origin_reference"),
                    String::from(&self.origin_reference),
                ),
                (String::from("timestamp"), String::from(&self.timestamp)),
            ]
        }

        pub fn set_request_details(&mut self, request_details: RequestDetails) {
            let data = set_request_details(&self.id, &request_details);
            self.details = Some(request_details)
        }

        pub fn set_vehicle_data(&mut self, vehicle_data: VehicleData) {
            let data = set_request_vehicle_information(&self.id, &vehicle_data);
            self.vehicle = Some(vehicle_data)
        }

        pub fn set_requestor(&mut self, requestor: Requestor) {
            let data = set_request_requestor(&self.id, &requestor);
            self.requestor = Some(requestor)
        }
    }

    pub struct RequestDetails {
        pub description: Option<String>,
        pub attached_files: Option<String>,
    }

    impl RequestDetails {
        pub fn new() -> RequestDetails {
            RequestDetails {
                description: None,
                attached_files: None,
            }
        }

        pub fn get_redis_fields(&self) -> Vec<(String, String)> {
            let mut vec: Vec<(String, String)> = vec![];

            vec.push((
                String::from("description"),
                String::from(self.description.as_ref().unwrap()),
            ));
            vec.push((
                String::from("attached_files"),
                String::from(self.attached_files.as_ref().unwrap()),
            ));
            vec
        }
    }

    // Requestor
    pub struct Requestor {
        pub user_id: String,
    }

    impl Requestor {
        pub fn get_redis_fields(&self) -> Vec<(String, String)> {
            let mut vec: Vec<(String, String)> = vec![];

            vec.push((String::from("user_id"), String::from(&self.user_id)));

            vec
        }
    }

    pub struct RequestorBuilder<T> {
        pub requestor: Option<User>,
        pub source: Option<T>,
    }

    impl<T> Default for RequestorBuilder<T> {
        fn default() -> Self {
            RequestorBuilder {
                requestor: None,
                source: None,
            }
        }
    }

    impl<T: Source> RequestorBuilder<T> {
        pub fn requestor(&mut self, reference: &str) -> &mut Self {
            let requestor = T::get_requestor(reference);

            if requestor.is_err() {
                panic!("Requestor couldnt be retrieved");
            }
            self.requestor = Some(requestor.unwrap());
            self
        }

        pub fn build(&self) -> Result<Requestor, String> {
            if self.requestor.is_none() {
                return Err(String::from("Requestor cannot be none"));
            }

            let id = String::from(&self.requestor.as_ref().unwrap().id);

            Ok(Requestor { user_id: id })
        }
    }

    // Request Details
    pub struct RequestDetailsBuilder<T> {
        pub description: Option<String>,
        pub attached_files: Option<String>,
        pub source: Option<T>,
    }

    impl<T> Default for RequestDetailsBuilder<T> {
        fn default() -> Self {
            RequestDetailsBuilder {
                description: None,
                attached_files: None,
                source: None,
            }
        }
    }

    impl<T: Source> RequestDetailsBuilder<T> {
        pub fn description(&mut self, reference: &str) -> &mut Self {
            let desc = T::get_description(reference);

            if desc.is_none() {
                error!("Description couldnt be found")
            }

            self.description = desc;
            self
        }

        pub fn attached_files(&mut self, reference: &str) -> &mut Self {
            let attached_files = T::get_attached_files(reference);

            if attached_files.is_none() {
                error!("Attached files couldnt be found")
            }

            self.attached_files = attached_files;
            self
        }

        pub fn build(self) -> RequestDetails {
            RequestDetails {
                description: self.description,
                attached_files: self.attached_files,
            }
        }
    }
}

pub mod constants {
    use std::collections::HashMap;

    pub fn get_year_encodings() -> HashMap<String, (String, String)> {
        let mut map: HashMap<String, (String, String)> = HashMap::new();

        map.insert(
            String::from("A"),
            (String::from("1980"), String::from("2010")),
        );
        map.insert(
            String::from("B"),
            (String::from("1981"), String::from("2011")),
        );
        map.insert(
            String::from("C"),
            (String::from("1982"), String::from("2012")),
        );
        map.insert(
            String::from("D"),
            (String::from("1983"), String::from("2013")),
        );
        map.insert(
            String::from("E"),
            (String::from("1984"), String::from("2014")),
        );
        map.insert(
            String::from("F"),
            (String::from("1985"), String::from("2015")),
        );
        map.insert(
            String::from("G"),
            (String::from("1986"), String::from("2016")),
        );
        map.insert(
            String::from("H"),
            (String::from("1987"), String::from("2017")),
        );
        map.insert(
            String::from("J"),
            (String::from("1988"), String::from("2018")),
        );
        map.insert(
            String::from("K"),
            (String::from("1989"), String::from("2019")),
        );
        map.insert(
            String::from("L"),
            (String::from("1990"), String::from("2020")),
        );
        map.insert(
            String::from("M"),
            (String::from("1991"), String::from("2021")),
        );
        map.insert(
            String::from("N"),
            (String::from("1992"), String::from("2022")),
        );
        map.insert(
            String::from("P"),
            (String::from("1993"), String::from("2023")),
        );
        map.insert(
            String::from("R"),
            (String::from("1994"), String::from("2024")),
        );
        map.insert(
            String::from("S"),
            (String::from("1995"), String::from("2025")),
        );
        map.insert(
            String::from("T"),
            (String::from("1996"), String::from("2026")),
        );
        map.insert(
            String::from("V"),
            (String::from("1997"), String::from("2027")),
        );
        map.insert(
            String::from("W"),
            (String::from("1998"), String::from("2028")),
        );
        map.insert(
            String::from("X"),
            (String::from("1999"), String::from("2029")),
        );
        map.insert(
            String::from("Y"),
            (String::from("2000"), String::from("2030")),
        );
        map.insert(
            String::from("1"),
            (String::from("2001"), String::from("2031")),
        );
        map.insert(
            String::from("2"),
            (String::from("2002"), String::from("2032")),
        );
        map.insert(
            String::from("3"),
            (String::from("2003"), String::from("2033")),
        );
        map.insert(
            String::from("4"),
            (String::from("2004"), String::from("2034")),
        );
        map.insert(
            String::from("5"),
            (String::from("2005"), String::from("2035")),
        );
        map.insert(
            String::from("6"),
            (String::from("2006"), String::from("2036")),
        );
        map.insert(
            String::from("7"),
            (String::from("2007"), String::from("2037")),
        );
        map.insert(
            String::from("8"),
            (String::from("2008"), String::from("2038")),
        );
        map.insert(
            String::from("9"),
            (String::from("2009"), String::from("2039")),
        );

        map
    }
}
