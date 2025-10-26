use actix_web::{HttpRequest, HttpResponse, Responder};

pub async fn health_checker(_request: HttpRequest) -> impl Responder {
    HttpResponse::Ok().finish()
}
