use actix_web::{web::{self, Data}, HttpResponse, http::StatusCode};
use serde_json::json;
use crate::{database::Database, USER_LIST, libs::*};
use super::structs::*;

// Ambil genre
pub async fn get_genre(path: web::Path<UserID>, query: web::Query<OptionalGenre>, db: Data::<Database>) -> HttpResponse {  

    // Cek genre ada diisi
    match &query.genre {
        // Kalau ada, cek kalo user sama genre ada
        Some(x) => {
            match check_userid_genre(&path.user_id, &x.to_lowercase(), &db).await{
                Ok(_) => (),
                Err((s, e)) => return HttpResponse::build(s).json(json!({"error": e.to_string()}))
            };
        },

        // Cek kalo elastic hidup
        None => if !check_server(&db).await { return HttpResponse::ServiceUnavailable().json(json!({"error": Errors::ServerDown.to_string()})) }
    }


    // Minta data genre ke elastic
    let response = db.get_indices(
        Some(
            format!("{}.{}", &path.user_id.to_lowercase(), 
            &query.genre.as_ref().unwrap_or(&"*".to_owned()).to_lowercase())
        )).await.unwrap();

    if !response.status_code().is_success(){
        return match response.status_code() {
            StatusCode::NOT_FOUND => HttpResponse::NotFound().finish(),
            _ => HttpResponse::build(response.status_code()).json(json!({"error": Errors::Unknown.to_string()}))
        }
    }
    HttpResponse::build(response.status_code()).json(response.json::<Vec<IndexResponse>>().await.unwrap())
}

// Buat genre baru
pub async fn create_genre(path: web::Path<UserID>, data: web::Json<Genre>, db: Data::<Database>) -> HttpResponse {  

    // Cek kalo elastic hidup
    if !check_server(&db).await { return HttpResponse::ServiceUnavailable().json(json!({"error": Errors::ServerDown.to_string()})); };
    
    // Konversi genre supaya stringnya valid pas lempar ke elastic
    let genre: String = data.genre.to_lowercase().chars().map(|c| if !c.is_ascii() || c.is_whitespace() {'_'} else {c}).collect();

    // Cek kalo genre udah ada
    match genre_exists(&path.user_id, &genre, &db).await {
        
        // Kalo ada, gagalin
        Ok(_) => HttpResponse::Conflict().json(json!({"error": Errors::GenreExists(genre).to_string()})),

        // Kalo eror, berarti antara belum ada atau usernya engga ada
        Err((s, e, mut l)) => match e {

            // User engga ada, langsung gagalin karena gabisa dibuat
            Errors::UserNotFound(_) => HttpResponse::build(s).json(json!({"error": e.to_string()})),

            // Genre engga ada, berarti bisa buat
            Errors::GenreNotFound(_) => {
                l.insert(genre.clone());
                let body = json!({"genres": l});
                let _ = db.update_single_document(USER_LIST, &path.user_id, &body).await;
                create_new_genre(Some(path.user_id.to_string()), &genre, &db).await;
                HttpResponse::Created().finish()
            },

            // Gatau eror apa
            _ => HttpResponse::build(s).json(json!({"error": Errors::Unknown.to_string()}))
        }
    }
}

// Hapus genre
pub async fn delete_genre(path: web::Path<UserGenre>, db: Data::<Database>) -> HttpResponse {  

    // Cek kalo user sama genre ada
    let genre = path.genre.to_lowercase();
    match check_userid_genre(&path.user_id, &genre, &db).await{
        Ok(_) => (),
        Err((s, e)) => return HttpResponse::build(s).json(json!({"error": e.to_string()}))
    };

    // Langsung kirim minta hapus
    let code = db.delete_single_index(format!("{}.{}", &path.user_id.to_lowercase(), &genre)).await.unwrap().status_code();

    if !code.is_success(){
        return match code {
            StatusCode::NOT_FOUND => HttpResponse::NotFound().finish(),
            _ => HttpResponse::build(code).json(json!({"error": Errors::Unknown.to_string()}))
        }
    }

    // Kalo berhasil dihapus, hapus juga yang ada di data usernya
    match genre_exists(&path.user_id, &genre, &db).await {
        Ok(mut l) => {
            l.remove(&genre);
            let _ = db.update_single_document(USER_LIST, &path.user_id, &json!({"genres": l})).await;
            HttpResponse::build(code).finish()
        },
        Err((s, e, _)) => HttpResponse::build(s).json(json!({"error": e.to_string()})),
    }
}