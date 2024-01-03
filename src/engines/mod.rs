use std::{
    collections::{BTreeSet, HashMap},
    fmt,
    net::IpAddr,
    ops::Deref,
    str::FromStr,
    sync::LazyLock,
    time::Instant,
};

use futures::future::join_all;
use reqwest::header::HeaderMap;
use tokio::sync::mpsc;
use url::Url;

pub mod answer;
pub mod postsearch;
pub mod search;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Engine {
    // search
    Google,
    Bing,
    Brave,
    Marginalia,
    // answer
    Useragent,
    Ip,
    Calc,
    Wikipedia,
    Dictionary,
    // post-search
    StackExchange,
    GitHub,
    DocsRs,
}

impl Engine {
    pub fn all() -> &'static [Engine] {
        &[
            Engine::Google,
            Engine::Bing,
            Engine::Brave,
            Engine::Marginalia,
            // answer
            Engine::Useragent,
            Engine::Ip,
            Engine::Calc,
            Engine::Wikipedia,
            Engine::Dictionary,
            // post-search
            Engine::StackExchange,
            Engine::GitHub,
            Engine::DocsRs,
        ]
    }

    pub fn id(&self) -> &'static str {
        match self {
            Engine::Google => "google",
            Engine::Bing => "bing",
            Engine::Brave => "brave",
            Engine::Marginalia => "marginalia",
            // answer
            Engine::Useragent => "useragent",
            Engine::Ip => "ip",
            Engine::Calc => "calc",
            Engine::Wikipedia => "wikipedia",
            Engine::Dictionary => "dictionary",
            // post-search
            Engine::StackExchange => "stackexchange",
            Engine::GitHub => "github",
            Engine::DocsRs => "docs.rs",
        }
    }

    pub fn weight(&self) -> f64 {
        match self {
            Engine::Google => 1.05,
            Engine::Bing => 1.,
            Engine::Brave => 1.25,
            Engine::Marginalia => 0.15,
            _ => 1.,
        }
    }

    pub fn request(&self, query: &SearchQuery) -> RequestResponse {
        #[allow(clippy::useless_conversion)]
        match self {
            Engine::Google => search::google::request(query).into(),
            Engine::Bing => search::bing::request(query).into(),
            Engine::Brave => search::brave::request(query).into(),
            Engine::Marginalia => search::marginalia::request(query).into(),
            Engine::Useragent => answer::useragent::request(query).into(),
            Engine::Ip => answer::ip::request(query).into(),
            Engine::Calc => answer::calc::request(query).into(),
            Engine::Wikipedia => answer::wikipedia::request(query).into(),
            Engine::Dictionary => answer::dictionary::request(query).into(),
            _ => RequestResponse::None,
        }
    }

    pub fn parse_response(&self, res: &HttpResponse) -> eyre::Result<EngineResponse> {
        #[allow(clippy::useless_conversion)]
        match self {
            Engine::Google => search::google::parse_response(res.into()),
            Engine::Bing => search::bing::parse_response(res.into()),
            Engine::Brave => search::brave::parse_response(res.into()),
            Engine::Marginalia => search::marginalia::parse_response(res.into()),
            Engine::Wikipedia => answer::wikipedia::parse_response(res.into()),
            Engine::Dictionary => answer::dictionary::parse_response(res.into()),
            _ => eyre::bail!("engine {self:?} can't parse response"),
        }
    }

    pub fn request_autocomplete(&self, query: &str) -> Option<RequestAutocompleteResponse> {
        match self {
            Engine::Google => Some(search::google::request_autocomplete(query).into()),
            Engine::Calc => Some(answer::calc::request_autocomplete(query).into()),
            _ => None,
        }
    }

    pub fn parse_autocomplete_response(&self, body: &str) -> eyre::Result<Vec<String>> {
        match self {
            Engine::Google => search::google::parse_autocomplete_response(body),
            _ => eyre::bail!("engine {self:?} can't parse autocomplete response"),
        }
    }

    pub fn postsearch_request(&self, response: &Response) -> Option<reqwest::RequestBuilder> {
        match self {
            Engine::StackExchange => postsearch::stackexchange::request(response),
            Engine::GitHub => postsearch::github::request(response),
            Engine::DocsRs => postsearch::docs_rs::request(response),
            _ => None,
        }
    }

    pub fn postsearch_parse_response(&self, body: &str, url: Url) -> Option<String> {
        match self {
            Engine::StackExchange => postsearch::stackexchange::parse_response(body, url),
            Engine::GitHub => postsearch::github::parse_response(body, url),
            Engine::DocsRs => postsearch::docs_rs::parse_response(body, url),
            _ => None,
        }
    }
}

impl fmt::Display for Engine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.id())
    }
}

pub struct SearchQuery {
    pub query: String,
    pub request_headers: HashMap<String, String>,
    pub ip: String,
}

impl Deref for SearchQuery {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.query
    }
}

pub enum RequestResponse {
    None,
    Http(reqwest::RequestBuilder),
    Instant(EngineResponse),
}
impl From<reqwest::RequestBuilder> for RequestResponse {
    fn from(req: reqwest::RequestBuilder) -> Self {
        Self::Http(req)
    }
}
impl From<EngineResponse> for RequestResponse {
    fn from(res: EngineResponse) -> Self {
        Self::Instant(res)
    }
}

pub enum RequestAutocompleteResponse {
    Http(reqwest::RequestBuilder),
    Instant(Vec<String>),
}
impl From<reqwest::RequestBuilder> for RequestAutocompleteResponse {
    fn from(req: reqwest::RequestBuilder) -> Self {
        Self::Http(req)
    }
}
impl From<Vec<String>> for RequestAutocompleteResponse {
    fn from(res: Vec<String>) -> Self {
        Self::Instant(res)
    }
}

pub struct HttpResponse {
    pub res: reqwest::Response,
    pub body: String,
}

impl<'a> From<&'a HttpResponse> for &'a str {
    fn from(res: &'a HttpResponse) -> Self {
        &res.body
    }
}

impl From<HttpResponse> for reqwest::Response {
    fn from(res: HttpResponse) -> Self {
        res.res
    }
}

#[derive(Debug)]
pub struct EngineSearchResult {
    pub url: String,
    pub title: String,
    pub description: String,
}

#[derive(Debug)]
pub struct EngineFeaturedSnippet {
    pub url: String,
    pub title: String,
    pub description: String,
}

#[derive(Debug, Default)]
pub struct EngineResponse {
    pub search_results: Vec<EngineSearchResult>,
    pub featured_snippet: Option<EngineFeaturedSnippet>,
    pub answer_html: Option<String>,
    pub infobox_html: Option<String>,
}

impl EngineResponse {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn answer_html(html: String) -> Self {
        Self {
            answer_html: Some(html),
            ..Default::default()
        }
    }

    pub fn infobox_html(html: String) -> Self {
        Self {
            infobox_html: Some(html),
            ..Default::default()
        }
    }
}

#[derive(Debug)]
pub enum EngineProgressUpdate {
    Requesting,
    Downloading,
    Parsing,
    Done,
}

#[derive(Debug)]
pub enum ProgressUpdateData {
    Engine {
        engine: Engine,
        update: EngineProgressUpdate,
    },
    Response(Response),
    PostSearchInfobox(Infobox),
}

#[derive(Debug)]
pub struct ProgressUpdate {
    pub data: ProgressUpdateData,
    pub time_ms: u64,
}

impl ProgressUpdate {
    pub fn new(data: ProgressUpdateData, start_time: Instant) -> Self {
        Self {
            data,
            time_ms: start_time.elapsed().as_millis() as u64,
        }
    }
}

pub async fn search_with_engines(
    engines: &[Engine],
    query: &SearchQuery,
    progress_tx: mpsc::UnboundedSender<ProgressUpdate>,
) -> eyre::Result<()> {
    let start_time = Instant::now();

    let mut requests = Vec::new();
    for engine in engines {
        requests.push(async {
            let engine = *engine;

            let request_response = engine.request(query);

            let response = match request_response {
                RequestResponse::Http(request) => {
                    progress_tx.send(ProgressUpdate::new(
                        ProgressUpdateData::Engine {
                            engine,
                            update: EngineProgressUpdate::Requesting,
                        },
                        start_time,
                    ))?;

                    let mut res = request.send().await?;

                    progress_tx.send(ProgressUpdate::new(
                        ProgressUpdateData::Engine {
                            engine,
                            update: EngineProgressUpdate::Downloading,
                        },
                        start_time,
                    ))?;

                    let mut body_bytes = Vec::new();
                    while let Some(chunk) = res.chunk().await? {
                        body_bytes.extend_from_slice(&chunk);
                    }
                    let body = String::from_utf8_lossy(&body_bytes).to_string();

                    progress_tx.send(ProgressUpdate::new(
                        ProgressUpdateData::Engine {
                            engine,
                            update: EngineProgressUpdate::Parsing,
                        },
                        start_time,
                    ))?;

                    let http_response = HttpResponse { res, body };

                    let response = match engine.parse_response(&http_response) {
                        Ok(response) => response,
                        Err(e) => {
                            eprintln!("parse error: {}", e);
                            EngineResponse::new()
                        }
                    };

                    progress_tx.send(ProgressUpdate::new(
                        ProgressUpdateData::Engine {
                            engine,
                            update: EngineProgressUpdate::Done,
                        },
                        start_time,
                    ))?;

                    response
                }
                RequestResponse::Instant(response) => response,
                RequestResponse::None => EngineResponse::new(),
            };

            Ok((engine, response))
        });
    }

    let mut response_futures = Vec::new();
    for request in requests {
        response_futures.push(request);
    }

    let responses_result: eyre::Result<HashMap<_, _>> =
        join_all(response_futures).await.into_iter().collect();
    let responses = responses_result?;

    let response = merge_engine_responses(responses);

    let has_infobox = response.infobox.is_some();

    progress_tx.send(ProgressUpdate::new(
        ProgressUpdateData::Response(response.clone()),
        start_time,
    ))?;

    if !has_infobox {
        // post-search

        let mut postsearch_requests = Vec::new();
        for engine in engines {
            if let Some(request) = engine.postsearch_request(&response) {
                postsearch_requests.push(async {
                    let response = match request.send().await {
                        Ok(res) => {
                            let url = res.url().clone();
                            let body = res.text().await?;
                            engine.postsearch_parse_response(&body, url)
                        }
                        Err(e) => {
                            eprintln!("postsearch request error: {}", e);
                            None
                        }
                    };
                    Ok((*engine, response))
                });
            }
        }

        let mut postsearch_response_futures = Vec::new();
        for request in postsearch_requests {
            postsearch_response_futures.push(request);
        }

        let postsearch_responses_result: eyre::Result<HashMap<_, _>> =
            join_all(postsearch_response_futures)
                .await
                .into_iter()
                .collect();
        let postsearch_responses = postsearch_responses_result?;

        for (engine, response) in postsearch_responses {
            if let Some(html) = response {
                progress_tx.send(ProgressUpdate::new(
                    ProgressUpdateData::PostSearchInfobox(Infobox { html, engine }),
                    start_time,
                ))?;
                // break so we don't send multiple infoboxes
                break;
            }
        }
    }

    Ok(())
}

pub async fn autocomplete_with_engines(
    engines: &[Engine],
    query: &str,
) -> eyre::Result<Vec<String>> {
    let mut requests = Vec::new();
    for engine in engines {
        if let Some(request) = engine.request_autocomplete(query) {
            requests.push(async {
                let response = match request {
                    RequestAutocompleteResponse::Http(request) => {
                        let res = request.send().await?;
                        let body = res.text().await?;
                        engine.parse_autocomplete_response(&body)?
                    }
                    RequestAutocompleteResponse::Instant(response) => response,
                };
                Ok((*engine, response))
            });
        }
    }

    let mut autocomplete_futures = Vec::new();
    for request in requests {
        autocomplete_futures.push(request);
    }

    let autocomplete_results_result: eyre::Result<HashMap<_, _>> =
        join_all(autocomplete_futures).await.into_iter().collect();
    let autocomplete_results = autocomplete_results_result?;

    Ok(merge_autocomplete_responses(autocomplete_results))
}

pub static CLIENT: LazyLock<reqwest::Client> = LazyLock::new(|| {
    reqwest::ClientBuilder::new()
        .local_address(IpAddr::from_str("0.0.0.0").unwrap())
        .default_headers({
            let mut headers = HeaderMap::new();
            // we pretend to be a normal browser so websites don't block us
            // (since we're not entirely a bot, we're acting on behalf of the user)
            headers.insert(
                "User-Agent",
                "Mozilla/5.0 (X11; Linux x86_64; rv:121.0) Gecko/20100101 Firefox/121.0"
                    .parse()
                    .unwrap(),
            );
            headers.insert("Accept-Language", "en-US,en;q=0.5".parse().unwrap());
            headers
        })
        .build()
        .unwrap()
});

pub async fn search(
    query: SearchQuery,
    progress_tx: mpsc::UnboundedSender<ProgressUpdate>,
) -> eyre::Result<()> {
    let engines = Engine::all();
    search_with_engines(engines, &query, progress_tx).await
}

pub async fn autocomplete(query: &str) -> eyre::Result<Vec<String>> {
    let engines = Engine::all();
    autocomplete_with_engines(engines, query).await
}

#[derive(Debug, Clone)]
pub struct Response {
    pub search_results: Vec<SearchResult>,
    pub featured_snippet: Option<FeaturedSnippet>,
    pub answer: Option<Answer>,
    pub infobox: Option<Infobox>,
}

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub url: String,
    pub title: String,
    pub description: String,
    pub engines: BTreeSet<Engine>,
    pub score: f64,
}

#[derive(Debug, Clone)]
pub struct FeaturedSnippet {
    pub url: String,
    pub title: String,
    pub description: String,
    pub engine: Engine,
}

#[derive(Debug, Clone)]
pub struct Answer {
    pub html: String,
    pub engine: Engine,
}

#[derive(Debug, Clone)]
pub struct Infobox {
    pub html: String,
    pub engine: Engine,
}

fn merge_engine_responses(responses: HashMap<Engine, EngineResponse>) -> Response {
    let mut search_results: Vec<SearchResult> = Vec::new();
    let mut featured_snippet: Option<FeaturedSnippet> = None;
    let mut answer: Option<Answer> = None;
    let mut infobox: Option<Infobox> = None;

    for (engine, response) in responses {
        for (result_index, search_result) in response.search_results.into_iter().enumerate() {
            // position 1 has a score of 1, position 2 has a score of 0.5, position 3 has a
            // score of 0.33, etc.
            let base_result_score = 1. / (result_index + 1) as f64;
            let result_score = base_result_score * engine.weight();

            if let Some(existing_result) = search_results
                .iter_mut()
                .find(|r| r.url == search_result.url)
            {
                // if the weight of this engine is higher than every other one then replace the
                // title and description
                if engine.weight()
                    > existing_result
                        .engines
                        .iter()
                        .map(Engine::weight)
                        .max_by(|a, b| a.partial_cmp(b).unwrap())
                        .unwrap_or(0.)
                {
                    existing_result.title = search_result.title;
                    existing_result.description = search_result.description;
                }

                existing_result.engines.insert(engine);
                existing_result.score += result_score;
            } else {
                search_results.push(SearchResult {
                    url: search_result.url,
                    title: search_result.title,
                    description: search_result.description,
                    engines: [engine].iter().cloned().collect(),
                    score: result_score,
                });
            }
        }

        if let Some(engine_featured_snippet) = response.featured_snippet {
            // if it has a higher weight than the current featured snippet
            let featured_snippet_weight = featured_snippet
                .as_ref()
                .map(|s| s.engine.weight())
                .unwrap_or(0.);
            if engine.weight() > featured_snippet_weight {
                featured_snippet = Some(FeaturedSnippet {
                    url: engine_featured_snippet.url,
                    title: engine_featured_snippet.title,
                    description: engine_featured_snippet.description,
                    engine,
                });
            }
        }

        if let Some(engine_answer_html) = response.answer_html {
            // if it has a higher weight than the current answer
            let answer_weight = answer.as_ref().map(|s| s.engine.weight()).unwrap_or(0.);
            if engine.weight() > answer_weight {
                answer = Some(Answer {
                    html: engine_answer_html,
                    engine,
                });
            }
        }

        if let Some(engine_infobox_html) = response.infobox_html {
            // if it has a higher weight than the current infobox
            let infobox_weight = infobox.as_ref().map(|s| s.engine.weight()).unwrap_or(0.);
            if engine.weight() > infobox_weight {
                infobox = Some(Infobox {
                    html: engine_infobox_html,
                    engine,
                });
            }
        }
    }

    search_results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());

    Response {
        search_results,
        featured_snippet,
        answer,
        infobox,
    }
}

pub struct AutocompleteResult {
    pub query: String,
    pub score: f64,
}

fn merge_autocomplete_responses(responses: HashMap<Engine, Vec<String>>) -> Vec<String> {
    let mut autocomplete_results: Vec<AutocompleteResult> = Vec::new();

    for (engine, response) in responses {
        for (result_index, autocomplete_result) in response.into_iter().enumerate() {
            // position 1 has a score of 1, position 2 has a score of 0.5, position 3 has a
            // score of 0.33, etc.
            let base_result_score = 1. / (result_index + 1) as f64;
            let result_score = base_result_score * engine.weight();

            if let Some(existing_result) = autocomplete_results
                .iter_mut()
                .find(|r| r.query == autocomplete_result)
            {
                existing_result.score += result_score;
            } else {
                autocomplete_results.push(AutocompleteResult {
                    query: autocomplete_result,
                    score: result_score,
                });
            }
        }
    }

    autocomplete_results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());

    autocomplete_results.into_iter().map(|r| r.query).collect()
}
