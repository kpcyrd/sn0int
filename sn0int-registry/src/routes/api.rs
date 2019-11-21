use sn0int_registry::errors::*;
use crate::auth2::AuthHeader;
use sn0int_registry::db;
use sn0int_registry::models::*;
use diesel::Connection;
use rocket::request::Form;
use rocket_contrib::json::Json;
use semver::Version;
use sn0int_common::api::*;
use sn0int_common::id;
use sn0int_common::metadata::Metadata;
use std::time::UNIX_EPOCH;


#[get("/quickstart")]
pub fn quickstart(connection: db::Connection) -> ApiResult<ApiResponse<Vec<Module>>> {
    let modules = Module::quickstart(&connection)?;
    Ok(ApiResponse::Success(modules))
}

#[get("/latest")]
pub fn latest(connection: db::Connection) -> ApiResult<ApiResponse<LatestResponse>> {
    let time = Release::latest(&connection)?
        .map(|x| {
            x.published.duration_since(UNIX_EPOCH)
                .expect("Time went backwards")
                .as_secs()
        });
    let latest = LatestResponse {
        time,
    };
    Ok(ApiResponse::Success(latest))
}

#[derive(Debug, FromForm)]
pub struct Search {
    q: String,
}

#[get("/search?<q..>")]
pub fn search(q: Form<Search>, connection: db::Connection) -> ApiResult<ApiResponse<Vec<SearchResponse>>> {
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

    Ok(ApiResponse::Success(modules))
}

#[get("/info/<author>/<name>", format="application/json")]
pub fn info(author: String, name: String, connection: db::Connection) -> ApiResult<ApiResponse<ModuleInfoResponse>> {
    info!("Querying {:?}/{:?}", author, name);
    let module = Module::find(&author, &name, &connection)
        .not_found()
        .public_context("Module does not exist")?;

    let redirect = if let Some(redirect) = module.redirect {
        Some(redirect.parse()?)
    } else {
        None
    };

    Ok(ApiResponse::Success(ModuleInfoResponse {
        author: module.author,
        name: module.name,
        description: module.description,
        latest: module.latest,
        redirect,
    }))
}

#[get("/dl/<author>/<name>/<version>", format="application/json")]
pub fn download(author: String, name: String, version: String, connection: db::Connection) -> ApiResult<ApiResponse<DownloadResponse>> {
    info!("Downloading {:?}/{:?} ({:?})", author, name, version);
    let module = Module::find(&author, &name, &connection)
        .not_found()
        .public_context("Module does not exist")?;
    debug!("Module: {:?}", module);
    let release = Release::find(module.id, &version, &connection)
        .not_found()
        .public_context("Release does not exist")?;
    debug!("Release: {:?}", release);

    release.bump_downloads(&connection)?;

    Ok(ApiResponse::Success(DownloadResponse {
        author,
        name,
        version,
        code: release.code,
    }))
}

#[post("/publish/<name>", format="application/json", data="<upload>")]
pub fn publish(name: String, upload: Json<PublishRequest>, session: AuthHeader, connection: db::Connection) -> ApiResult<ApiResponse<PublishResponse>> {
    let user = session.verify(&connection)
        .bad_request()
        .public_context("Invalid auth token")?;

    id::valid_name(&user)
        .bad_request()
        .public_context("Username is invalid")?;
    id::valid_name(&name)
        .bad_request()
        .public_context("Module name is invalid")?;

    let metadata = upload.code.parse::<Metadata>()
        .bad_request()
        .public_context("Failed to parse module metadata")?;

    let version = metadata.version.clone();
    Version::parse(&version)
        .bad_request()
        .public_context("Version is invalid")?;

    connection.transaction::<_, WebError, _>(|| {
        let module = Module::update_or_create(&user, &name, &metadata, &connection)
            .private_context("Failed to write module metadata")?;

        match Release::try_find(module.id, &version, &connection)? {
            Some(release) => {
                // if the code is identical, pretend we published the version
                if release.code != upload.code {
                    bad_request!("Version number already in use")
                }
            },
            None => module.add_version(&version, &upload.code, &connection)
                .private_context("Failed to add release")?,
        }

        Ok(())
    })?;

    Ok(ApiResponse::Success(PublishResponse {
        author: user,
        name,
        version,
    }))
}

#[get("/whoami")]
pub fn whoami(session: AuthHeader, connection: db::Connection) -> ApiResult<ApiResponse<WhoamiResponse>> {
    let user = session.verify(&connection)
        .bad_request()
        .public_context("Invalid auth token")?;
    Ok(ApiResponse::Success(WhoamiResponse {
        user,
    }))
}
