use serde::Deserialize;
use url::Url;

const BASE_URL: &str = "https://newsapi.org/v2/";

#[derive(thiserror::Error, Debug)]
pub enum NewsApiError {
	#[error("Failed fetching articles")]
  RequestFailed(#[from] ureq::Error),
	#[error("Failed converting response to string")]
	ResponseConversionFailure(#[from] std::io::Error),
	#[error("Article Parsing failed")]
	ArticleParseFailed(#[from] serde_json::Error),
	#[error("Url parsing failed")]
	UrlParsing(#[from] url::ParseError),
	#[error("Request failed: {0}")]
	BadRequest(&'static str)
}

#[derive(Deserialize, Debug)]
pub struct Article {
	title: String,
	url: String
}

impl Article {
	pub fn title(&self) -> &str {
		&self.title
	}

	pub fn url(&self) -> &str {
		&self.url
	}
}

#[derive(Deserialize, Debug)]
pub struct NewsAPIResponse {
	pub articles: Vec<Article>,
	status: String,
	code: Option<String>
}

impl NewsAPIResponse {
	pub fn articles(&self) -> &Vec<Article> {
		&self.articles
	}
}

pub enum Endpoint {
	TopHeadlines,
}

impl ToString for Endpoint {
	fn to_string(&self) -> String {
		match self {
			Self::TopHeadlines => "top-headlines".to_string()
		}
	}
}

pub enum Country {
	Us
}

impl ToString for Country {
	fn to_string(&self) -> String {
		match self {
			Self::Us => "us".to_string()
		}
	}
}


pub struct NewsAPI {
	api_key: String,
	endpoint: Endpoint,
	country: Country,
}

impl NewsAPI {
	pub fn new(api_key: &str) -> NewsAPI {
		NewsAPI {
			api_key: api_key.to_string(),
			endpoint: Endpoint::TopHeadlines,
			country: Country::Us
		}
	}

	pub fn endpoint(&mut self, endpoint: Endpoint) -> &mut NewsAPI {
		self.endpoint = endpoint;
		self
	}
	
	pub fn country(&mut self, country: Country) -> &mut NewsAPI {
		self.country = country;
		self
	}

	fn prepare_url(&self) -> Result<String, NewsApiError> {
		let mut url = Url::parse(BASE_URL)?;
		url.path_segments_mut().unwrap().push(&self.endpoint.to_string());
		let country = format!("country={}", self.country.to_string());
		url.set_query(Some(&country));
		
		Ok(url.to_string())
	}

	pub fn fetch(&self) -> Result<NewsAPIResponse, NewsApiError> {
		let url = self.prepare_url()?;
		let req = ureq::get(&url)
			.set("Authorization", &self.api_key);
		let response: NewsAPIResponse = req.call()?.into_json()?;
		match response.status.as_str() {
			"ok" => return Ok(response),
			_ => return Err(map_response_err(response.code))
		}
	}
}

fn map_response_err(code: Option<String>) -> NewsApiError {
	if let Some(code) = code {
		match code.as_str() {
			"apiKeyDisabled" => NewsApiError::BadRequest("Your API Key has been disabled"),
			_ => NewsApiError::BadRequest("Unknown Error")
		}
	} else {
		NewsApiError::BadRequest("Unknown Error")
	}
}