#[macro_use]
extern crate rocket;
use rocket::http::Status;
use rocket::response::{content::RawHtml, Responder};
use rocket::Request;
use rocket_db_pools::sqlx::{self, Column, Row};
use rocket_db_pools::{Connection, Database};
use sqlx::sqlite::SqliteRow;

#[derive(Database)]
#[database("music_log")]
struct Logs(sqlx::SqlitePool);

enum Error {
    QueryExec,
    EmptyRows,
}

impl<'r, 'o: 'r> Responder<'r, 'o> for Error {
    fn respond_to(
        self,
        req: &'r Request<'_>,
    ) -> rocket::response::Result<'o> {
        Status::InternalServerError.respond_to(req)
    }
}

fn parse_heading(filename: &str) -> String {
    if let Some(name) = filename.strip_suffix(".mp3") {
        name.replace('-', " ")
    } else {
        filename.replace('-', " ")
    }
}

fn to_html(
    rows: &[SqliteRow],
    heading: &str,
    titles: &[&str],
) -> Result<RawHtml<String>, Error> {
    if rows.is_empty() {
        Err(Error::EmptyRows)
    } else {
        let mut table = String::from("<table>");
        for title in titles {
            table.push_str(format!("<th>{title}</th>").as_str());
        }
        let columns = rows.first().unwrap().columns();
        for row in rows {
            table.push_str("<tr>");
            for column in columns {
                let name = column.name();
                if name.starts_with("count") || name == "id" {
                    let ele: u32 = row.get(name);
                    table.push_str(format!("<th>{ele}</th>").as_str());
                } else {
                    let ele: String = row.get(name);
                    let ele = parse_heading(&ele);
                    table.push_str(format!("<th>{ele}</th>").as_str());
                }
            }
            table.push_str("</tr>");
        }
        table.push_str("</table>");
        let css = include_str!("../assets/style.css");
        let html = format!(
            "<!;DOCTYPE html><html>{css}<title>music</title>
             <body><h1>{heading}</h1>{table}</body></html>"
        );
        Ok(RawHtml(html))
    }
}

#[get("/top/<n>")]
async fn top_n(
    mut db: Connection<Logs>,
    n: u32,
) -> Result<RawHtml<String>, Error> {
    // let mut result = String::new();
    let rows = sqlx::query(
        "select count(title), title
              from music_history group by title order by
              count(title) desc limit ?",
    )
    .bind(n)
    .fetch_all(&mut *db)
    .await
    .map_err(|_e| Error::QueryExec)?;

    to_html(
        &rows,
        format!("top {n} most listened tracks").as_str(),
        &["freq", "title"],
    )
}

#[get("/")]
async fn select_all(
    mut db: Connection<Logs>,
) -> Result<RawHtml<String>, Error> {
    let rows =
        sqlx::query("SELECT * FROM music_history ORDER BY id DESC;")
            .fetch_all(&mut *db)
            .await
            .map_err(|_e| Error::QueryExec)?;
    to_html(
        &rows,
        "recently played tracks",
        &["#", "title", "date", "time"],
    )
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .attach(Logs::init())
        .mount("/", routes![select_all, top_n])
}
