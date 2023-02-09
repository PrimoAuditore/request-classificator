use fizzy_commons::shared_structs::{MessageLog, StandardResponse};
use log::{error, debug};
use crate::redis::part_register::{create_part_request};
use crate::redis::classification::get_label_childs;

use crate::structs::part_request::{VehicleDataBuilder, RequestDetailsBuilder, RequestorBuilder};
use crate::structs::{WhatsappSource};
use crate::helpers::{process_new_request};

pub fn new_request_received2(notification: MessageLog){

    match notification.origin_system.parse::<u16>().unwrap() {
        1 => {
            // Selected option in the menu
            
        }
        3 => {
            // User requested part
            let part_request = process_new_request(&notification);


        },
        4 => {
            // Self call
        }
        _ => {
            error!("Unexpected system");
            panic!("Unexpected system")
        }
    
    }
}

pub fn new_request_received(notification: MessageLog){

    get_label_childs();



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
    let mut part_request = create_part_request(origin, &notification.register_id).unwrap();

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


    // Get model 
    builder.model(&notification.register_id);
    debug!("Model: {}", builder.model.as_ref().unwrap());

    // Update request vehicle data
    let vehicle_data = builder.build();
    part_request.set_vehicle_data(vehicle_data);

    let mut details_builder: RequestDetailsBuilder<WhatsappSource> = RequestDetailsBuilder::default();

    // Get description
    details_builder.description(&notification.register_id);

    debug!("Description: {}", details_builder.description.as_ref().unwrap());

    // Get attached files

    details_builder.attached_files(&notification.register_id);

    debug!("Attached files: {}", details_builder.attached_files.as_ref().unwrap());

    let details = details_builder.build();

    part_request.set_request_details(details);

    let mut requestor_builder: RequestorBuilder<WhatsappSource> = RequestorBuilder::default();
    let requestor = requestor_builder
        .requestor(&notification.phone_number)
        .build();

    if requestor.is_err() {
        error!("Error obtaining requestor");

    }

    part_request.set_requestor(requestor.unwrap())


    //println!("user: {}", user.name);


}

// pub fn outgoing_messages() -> Result<StandardResponse, StandardResponse>{
//     let mut response: StandardResponse = StandardResponse::new();
//     let mut errors = vec![];
//  gt   let mut references = vec![];
//
//
// }

#[cfg(test)]
mod tests {

}
