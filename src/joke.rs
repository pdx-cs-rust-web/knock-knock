use crate::*;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Joke {
    #[schema(example = "boo")]
    pub id: String,
    #[schema(example = "Boo")]
    pub whos_there: String,
    #[schema(example = "You don't have to cry about it!")]
    pub answer_who: String,
    #[schema(example = r#"["kids", "food"]"#)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<HashSet<String>>,
    #[schema(example = "http://example.com/knock-knock-jokes")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
}

impl Joke {
    pub fn new(
        id: &str,
        whos_there: &str,
        answer_who: &str,
        tags: &[&str],
        source: Option<&str>,
    ) -> Self {
        let id = id.into();
        let whos_there = whos_there.into();
        let answer_who = answer_who.into();
        let tags: Option<HashSet<String>> = if tags.is_empty() {
            None
        } else {
            Some(tags.iter().copied().map(String::from).collect())
        };
        let source = source.map(String::from);
        Self {
            id,
            whos_there,
            answer_who,
            tags,
            source,
        }
    }
}

pub fn format_tags(tags: &HashSet<String>) -> String {
    let taglist: Vec<&str> = tags.iter().map(String::as_ref).collect();
    taglist.join(", ")
}

impl From<&Joke> for String {
    fn from(joke: &Joke) -> Self {
        let mut text: String = "Knock knock!\n".into();
        text += "Who's there?\n";
        text += &format!("{}.\n", joke.whos_there);
        text += &format!("\"{}\" who?\n", joke.whos_there);
        text += &format!("{}\n", joke.answer_who);
        text += "\n";

        let mut annote: Vec<String> = vec![format!("id: {}", joke.id)];
        if let Some(tags) = &joke.tags {
            annote.push(format!("tags: {}", format_tags(tags)));
        }
        if let Some(source) = &joke.source {
            annote.push(format!(r#"source: "{}""#, source));
        }
        let annote = annote.join("; ");
        text += &format!("[{}]\n", annote);
        text
    }
}

impl IntoResponse for &Joke {
    fn into_response(self) -> Response {
        (StatusCode::OK, Json(&self)).into_response()
    }
}
