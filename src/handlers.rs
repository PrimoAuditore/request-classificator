use fizzy_commons::*;
use fizzy_commons::shared_structs::{MessageLog, StandardResponse};
use log::{error, debug};
use crate::redis::part_register::create_part_request;

use crate::structs::part_request::VehicleDataBuilder;
use crate::structs::{Source, WhatsappSource};

pub fn new_request_received(notification: MessageLog){

    let mut builder: VehicleDataBuilder<WhatsappSource> = VehicleDataBuilder::<WhatsappSource>::default();
    let mut origin = "";

    // Determine source to be used to obtain data
    match notification.origin_system.parse::<u16>().unwrap() {
        3 => {
            origin = "WHATSAPP";
            let mut builder = VehicleDataBuilder::<WhatsappSource>::default();
        },
        _ => {
            error!("Unexpected system");
            panic!("Unexpected system")
        }
    };

    // Create Part request
    let part_request = create_part_request(origin, &notification.register_id).unwrap();

    // Get vin
    builder.vin(&notification.register_id);

    if builder.vin.is_none() {
        // TODO: Implement error
    }

    // Get possible year 
    builder.year();
    println!("{}", builder.year.as_ref().unwrap());

    // Get make
    builder.make(&notification.register_id);
    debug!("Make: {}", builder.make.as_ref().unwrap());

}

// pub fn outgoing_messages() -> Result<StandardResponse, StandardResponse>{
//     let mut response: StandardResponse = StandardResponse::new();
//     let mut errors = vec![];
//     let mut references = vec![];
//
//
// }

#[cfg(test)]
mod tests {

}
