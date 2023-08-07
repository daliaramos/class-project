use axum::extract::{Path,Query, State};
use axum::Json;
use crate::db::Store;
use crate::error::{AppError};
use crate::question::{Question, QuestionId, UpdateQuestion, CreateQuestion, GetQuestionById};
use crate::answer::{Answer, CreateAnswer};
use jsonwebtoken::Header;
use crate::get_timestamp_after_8_hours;
use crate::user::{UserSignup, Claims, User, KEYS};
use serde_json::{json, Value};
pub async fn root() -> String {
    "Hello World!".to_string()
}

//CRUD - create, read, update, delete
pub async fn get_questions(
    State(mut am_database): State<Store>,
) -> Result<Json<Vec<Question>>, AppError> {
    let all_questions = am_database.get_all_questions().await?;
    Ok(Json(all_questions))
}


pub async fn get_question_by_id(
    State(mut am_database): State<Store>,
    Path(query): Path<i32>,
) -> Result<Json<Question>, AppError> {
    let question = am_database.get_question_by_id(QuestionId(query)).await?;
    Ok(Json(question))
}

pub async fn create_question(
    State(mut am_database): State<Store>,
    Json(question): Json<CreateQuestion>
) -> Result<Json<()>, AppError> {
    let new_question = am_database.add_question(question.title, question.content, question.tags).await?;
    Ok(Json(new_question)) //ORM - object relational mapper
}

pub async fn update_question(
    State(mut am_database): State<Store>,
    Json(question): Json<UpdateQuestion>,
) -> Result<Json<Question>, AppError> {
    let updated_question = am_database.update_question(question).await?;
    Ok(Json(updated_question))
}


pub async fn delete_question(
    State(mut am_database): State<Store>,
    Query(query): Query<GetQuestionById>
) -> Result<(), AppError> {
    am_database.delete_question(query.question_id).await?;
    Ok(())
}
pub async fn create_answer(
    State(mut am_database): State<Store>,
    Json(answer): Json<CreateAnswer>,
) -> Result<Json<Answer>, AppError> {
    dbg!("GOT CREATE ANSWER:");
    dbg!(&answer);
    let new_answer = am_database.add_answer(answer.content, answer.question_id).await?;
    Ok(Json(new_answer))
}

pub async fn register(
    State(mut database) : State<Store>,
    Json(credentials): Json<UserSignup>
) -> Result<Json<Value>, AppError> {
    if credentials.email.is_empty() || credentials.password.is_empty() {
        return Err(AppError::MissingCredentials)
    }

    if credentials.password != credentials.confirm_password {
        return Err(AppError::MissingCredentials)
    }

    //check to see if there is already a user in the db w the given email address
    let existing_user = database.get_user(&credentials.email).await;

    if let Ok(_) = existing_user {
        return Err(AppError::UserAlreadyExists);
    }

    let new_user = database.create_user(credentials).await?;
    Ok(new_user)
}

pub async fn login(
    State(mut database): State<Store>,
    Json(creds): Json<User>
) -> Result<Json<Value>, AppError> {
    if creds.email.is_empty() || creds.password.is_empty() {
        return Err(AppError::MissingCredentials)
    }

    let existing_user = database.get_user(&creds.email).await?;

    if existing_user.password != creds.password {
        Err(AppError::MissingCredentials)
    } else{
        //create jwt to return
        let claims = Claims {
            id: 0,
            email: creds.email.to_owned(),
            exp: get_timestamp_after_8_hours(),
        };

        let token = jsonwebtoken::encode(&Header::default(), &claims, &KEYS.encoding)
            .map_err(|_| AppError::MissingCredentials)?;
        Ok(Json(json!({ "access_token" : token, "type": "Bearer"})))
    }
}

pub async fn protected (
    claims: Claims,
) -> Result<String, AppError> {
    Ok(format!(
        "Welcome to the PROTECTED area \n your claim data is: {}",
        claims
    ))
}