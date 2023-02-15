use crate::redis::classification::{
    append_label, get_label_childs, get_pending_classification_requests, remove_label, get_request_labels
};
use crate::redis::part_register::create_part_request;
use fizzy_commons::shared_structs::{MessageLog, ModifiedReference, StandardResponse};
use log::{debug, error};

use crate::helpers::{process_new_request, retrieve_label_tree, remove_label_tree};
use crate::structs::classification::Label;
use crate::structs::part_request::{
    PartRequest, RequestDetailsBuilder, RequestorBuilder, VehicleDataBuilder,
};
use crate::structs::WhatsappSource;

pub fn get_labels(label_id: String) -> Result<Vec<Label>, String> {
    // Don't verify if looking for parent label as label with id 0 doesn't exist
    if label_id != "0" {
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
        return Err(err);
    }

    Ok(res.unwrap())
}

pub fn remove_request_labels(
    request_id: String,
    label_id: String,
) -> Result<StandardResponse, StandardResponse> {
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

    // Get label to check if child labels are added and remove them
    let label = Label::get(&label_id);

    // Get all labels added to request
    let assigned_labels: Result<Vec<Label>, String> = get_request_labels(&request_id);

    // Delete child labels recursively

    // Remove label from request
    debug!("assigned labels: {:?}", assigned_labels.as_ref().unwrap());
    let res = remove_label_tree(&request_id, &label_id, &mut assigned_labels.unwrap());


    if res.is_err() {
        response.errors = Some(vec![res.unwrap_err()]);
        return Err(response);
    }

    Ok(response)
}

pub fn classification_completed(request_id: String) {
    //PartRequest
}

pub fn update_request_labels(
    request_id: String,
    label_id: String,
) -> Result<StandardResponse, StandardResponse> {
    let mut response = StandardResponse {
        references: vec![],
        errors: None,
    };
    use std::time::Instant;
    let mut now = Instant::now();

    // Verify is label with specified code exists
    let label_exists = Label::exists(&label_id);

    let mut elapsed = now.elapsed();
    debug!("Label::exists Elapsed: {:.2?}", elapsed);

    if !label_exists {
        let err = format!("Label '{}' doesnt exist", &label_id);
        response.errors = Some(vec![err]);
        return Err(response);
    }

    now = Instant::now();

    // Verify if part request exists
    let request_exists = PartRequest::exists(&request_id);

    elapsed = now.elapsed();
    debug!("PartRequest::exists Elapsed: {:.2?}", elapsed);

    if !request_exists {
        let err = format!("Part request '{}' doesnt exist", &request_id);
        response.errors = Some(vec![err]);
        return Err(response);
    }

    // Get label to check parent
    let label = Label::get(&label_id);
    debug!("label: {label:?}");
    now = Instant::now();
    let mut label_list: Vec<Label> = vec![label.clone()];
    if label.parent != "0" {
        let list = retrieve_label_tree(&label.parent, &mut label_list);
        debug!("Label list: {:?}", label_list)
    }

    elapsed = now.elapsed();
    debug!("Label tree Elapsed: {:.2?}", elapsed);

    now = Instant::now();
    // Append label to request
    for iter_label in label_list{
        let res = append_label(&request_id, &iter_label.id);

        if res.is_err() {
            response.errors = Some(vec![res.unwrap_err()]);
            return Err(response);
        }
    }

    elapsed = now.elapsed();
    debug!("Append label Elapsed: {:.2?}", elapsed);



    Ok(response)
}

pub fn get_pending_requests() -> Result<Vec<PartRequest>, String> {
    let res = get_pending_classification_requests();

    if res.is_err() {
        return Err(format!("{}", res.unwrap_err()));
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
