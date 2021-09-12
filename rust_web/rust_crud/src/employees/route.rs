use crate::employees::{Employee, Employees};
use crate::error_handler::CustomError;
use actix_web::{delete, get, post, put, web, HttpResponse };
use serde_json::json;

#[get("/employeess")]
async fn find_all() -> Result<HttpResponse, CustomError> {
    let employees = Employees::find_all()?;
    Ok(HttpResponse::Ok().json(employees))
}

#[get("/")]
async fn main() -> Result<HttpResponse, CustomError> {
    let employees = Employees::find_all()?;
    Ok(HttpResponse::Ok().json(employees))
}

#[post("/employees")]
async fn create(employee: web::Json<Employee>) -> Result<HttpResponse, CustomError> {
    let employee = Employees::create(employee.into_inner())?;
    Ok(HttpResponse::Ok().json(employee))
}

#[get("/employees/{id}")]
async fn find(id: web::Path<i32>) -> Result<HttpResponse, CustomError> {
    let employee = Employees::find(id.into_inner())?;
    Ok(HttpResponse::Ok().json(employee))
}

pub fn init_routes(config: &mut web::ServiceConfig) {
    config.service(find_all);
    config.service(find);
    config.service(main);
    config.service(create);
}
