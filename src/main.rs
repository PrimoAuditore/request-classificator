use crate::request_structs::LabelUpdate;
use crate::structs::classification::Label;
use actix_web::middleware::Logger;
use actix_web::{delete, get, post, put, web, App, HttpResponse, HttpServer, Responder};
use fizzy_commons::shared_structs::MessageLog;
use log::debug;

mod handlers;
mod helpers;
mod redis;
mod request_structs;
mod structs;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    HttpServer::new(|| {
        App::new()
            .wrap(Logger::new("%U"))
            .service(incoming_messages)
            .service(pending_requests)
            .service(append_label)
            .service(remove_label)
            .service(get_child_labels)
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}

#[post("/incoming")]
async fn incoming_messages(log: web::Json<MessageLog>) -> impl Responder {
    let response = handlers::new_request_received(log.0);

    match response {
        Ok(response) => HttpResponse::Ok().body(serde_json::to_string(&response).unwrap()),
        Err(response) => {
            HttpResponse::InternalServerError().body(serde_json::to_string(&response).unwrap())
        }
    }
}

#[get("/request")]
async fn pending_requests() -> impl Responder {
    let response = handlers::get_pending_requests();

    match response {
        Ok(response) => HttpResponse::Ok().body(serde_json::to_string(&response).unwrap()),
        Err(response) => HttpResponse::InternalServerError().body("error"),
    }
}

#[get("/label/{label_id}")]
async fn get_child_labels(path: web::Path<String>) -> impl Responder {
    let label_id = String::from(path.into_inner());

    let response = handlers::get_labels(label_id);

    match response {
        Ok(response) => HttpResponse::Ok().body(serde_json::to_string(&response).unwrap()),
        Err(err) => HttpResponse::InternalServerError().body(serde_json::to_string(&err).unwrap()),
    }
}

// Path: (Request Id, Label Code)
#[put("/request/{request_id}/labels")]
async fn append_label(path: web::Path<String>, label: web::Query<LabelUpdate>) -> impl Responder {
    debug!("{path:?} -> {label:?}");
    let request_id = String::from(&path.into_inner());
    let label_id = String::from(&label.label_id);
    let response = handlers::update_request_labels(request_id, label_id);

    match response {
        Ok(ok) => HttpResponse::Created().body(serde_json::to_string(&ok).unwrap()),
        Err(err) => HttpResponse::InternalServerError().body(serde_json::to_string(&err).unwrap()),
    }
}

//#[put("/request/{request_id}/done")]
//async fn classification_completed(path: web::Path<String>) -> impl Responder {
//    let request_id = String::from(&path.into_inner());
//    let response = handlers::classification_completed(request_id);

//    match response {
//        Ok(ok) => HttpResponse::Created().body(serde_json::to_string(&ok).unwrap()),
//        Err(err) => HttpResponse::InternalServerError().body(serde_json::to_string(&err).unwrap()),
//    }
//}

#[delete("/request/{request_id}/labels")]
async fn remove_label(path: web::Path<String>, label: web::Query<LabelUpdate>) -> impl Responder {
    debug!("{path:?} -> {label:?}");
    let request_id = String::from(&path.into_inner());
    let label_id = String::from(&label.label_id);
    let response = handlers::remove_request_labels(request_id, label_id);

    match response {
        Ok(ok) => HttpResponse::Created().body(serde_json::to_string(&ok).unwrap()),
        Err(err) => HttpResponse::InternalServerError().body(serde_json::to_string(&err).unwrap()),
    }
}

// #[post("/outgoing")]
// async fn outgoing_message(log: web::Json<MessageLog>) -> impl Responder {
//     let response = handlers::outgoing_messages(log.0);
//
//     match response {
//         Ok(response) => HttpResponse::Ok().body(serde_json::to_string(&response).unwrap()),
//         Err(response) => {
//             HttpResponse::InternalServerError().body(serde_json::to_string(&response).unwrap())
//         }
//     }
// }
