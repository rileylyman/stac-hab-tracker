#![feature(proc_macro_hygiene, decl_macro, type_ascription)]

#[macro_use] extern crate rocket;
extern crate rusqlite;
extern crate chrono;

#[cfg(test)] mod tests;
mod secrets;

use std::sync::Mutex;

type DbConn = Mutex<rusqlite::Connection>;

#[derive(FromForm)]
struct RockData {
    packed_string: String,
    auth: String
}

#[post("/log", data = "<data>")]
fn log(conn: rocket::State<DbConn>, data: rocket::request::Form<RockData>) -> Result<(), String> {
    
    if data.auth != secrets::AUTH_STRING { return Err("Authentication does not match.".into()); }

    let components: Vec<&str> = data.packed_string
        .split(':')
        .collect();

    match conn.lock()
        .unwrap()
        .execute(
        "insert into updates 
            (time_logged, hour, minute, fixquality, speed, angle, lon, lat, altitude, temp)
         values
            ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)",

        &[  &fnow(),
            components[0], components[1], components[2],
            components[3], components[4], components[5],
            components[6], components[7], components[8], 
        ]) 
    {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("{}", e))
    }
}

#[get("/")]
fn get(conn: rocket::State<DbConn>) -> String {
  
    let lock = conn.lock().expect("acquire lock");
    let mut statement = lock.prepare("select * from updates").expect("select rows from db");
    let mut rows = statement.query(&[]: &[i32;0]).expect("execute db query");
    
    let mut ret = String::new();

    while let Some(maybe_row) = rows.next() 
    {
        let row = maybe_row.unwrap();
        let first: String = row.get(3);
        ret.push_str(&first);
    }

    ret
}

fn main() -> () {

    let conn = rusqlite::Connection::open_in_memory().expect("Failed to create sqlite");
    conn.execute("create table updates (
                  id, time_logged, hour, minute, fixquality, speed, angle, lon, lat, altitude, temp )", 
              &[]: &[i32;0]
            )
    .expect("create table");

    rocket::ignite()
        .manage(Mutex::new(conn))
        .mount("/", routes![get, log])
        .launch();
}

fn fnow() -> String {
    let now = std::time::SystemTime::now();
    let datetime: chrono::DateTime<chrono::offset::Utc> = now.into();
    format!("{}", datetime.format("%d/%m/%Y %T"))
}