// TODO: Document and potentially simply this(if I can)
// Instesad of 3 calls it should be possible to make it 2 type calls on response
use poem_openapi::Object;
use serde::Serialize;

#[derive(Object, Serialize, Debug)]
pub struct StatsResponse {
    name: String, // NOTE: can I also make it specific "github_user_fetcher here?"
    running: bool
}

pub fn get_stats() -> StatsResponse {
    StatsResponse {
        name: "github_user_fetcher".to_string(),
        running: true,
    }
}

// NOTE: Multiple variant Response option is:
// #[derive(ApiResponse, Debug)]
// pub enum GetStatsResponse {
//     #[oai(status = 200)]
//     Ok(Json<StatsResponse>),
// }
//
// pub async fn get_stats() -> GetStatsResponse {
//     GetStatsResponse::Ok(Json(StatsResponse {
//         name: "github_user_fetcher".to_string(),
//         running: true
//     }))
// }

