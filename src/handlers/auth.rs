
struct AppState {
}

type State = web::Data<AppState>;

#[post("/register")]
pub async fn login_handler(state: State, form: ) -> impl Responder {

}

#[post("/login")]
pub async fn register_handler(state: State, form: ) -> impl Responder {

}

