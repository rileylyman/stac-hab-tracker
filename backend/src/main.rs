#![feature(proc_macro_hygiene, decl_macro, type_ascription)]

#[macro_use] extern crate rocket;
#[macro_use] extern crate serde_derive;
extern crate rusqlite;
extern crate chrono;
#[macro_use] extern crate rocket_contrib;

#[cfg(test)] mod tests;
mod secrets;

use std::sync::Mutex;
use rocket_contrib::json::{ Json, JsonValue };

type DbConn = Mutex<rusqlite::Connection>;

#[derive(FromForm)]
struct RockPost {
    packed_string: String,
    auth: String
}

#[derive(Serialize, Deserialize)]
struct RockData {
    time_logged: String,
    hour: u32,
    minute: u32,
    fixquality: u32,
    speed: f64,
    angle: f64,
    lon: f64,
    lat: f64,
    altitude: f64,
    temp: f64,
}

impl std::fmt::Display for RockData {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
                f, "Received: {}\n 
                    Sent: {}:{}\n 
                    Fix: {}\n 
                    Speed: {}\n 
                    Angle: {}\n
                    Longitude: {}\n 
                    Latitude: {}\n 
                    Altitude: {}\n 
                    Temperature: {}",
                self.time_logged, self.hour, self.minute, self.fixquality, self.speed, 
                self.angle, self.lon, self.lat, self.altitude, self.temp
        )
    }
}

#[post("/log", data = "<data>")]
fn log(conn: rocket::State<DbConn>, data: rocket::request::Form<RockPost>) -> JsonValue {
    
    //TODO: check packed string values

    if data.auth != secrets::AUTH_STRING { 
        return json!({
            "status":"error", 
            "reason":"Authentication did not match."
        }); 
    }

    let components: Vec<&str> = data.packed_string
        .split(':')
        .collect();

    if components.len() != 9 {
        return json!({
            "status": "error",
            "reason": "Invalid packed-data format."
        })
    }

    match conn.lock()
        .unwrap()
        .execute(
        "insert into updates 
            (time_logged, hour, minute, fixquality, speed, angle, lon, lat, altitude, temp)
         values
            ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)",

        &[  &fnow(),
            components[0], components[1], components[2],
            components[3], components[4], components[5],
            components[6], components[7], components[8] 
        ]) 
    {
        Ok(_) => json!({
            "status":"ok"
        }),
        Err(e) => json!({
            "status":"error",
            "reason": format!("{}", e)
        })
    }
}

#[get("/")]
fn get(conn: rocket::State<DbConn>) -> Json<Option<Vec<RockData>>> {
  
    let lock = conn.lock().expect("acquire lock");
    let mut statement = lock.prepare("select * from updates").expect("select rows from db");
    let mut rows = statement.query(&[]: &[i32;0]).expect("execute db query");
    
    let mut all_data: Vec<RockData> = Vec::new();

    while let Some(maybe_row) = rows.next() 
    {
        let row = maybe_row.expect("get next row");
        all_data.push(RockData {
            time_logged: row.get(0),
            hour: row.get(1),
            minute: row.get(2),
            fixquality: row.get(3),
            speed: row.get(4),
            angle: row.get(5),
            lon: row.get(6),
            lat: row.get(7),
            altitude: row.get(8),
            temp: row.get(9),
        });  
    }

    if all_data.len() > 0 {
        Json(Some(all_data))
    } else {
        Json(None)
    }
}

#[catch(404)]
fn not_found() -> JsonValue {
    json!({
        "status": "error",
        "reason": "Resource was not found."
    })
}

fn main() -> () {

    let conn = rusqlite::Connection
        ::open(std::path::Path::new("./target/test_db.db"))
        .expect("Failed to create sqlite");

    match conn.execute("create table updates (
                            time_logged TEXT,
                            hour INTEGER,
                            minute INTEGER,
                            fixquality INTEGER, 
                            speed REAL, 
                            angle REAL, 
                            lon REAL, 
                            lat REAL, 
                            altitude REAL, 
                            temp REAL
                        )", &[]: &[i32;0]) { _ => {} }

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