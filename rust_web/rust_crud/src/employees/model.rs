use crate::db;
use crate::error_handler::CustomError;
use crate::schema::employees;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Deserialize,Serialize, AsChangeset, Insertable)]
#[table_name = "employees"]
pub struct Employee {
    pub first_name: String,
    pub last_name: String,
    pub department: String,
    pub salary: i32,
    pub age: i32,
}

impl Employee {
    fn from(employee: Employee) -> Employee {
        Employee {
            first_name: employee.first_name,
            last_name: employee.last_name,
            department: employee.department,
            salary: employee.salary,
            age: employee.age
        }
    }
}


no_arg_sql_function!(last_insert_rowid, diesel::sql_types::Integer);

#[derive(Deserialize,Serialize,Queryable, Insertable)]
#[table_name = "employees"]
pub struct Employees {
    pub id: i32,
    pub first_name: String,
    pub last_name: String,
    pub department: String,
    pub salary: i32,
    pub age: i32,
}

impl Employees {
    pub fn find_all() -> Result<Vec<Self>, CustomError> {
        let conn = db::connection()?;
        let employees = employees::table.load::<Employees>(&conn)?;
        Ok(employees)
    }

    pub fn find(id: i32) -> Result<Self, CustomError> {
        let conn = db::connection()?;
        let employee = employees::table.filter(employees::id.eq(id)).first(&conn)?;
        Ok(employee)
    }

    pub fn create(employee: Employee) -> Result<Self, CustomError>
    {
        let conn = db::connection()?;
        let employee = Employee::from(employee);
        let emp = diesel::insert_into(employees::table)
            .values(employee)
            .execute(&conn)?;
        //Ok(emp)
        //let last = employees::table.find(1).last::<Employees>(&conn).expect("row just inserted!");
        let last_id: i32 = diesel::select(last_insert_rowid).first(&conn).expect("row just inserted, must exist");
        let last = employees::table.filter(employees::id.eq(last_id)).first(&conn)?;
        Ok(last)
    }
}
