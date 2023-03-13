use crate::structs::WhatsappSource;
use fizzy_commons::shared_structs::MessageLog;
use log::{debug, error};

use crate::redis::classification::{get_label, remove_label};
use crate::redis::part_register::create_part_request;
use crate::structs::classification::Label;
use crate::structs::part_request::{
    PartRequest, RequestDetailsBuilder, RequestorBuilder, VehicleDataBuilder,
};

pub fn whatsapp_reference_exists(tracker_id: &str) -> bool {
    false
}

pub fn remove_label_tree(request_id: &str, label_id: &str, labels: &mut Vec<Label>) -> Result<(), String>{
    for label in labels.clone(){
        if label.parent == label_id {
            debug!("Checking if {} is child of {}", label.id, label_id );
            let res = remove_label(request_id, &label.id);

            if res.is_err() {
                let err = format!("Error deleting label: {}", res.unwrap_err());
                error!("{}", err);
                return Err(err)
            }

            let mut updated_list: Vec<Label> = labels.iter()
            .filter(|x| &x.id != &label.id).cloned().collect::<Vec<Label>>();
            remove_label_tree(request_id, &label.id, &mut updated_list);
        }
    }

    let res = remove_label(request_id, &label_id);

    if res.is_err() {
        let err = format!("Error deleting label: {}", res.unwrap_err());
        error!("{}", err);
        return Err(err)
    }

    Ok(())

}


pub fn print_type_name<T>(_: &T) -> String{
    std::any::type_name::<T>().to_string()
}


pub fn retrieve_label_tree(id: &str, labels: &mut Vec<Label>) -> Result<(), ()>{
    debug!("Retrieve label tree {id}");
    // Check if label exists
    if id == "0" {
        return Ok(());
    }

    if !Label::exists(id) {
        return Err(());
    }

    // Get label
    let parent_label: Label = Label::get(id).expect("Failed to get label: ");

    // Add label to label list
    labels.push(parent_label.clone());

    if parent_label.parent != "0" {
        retrieve_label_tree(&parent_label.parent, labels);
    }

    Ok(())
}

fn set_request_details(
    part_request: &mut PartRequest,
    notification: &MessageLog,
) -> Result<(), String> {
    let mut details_builder: RequestDetailsBuilder<WhatsappSource> =
        RequestDetailsBuilder::default();

    // Get description
    details_builder.description(&notification.register_id);
    debug!(
        "Description: {}",
        details_builder.description.as_ref().unwrap()
    );

    // Get attached files
    details_builder.attached_files(&notification.register_id);
    debug!(
        "Attached files: {}",
        details_builder.attached_files.as_ref().unwrap()
    );

    // Update request details
    let details = details_builder.build();
    part_request.set_request_details(details);

    Ok(())
}

fn set_requestor(part_request: &mut PartRequest, notification: &MessageLog) -> Result<(), String> {
    let mut requestor_builder: RequestorBuilder<WhatsappSource> = RequestorBuilder::default();
    let requestor = requestor_builder
        .requestor(&notification.phone_number)
        .build();

    if requestor.is_err() {
        error!("Error obtaining requestor");
        return Err("Error obtaining requestor".to_string());
    }

    part_request.set_requestor(requestor.unwrap());

    Ok(())
}
fn set_request_vehicle(
    part_request: &mut PartRequest,
    notification: &MessageLog,
) -> Result<(), String> {
    let mut builder = VehicleDataBuilder::<WhatsappSource>::default();

    // Get vin
    builder.vin(&notification.register_id);

    if builder.vin.is_none() {
        // TODO: Implement error
        return Err(String::from("VIN couldnt be found"));
    }

    // Get possible year
    builder.year();
    debug!("{}", builder.year.as_ref().unwrap());

    // Get make
    builder.make(&notification.register_id);
    debug!("Make: {}", builder.make.as_ref().unwrap());

    // Get model
    builder.model(&notification.register_id);
    debug!("Model: {}", builder.model.as_ref().unwrap());

    // Update request vehicle data
    let vehicle_data = builder.build();
    part_request.set_vehicle_data(vehicle_data);

    Ok(())
}

pub fn process_new_request(notification: &MessageLog) -> Result<PartRequest, String> {
    let mut builder: VehicleDataBuilder<WhatsappSource> =
        VehicleDataBuilder::<WhatsappSource>::default();
    // Request origin
    let mut origin = "WHATSAPP";

    // Create Part request
    let mut part_request = create_part_request(origin, &notification.register_id).unwrap();

    // Set vehicle information
    let mut res = set_request_vehicle(&mut part_request, notification);

    if res.is_err() {
        error!(
            "{}",
            format!(
                "Error setting request vehicle information: {}",
                res.as_ref().unwrap_err()
            )
        );
        return Err(format!(
            "Error setting request vehicle information: {}",
            res.unwrap_err()
        ));
    }

    res = set_request_details(&mut part_request, notification);

    if res.is_err() {
        error!(
            "{}",
            format!(
                "Error setting request details: {}",
                res.as_ref().unwrap_err()
            )
        );
        return Err(format!(
            "Error setting request details: {}",
            res.unwrap_err()
        ));
    }

    res = set_requestor(&mut part_request, notification);

    if res.is_err() {
        error!(
            "{}",
            format!("Error setting requestor: {}", res.as_ref().unwrap_err())
        );
        return Err(format!("Error setting requestor: {}", res.unwrap_err()));
    }

    Ok(part_request)
}

// pub fn determine_origin<T: Source>(origin_system: &str) -> Box<VehicleDataBuilder<T>> {
//
//     // Transform id to u16
//     let id = origin_system.parse::<u16>().unwrap();
//
//     match id {
//         3 => {
//             Box::new(VehicleDataBuilder::<WhatsappSource>::default()
//                 .build())
//         },
//         1 => {
//             Box::new(VehicleDataBuilder::<WhatsappSource>::default()
//                 .build())
//         }
//         _ => {
//             panic!("Unexpected system id")
//         }
//     }
// }

#[cfg(test)]
mod vin_tests {

    // Passes if vehicles from 2010 and 2039 year is returned correctly
    #[test]
    fn decoded_year_current_rotation() {
        assert!(false);
    }

    // Passes if vehicles from 1980 and 2009 year is returned correctly
    #[test]
    fn decoded_year_old_rotation() {
        assert!(false);
    }
}

#[cfg(test)]
mod request_build {

    // Passes if an error is thrown when trying to create a request with an id that already exists
    #[test]
    fn fails_request_duplicated() {
        assert!(false);
    }

    // Passes if a requestor id phone number matches the log phone number
    #[test]
    fn requestor_matches_log_phone() {
        assert!(false);
    }

    // Passes if a the vehicle brand associated to the request matches the one from the whatsapp-workflow step
    #[test]
    fn brand_is_set() {
        assert!(false);
    }

    // Passes if a the vehicle model associated to the request matches the one from the whatsapp-workflow step
    #[test]
    fn model_is_set() {
        assert!(false);
    }

    // Passes if a the vehicle vin associated to the request matches the one from the whatsapp-workflow step
    #[test]
    fn vin_is_set() {
        assert!(false);
    }
}
