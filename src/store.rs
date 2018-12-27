use blobstore::{BlobStore, Store};
use crate::models::{Entity, Status};
use diesel::{prelude::*, result::Error as DieselError};
use reqwest::get;
use std::path::Path;

pub fn sync(conn: &PgConnection, path: &Path) {
    use crate::schema::entities::dsl::*;
    use crate::schema::statuses;

    let todo: Vec<(Entity, Status)> = entities
        .inner_join(statuses::table)
        .filter(statuses::deleted_at.is_null())
        .filter(blob_hash.is_null())
        .order_by(fetched_at)
        .load(conn)
        .expect("Cannot load entities from DB");

    if todo.is_empty() {
        println!("All entities in DB are already local, skip.");
        return;
    }

    println!("Downloading content for {} entities", todo.len());

    let bs = BlobStore::new(path.to_string_lossy().into());

    let mut successes = 0;
    for (entity, status) in &todo {
        println!(
            "\nDownloading entity #{} for {:?} {} (#{})\n==> {}",
            entity.id, status.source, status.source_id, status.id, entity.source_url
        );

        match get(&entity.source_url) {
            Err(err) => println!("Error downloading: {:?}", err),
            Ok(ref mut resp) => match bs.put(resp) {
                Err(err) => println!("Error storing: {:?}", err),
                Ok(hash) => match write_hash(conn, &entity, &hash) {
                    Err(err) => println!("Error recording: {:?}", err),
                    Ok(_) => {
                        successes += 1;
                        println!("Stored at hash {}.", hash);
                    }
                }
            }
        };
    }

    println!("Successfully downloaded {} (out of {}) entities", successes, todo.len());
}

fn write_hash(conn: &PgConnection, entity: &Entity, hash: &String) -> Result<(), DieselError> {
    use crate::schema::entities::dsl::*;

    diesel::update(entities.find(entity.id))
        .set(blob_hash.eq(hash))
        .execute(conn)?;

    Ok(())
}
