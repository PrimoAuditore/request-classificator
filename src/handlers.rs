use crate::redis::classification::{
    append_label, get_label_childs, get_pending_classification_requests, remove_label
};
use crate::redis::part_register::create_part_request;
use fizzy_commons::shared_structs::{MessageLog, ModifiedReference, StandardResponse};
use log::{debug, error};

use crate::helpers::process_new_request;
use crate::structs::part_request::{
    PartRequest, RequestDetailsBuilder, RequestorBuilder, VehicleDataBuilder,
};
use crate::structs::classification::{Label};
use crate::structs::WhatsappSource;

pub fn get_labels(label_id: String) -> Result<Vec<Label>, String>{

    // Don't verify if looking for parent label as label with id 0 doesn't exist
    if label_id != "0"{
        // Verify is label with specified code exists
        let label_exists = Label::exists(&label_id);
        if !label_exists {
            let err = format!("Label with id '{}' doesnt exist", &label_id);
            return Err(err);
        }
    }

    // Get label which parent corresponds to label_id
    let res = get_label_childs(&label_id);
    if res.is_err() {
        let err = format!("Error obtaining labels: {}", res.unwrap_err());
        error!("{}", err);
        return  Err(err)
    }

    Ok(res.unwrap())
}

pub fn remove_request_labels(request_id: String, label_id: String) -> Result<StandardResponse, StandardResponse> {
    let mut response = StandardResponse {
        references: vec![],
        errors: None,
    };


    // Verify if part request exists
    let request_exists = PartRequest::exists(&request_id);

    if !request_exists {
        let err = format!("Part request '{}' doesnt exist", &request_id);
        response.errors = Some(vec![err]);
        return Err(response);
    }


    // Append label to request
    let res = remove_label(&request_id, &label_id);

    if res.is_err() {
        response.errors = Some(vec![res.unwrap_err()]);
        return Err(response);
    }

    Ok(response)
}

pub fn update_request_labels(request_id: String, label_id: String) -> Result<StandardResponse, StandardResponse> {
    let mut response = StandardResponse {
        references: vec![],
        errors: None,
    };

    // Verify is label with specified code exists
    let label_exists = Label::exists(&label_id);

    if !label_exists {
        let err = format!("Label '{}' doesnt exist", &label_id);
        response.errors = Some(vec![err]);
        return Err(response);
    }

    // Verify if part request exists
    let request_exists = PartRequest::exists(&request_id);

    if !request_exists {
        let err = format!("Part request '{}' doesnt exist", &request_id);
        response.errors = Some(vec![err]);
        return Err(response);
    }

    // Append label to request
    let res = append_label(&request_id, &label_id);

    if res.is_err() {
        response.errors = Some(vec![res.unwrap_err()]);
        return Err(response);
    }

    Ok(response)
}

pub fn get_pending_requests() -> Result<Vec<PartRequest>, String> {
    let res = get_pending_classification_requests();

    if res.is_err() {
        return Err(format!("{}", res.unwrap_err()))
    }
    Ok(res.unwrap())
}

pub fn new_request_received(
    notification: MessageLog,
) -> Result<StandardResponse, StandardResponse> {
    let mut response = StandardResponse {
        references: vec![],
        errors: None,
    };

    match notification.origin_system.parse::<u16>().unwrap() {
        3 => {
            // User requested part
            let part_request = process_new_request(&notification);

            if part_request.is_err() {
                response.errors = Some(vec!["Error creating part request".to_string()]);
                return Err(response);
            }

            response.references.push(ModifiedReference {
                system: "REDIS".to_string(),
                reference: part_request.unwrap().id,
            });
            Ok(response)
        }
        _ => {
            error!("Unexpected system");
            Err(response)
        }
    }
}

// pub fn outgoing_messages() -> Result<StandardResponse, StandardResponse>{
//     let mut response: StandardResponse = StandardResponse::new();
//     let mut errors = vec![];
//  gt   let mut references = vec![];
//
//
// }

#[cfg(test)]
mod tests {}
