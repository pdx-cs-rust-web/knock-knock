use crate::*;

pub async fn handler_index(State(jokebase): State<Arc<RwLock<JokeBase>>>) -> Response {
    match jokebase.read().await.get_random() {
        Some(joke) => {
            let joke_string = format!(
                r#"
<html>
<head>
  <title>Knock-Knock!</title>
</head>
<body>
  <p>
    Knock-Knock!<br/>
    Who's there?<br/>
    {}<br/>
    {} who?<br/>
    {}<br/>
  </p>
</body>
</html>
                "#,
                joke.whos_there,
                joke.whos_there,
                joke.answer_who,
            );
            (StatusCode::OK, joke_string).into_response()
        },
        None => (StatusCode::NOT_FOUND, "404 Not Found").into_response(),
    }
}
