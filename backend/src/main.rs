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

const MAIN_DB: &'static str = "./db/main.db";
const BACKUP_DB: &'static str = "./db/backup.db";

type DbConn = Mutex<rusqlite::Connection>;

enum TimeType {
    All, Day, Month, Year
}

#[derive(FromForm)]
struct RockPost {
    packed_string: String,
    auth: String
}

#[derive(Serialize, Deserialize)]
struct RockData {
        trip: u32,
        time_logged: String,
        day_logged: u32,
        month_logged: u32,
        year_logged: u32,
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

#[post("/backup?<auth_string>")]
fn do_backup(conn: rocket::State<DbConn>, auth_string: String) -> JsonValue {
    
    if auth_string != secrets::AUTH_STRING { 
        return json!({
            "status":"error", 
            "reason":"Invalid authentication."
        }); 
    }
    
    //Backup the database now...
    let main_connection: &rusqlite::Connection = &*conn.lock().unwrap();
    let mut backup_connection = rusqlite::Connection::open(std::path::Path::new(BACKUP_DB)).unwrap();
    let backup = rusqlite::backup::Backup::new(main_connection, &mut backup_connection).unwrap();
    backup.run_to_completion(5, std::time::Duration::from_millis(250), None).unwrap();
    json!({
        "status": "ok"
    })
}

#[post("/log", data = "<data>")]
fn log(conn: rocket::State<DbConn>, data: rocket::request::Form<RockPost>) -> JsonValue {
    
    //TODO: Transform into degree lon/lat

    if data.auth != secrets::AUTH_STRING { 
        return json!({
            "status":"error", 
            "reason":"Invalid authentication."
        }); 
    }

    let components: Vec<&str> = data.packed_string
        .split(':')
        .collect();

    if components.len() != 10 {
        return json!({
            "status": "error",
            "reason": "Packed string contains an unexpected number of values."
        })
    }

    let mut trip: u32 = 0;
    let mut hour: u32 = 0;
    let mut minute: u32 = 0;
    let mut fixquality: u32 = 0;
    let mut speed: f64 = 0.0;
    let mut angle: f64 = 0.0;
    let mut lon: f64 = 0.0;
    let mut lat: f64 = 0.0;
    let mut altitude: f64 = 0.0;
    let mut temp: f64 = 0.0;

    if let Err(_) = try_parse_packed(&components, &mut trip,       &mut hour,     &mut minute,   
                                                  &mut fixquality, &mut speed,    &mut angle,    
                                                  &mut lon,        &mut lat,      &mut altitude, 
                                                  &mut temp) 
    { 
        return json!({
            "status": "error",
            "reason": "Could not parse input string."
        });
    }

    match conn.lock()
        .unwrap()
        .execute(
        "insert into updates 
            (trip, time_logged, day_logged, month_logged, year_logged,
             hour, minute, fixquality, speed, angle, lon, lat, altitude, temp)
         values
            ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)",

        &[  &trip.to_string(), &fnow(TimeType::All), &fnow(TimeType::Day), 
            &fnow(TimeType::Month), &fnow(TimeType::Year), 
            &hour.to_string(),  &minute.to_string(),   &fixquality.to_string(),
            &speed.to_string(), &angle.to_string(),    &lon.to_string(),
            &lat.to_string(),   &altitude.to_string(), &temp.to_string() 
        ]
    )   {
        Ok(_) => json!({
            "status": "ok",
            "message": "have a nice day"
        }),
        Err(e) => json!({
            "status":"error",
            "reason": format!("{}", e)
        })
    }
}

#[get("/<trip>")]
fn get(conn: rocket::State<DbConn>, trip: u32) -> Json<Option<Vec<RockData>>> {
  
    let lock = conn.lock().expect("acquire lock");
    let mut statement = lock.prepare("select * from updates where trip = ?").expect("select rows from db");
    let mut rows = statement.query(&[trip]: &[u32;1]).expect("execute db query");
    
    let mut all_data: Vec<RockData> = Vec::new();

    while let Some(maybe_row) = rows.next() 
    {
        let row = maybe_row.expect("get next row");
        all_data.push(
            RockData {
                trip:         row.get(0),
                time_logged:  row.get(1),
                day_logged:   row.get(2),
                month_logged: row.get(3),
                year_logged:  row.get(4),
                hour:         row.get(5),
                minute:       row.get(6),
                fixquality:   row.get(7),
                speed:        row.get(8),
                angle:        row.get(9),
                lon:          row.get(10),
                lat:          row.get(11),
                altitude:     row.get(12),
                temp:         row.get(13),
            }
        );  
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
        ::open(std::path::Path::new(MAIN_DB))
        .expect("create main connection");

    init_db(&conn);

    let backup_conn = rusqlite::Connection
        ::open(std::path::Path::new(BACKUP_DB))
        .expect("create backup connection");

    init_db(&backup_conn);

    rocket::ignite()
        .manage(Mutex::new(conn))
        .mount("/", routes![get, log, do_backup])
        .launch();
}

fn init_db(conn: &rusqlite::Connection) {
    if let Err(_) = conn.execute(
        "create table updates (
            trip INTEGER,
            time_logged TEXT,
            day_logged INTEGER,
            month_logged INTEGER,
            year_logged INTEGER,
            hour INTEGER,
            minute INTEGER,
            fixquality INTEGER, 
            speed REAL, 
            angle REAL, 
            lon REAL, 
            lat REAL, 
            altitude REAL, 
            temp REAL
        )",
         &[]: &[i32;0]) 
    { 
        println!("Table updates already created."); 
    }
}

fn fnow(t: TimeType) -> String {
    let now = std::time::SystemTime::now();
    let datetime: chrono::DateTime<chrono::offset::Utc> = now.into();
    let mut fstr = String::new();
    match t {
        TimeType::All => fstr.push_str("%d/%m/%Y %T"),
        TimeType::Day => fstr.push_str("%d"),
        TimeType::Month => fstr.push_str("%m"),
        TimeType::Year => fstr.push_str("%Y")
    }
    format!("{}", datetime.format(&fstr))
}

fn try_parse_packed(components: &Vec<&str>, trip: &mut u32, hour: &mut u32, minute: &mut u32, fixquality: &mut u32,
            speed: &mut f64, angle: &mut f64, lon: &mut f64, lat: &mut f64, altitude: &mut f64, temp: &mut f64) -> Result<(), ()> 
{
    if let ( Ok(_trip), Ok(_hour),  Ok(_minute),   Ok(_fixquality),
             Ok(_speed), Ok(_angle),    Ok(_lon),
             Ok(_lat),   Ok(_altitude), Ok(_temp)) = 
           ( components[0].parse::<u32>(), components[1].parse::<u32>(),
             components[2].parse::<u32>(), components[3].parse::<u32>(),
             components[4].parse::<f64>(), components[5].parse::<f64>(),
             components[6].parse::<f64>(), components[7].parse::<f64>(), 
             components[8].parse::<f64>(), components[9].parse::<f64>(),) 
    {
        *hour  = _hour;  *minute   = _minute;   *fixquality = _fixquality;
        *speed = _speed; *angle    = _angle;    *lon        = _lon;
        *lat   = _lat;   *altitude = _altitude; *temp       = _temp;
        *trip  = _trip;
        Ok(())
    } 
    else {
        Err(())
    } 
}