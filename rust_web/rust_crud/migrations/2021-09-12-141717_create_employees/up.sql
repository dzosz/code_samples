-- Your SQL goes here
CREATE TABLE "employees"
(
    id Integer PRIMARY KEY NOT NULL,
    first_name VARCHAR NOT NULL,
    last_name VARCHAR NOT NULL,
    department VARCHAR NOT NULL,
    salary INT NOT NULL,
    age INT NOT NULL
);

insert into "employees" values (0, "Mr", "Admin", "Dept", 3000, 30)
