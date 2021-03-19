use futures_timer::Delay;
use select::document::Document;
use select::predicate::{Class, Attr};

use std::time::{Instant, Duration};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[derive(Deserialize, Debug)]
pub struct SteamPublishedFileDetails {
	pub publishedfileid: String,
	pub title: Option<String>,
	pub filename: Option<String>,
	pub creator: Option<String>,
	pub result: u8,
}

#[derive(Deserialize, Debug)]
pub struct SteamPublishedFileDetailsList {
	pub result: usize,
	pub resultcount: usize,
	pub publishedfiledetails: Vec<SteamPublishedFileDetails>,
}

#[derive(Deserialize, Debug)]
pub struct SteamGetPublishedFileDetailsResponse {
	pub response: SteamPublishedFileDetailsList,
}

pub struct StatusError {
	pub status_code: reqwest::StatusCode,
}

impl std::fmt::Display for StatusError {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "A request failed: Status code {}", self.status_code.as_str()) // user-facing output
	}
}

impl std::fmt::Debug for StatusError {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "A request failed after timeout: Status code {:?}: {:?} {{ file: {}, line: {} }}", self.status_code.as_str(), self.status_code.canonical_reason(), file!(), line!())
	}
}

impl std::error::Error for StatusError {}

pub async fn get_level_details(levels: &Vec<String>, timeout_ms: u64) -> Result<Vec<SteamPublishedFileDetails>> {
	let start = Instant::now();

	let mut req_body: String = format!("itemcount={}", levels.len().to_string());
	let mut index: usize = 0;
	for level_id in levels {
		req_body.push_str(&format!("&publishedfileids%5B{}%5D={}", index.to_string(), level_id));
		index += 1;
	}

	let response = loop {
		let client = reqwest::Client::new();
		let response = client
			.post("https://api.steampowered.com/ISteamRemoteStorage/GetPublishedFileDetails/v1/")
			.header("Content-Type", "application/x-www-form-urlencoded")
			.body(req_body.clone())
			.send()
			.await;

		if response.is_err() {
			return std::result::Result::Err(Box::new(response.unwrap_err()));
		}
		let response = response.unwrap();

		if response.status().is_success() {
			break response;
		} else if start.elapsed() >= Duration::from_millis(timeout_ms) {
			return Result::Err(Box::new(StatusError {status_code: response.status()}));
		}

		Delay::new(Duration::from_secs(1)).await;
	};

	let json_res = response.json::<SteamGetPublishedFileDetailsResponse>().await?;

	Ok(json_res.response.publishedfiledetails)
}

pub async fn get_distance_workshop_page(page: u32) -> Result<Option<Vec<String>>> {
	let client = reqwest::Client::new();
	let request_url = format!(
		"https://steamcommunity.com/workshop/browse/?appid=233610&actualsort=mostrecent&browsesort=mostrecent&p={}",
		page
	);
	let mut attempts: u8 = 0;
	let document = loop {
		let response = client.get(&request_url[..]).send().await?;

		let body_text = response.text().await?;

		let document = Document::from(&body_text[..]);

		if document.find(Attr("id", "no_items")).next() == None {
			break document;
		}

		attempts += 1;
		println!("no items! attempt: {}", attempts);
		if attempts == 5 {
			println!("assuming page is actually empty...");
			return Ok(None);
		}
		tokio::time::sleep(Duration::from_secs(5)).await;
	};

	Ok(Some(document.find(Class("ugc"))
		.map(|node| node.attr("data-publishedfileid"))
		.filter(Option::is_some)
		.map(|str_opt| String::from(str_opt.unwrap()))
		.collect::<Vec<String>>()))
}

pub struct DistanceWorkshopGetter {
	page_num: u32,
}

impl DistanceWorkshopGetter {
	pub async fn next_getter_page(&mut self) -> Result<Option<Vec<String>>> {
		self.page_num += 1;
		get_distance_workshop_page(self.page_num).await
	}
}

pub fn for_distance_workshop() -> DistanceWorkshopGetter {
	return DistanceWorkshopGetter{ page_num: 0};
}