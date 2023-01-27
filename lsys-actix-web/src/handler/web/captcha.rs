use std::str::FromStr;

use actix_web::http::header::{CacheControl, CacheDirective};
use actix_web::web::Data;
use actix_web::{get, HttpResponse};
use lsys_web::dao::{CaptchaKey, WebDao};

#[get("/captcha/{type}/{tag}")]
pub(crate) async fn captcha(
    path: actix_web::web::Path<(String, String)>,
    web_dao: Data<WebDao>,
) -> HttpResponse {
    match CaptchaKey::from_str(path.0.to_string().as_str()) {
        Ok(captcha_key) => {
            let valid_code = web_dao.captcha.valid_code(&captcha_key);
            let mut valid_code_data = web_dao.captcha.valid_code_builder();
            match valid_code
                .set_code(&path.1.to_string(), &mut valid_code_data)
                .await
            {
                Ok(_) => HttpResponse::Ok()
                    .content_type(valid_code_data.image_header)
                    .append_header(CacheControl(vec![
                        CacheDirective::Private,
                        CacheDirective::MaxAge(valid_code_data.save_time as u32),
                    ]))
                    .body(valid_code_data.image_data),
                Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
            }
        }
        Err(_) => HttpResponse::NotFound().body("not find"),
    }
}
