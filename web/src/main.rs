#[macro_use] extern crate rocket;
use rocket_db_pools::{Database, Connection};
use rocket_db_pools::sqlx::{self, Row};
use rocket::Request;
use rocket::response::Responder;
use rocket::http::Status;

#[derive(Database)]
#[database("music_log")]
struct Logs(sqlx::SqlitePool);

enum Error {
    QueryExec
}

impl<'r, 'o: 'r> Responder<'r, 'o> for Error {
    fn respond_to(self, req: &'r Request<'_>) -> rocket::response::Result<'o> {
        Status::InternalServerError.respond_to(req)
    }
}

#[get("/top/<n>")]
async fn top_n(mut db: Connection<Logs>, n: u32) -> Result<String, Error> {
    let mut result = String::new();
    let rows = sqlx::query("select count(title) ,title from music_history group by title order by count(title) desc limit ?")
        .bind(n)
        .fetch_all(&mut *db).await
        .map_err(|_e| Error::QueryExec)?;
    for row in rows {
        let date: u32 = row.get("count(title)");
        let title: String = row.get("title");
        result.push_str(&title);
        result.push(' ');
        result.push_str(&date.to_string());
        result.push('\n');
    }
    Ok(result)
}


#[get("/")]
async fn select_all(mut db: Connection<Logs>) -> Result<String, Error> { 
    let mut result = String::new();
    let rows = sqlx::query("SELECT * FROM music_history ORDER BY id DESC;")
        .fetch_all(&mut *db).await
        .map_err(|_e| Error::QueryExec)?;
    
    for row in rows {
        let title: String = row.get("title");
        let date: String = row.get("date");
        result.push_str(&title);
        result.push(' ');
        result.push_str(&date);
        result.push('\n');
    }
    Ok(result)
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .attach(Logs::init())
        .mount("/", routes![select_all, top_n])
}
