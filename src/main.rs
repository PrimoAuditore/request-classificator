use actix_web::{App, HttpResponse, HttpServer, Responder, web, post};
use actix_web::middleware::Logger;
use fizzy_commons::shared_structs::MessageLog;

mod structs;
mod handlers;
mod redis;
mod helpers;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    HttpServer::new(|| {
        App::new()
            .wrap(Logger::new("%U"))
            .service(incoming_messages)
    })
        .bind(("0.0.0.0", 8080))?
        .run()
        .await
}

#[post("/incoming")]
async fn incoming_messages(log: web::Json<MessageLog>) -> impl Responder {
    let response = handlers::new_request_received(log.0);

    // match response {
    //     Ok(response) => HttpResponse::Ok().body(serde_json::to_string(&response).unwrap()),
    //     Err(response) => {
    //         HttpResponse::InternalServerError().body(serde_json::to_string(&response).unwrap())
    //     }
    // }


    "OK"
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
