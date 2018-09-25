use rocket_contrib::{Json, Value};


#[derive(Debug, Serialize, Deserialize)]
pub struct Dummy {
}

#[get("/dashboard")]
fn dashboard() -> Json<Value> {
    Json(json!({ "dashboard": Dummy {
    }}))
}

#[derive(Debug, FromForm)]
pub struct Search {
    q: String,
}

#[get("/search?<q>")]
fn search(q: Search) -> Json<Value> {
    println!("{:?}", q);
    Json(json!({ "dashboard": Dummy {
    }}))
}

#[get("/dl/<author>/<name>", format="application/json")]
fn download(author: String, name: String) -> Json<Value> {
    println!("{:?}/{:?}", author, name);
    Json(json!({ "dashboard": Dummy {
    }}))
}

#[post("/publish/<author>/<name>", format="application/json", data="<upload>")]
fn publish(author: String, name: String, upload: String) -> Json<Value> {
    println!("{:?}/{:?}: {:?}", author, name, upload);
    Json(json!({ "dashboard": Dummy {
    }}))
}

#[post("/login")]
fn login() -> Json<Value> {
    Json(json!({ "dashboard": Dummy {
    }}))
}
