use crate::errors::*;
use crate::auth2::AuthHeader;
use crate::db;
use crate::models::*;
use diesel::Connection;
use rocket::request::Form;
use rocket_contrib::json::Json;
use semver::Version;
use sn0int_common::api::*;
use sn0int_common::id;
use sn0int_common::metadata::Metadata;


#[get("/quickstart")]
pub fn quickstart(connection: db::Connection) -> ApiResult<Json<ApiResponse<Vec<Module>>>> {
    let modules = Module::quickstart(&connection)?;
    Ok(Json(ApiResponse::Success(modules)))
}

#[derive(Debug, FromForm)]
pub struct Search {
    q: String,
}

#[get("/search?<q..>")]
pub fn search(q: Form<Search>, connection: db::Connection) -> ApiResult<Json<ApiResponse<Vec<SearchResponse>>>> {
    info!("Searching: {:?}", q.q);

    let modules = Module::search(&q.q, &connection)?;
    let modules = modules.into_iter()
        .flat_map(|(module, downloads)| {
            Ok::<_, ()>(SearchResponse {
                author: module.author,
                name: module.name,
                description: module.description,
                latest: module.latest.ok_or(())?,
                featured: module.featured,
                downloads,
            })
        })
        .collect();

    Ok(Json(ApiResponse::Success(modules)))
}

#[get("/info/<author>/<name>", format="application/json")]
pub fn info(author: String, name: String, connection: db::Connection) -> ApiResult<Json<ApiResponse<ModuleInfoResponse>>> {
    info!("Querying {:?}/{:?}", author, name);
    let module = Module::find(&author, &name, &connection)?;

    Ok(Json(ApiResponse::Success(ModuleInfoResponse {
        author: module.author,
        name: module.name,
        description: module.description,
        latest: module.latest,
    })))
}

#[get("/dl/<author>/<name>/<version>", format="application/json")]
pub fn download(author: String, name: String, version: String, connection: db::Connection) -> ApiResult<Json<ApiResponse<DownloadResponse>>> {
    info!("Downloading {:?}/{:?} ({:?})", author, name, version);
    let module = Module::find(&author, &name, &connection)?;
    debug!("Module: {:?}", module);
    let release = Release::find(module.id, &version, &connection)?;
    debug!("Release: {:?}", release);

    release.bump_downloads(&connection)?;

    Ok(Json(ApiResponse::Success(DownloadResponse {
        author,
        name,
        version,
        code: release.code,
    })))
}

#[post("/publish/<name>", format="application/json", data="<upload>")]
pub fn publish(name: String, upload: Json<PublishRequest>, session: AuthHeader, connection: db::Connection) -> ApiResult<Json<ApiResponse<PublishResponse>>> {
    let user = session.verify(&connection)?;

    id::valid_name(&user)
        .context("Username is invalid")
        .map_err(Error::from)?;
    id::valid_name(&name)
        .context("Module name is invalid")
        .map_err(Error::from)?;

    let metadata = upload.code.parse::<Metadata>()?;

    let version = metadata.version.clone();
    Version::parse(&version)
        .context("Version is invalid")
        .map_err(Error::from)?;

    connection.transaction::<_, Error, _>(|| {
        let module = Module::update_or_create(&user, &name, &metadata.description, &connection)?;

        match Release::try_find(module.id, &version, &connection)? {
            Some(release) => {
                // if the code is identical, pretend we published the version
                if release.code != upload.code {
                    bail!("Version number already in use")
                }
            },
            None => module.add_version(&version, &upload.code, &connection)?,
        }

        Ok(())
    })?;

    Ok(Json(ApiResponse::Success(PublishResponse {
        author: user,
        name,
        version,
    })))
}

#[get("/whoami")]
pub fn whoami(session: AuthHeader, connection: db::Connection) -> ApiResult<Json<ApiResponse<WhoamiResponse>>> {
    let user = session.verify(&connection)?;
    Ok(Json(ApiResponse::Success(WhoamiResponse {
        user,
    })))
}
