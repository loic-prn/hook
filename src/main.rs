use std::{fs::File, io::Write, path::Path};
use chrono::Local;
use clap::Parser;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
	#[clap(short, long)]
	sessionid: String,
	#[clap(short, long)]
	username: String
}

const BASE_URL: &str = "https://instagram.com/";
const USER_AGENT: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/94.0.4606.71 Safari/537.36";
const END_URL: &str = "/?__a=1&__d=dis";

#[tokio::main]
async fn main() {
	let args = Args::parse();
	let header = create_header(&args);

	let client = reqwest::Client::builder()
		.user_agent(USER_AGENT)
		.default_headers(header)
		.build()
		.unwrap();

	let profil_url = BASE_URL.to_owned() + args.username.as_str() + END_URL;
	let schema = load_json(profil_url.as_str(), &client).await;
	println!("{}", schema);
	let folder = check_dir(schema["graphql"]["user"]["username"].as_str().unwrap());


	let posts = &schema["graphql"]["user"]["edge_owner_to_timeline_media"]["edges"];

	let posts_array = posts.as_array().unwrap();

	for post in posts_array {
		let post_url = BASE_URL.to_owned() + "p/" + post["node"]["shortcode"].as_str().unwrap() + END_URL;
		let post_schema = load_json(post_url.as_str(), &client).await;

		if &post_schema["graphql"]["shortcode_media"]["__typename"] == "GraphImage" {
			let url_src = post_schema["graphql"]["shortcode_media"]["display_url"].as_str().unwrap();
			download_image(url_src, &client, &folder, 0).await;
		}
		else if &post_schema["graphql"]["shortcode_media"]["__typename"] == "GraphSidecar" {
			let mut i = 0;
			for media in post_schema["graphql"]["shortcode_media"]["edge_sidecar_to_children"]["edges"].as_array().unwrap() {
				let url_src = media["node"]["display_url"].as_str().unwrap();
				download_image(url_src, &client, &folder, i).await;
				i+=1;
			}
		}
		else if &post_schema["graphql"]["shortcode_media"]["__typename"] == "GraphVideo" {
			let url_src = post_schema["graphql"]["shortcode_media"]["video_url"].as_str().unwrap();
			download_video(url_src, &client, &folder, 0).await;
		}
		else {
			println!("unknown type");
		}
	}
}

fn check_dir(username: &str) -> &Path{
	let path = Path::new(username);
	if !path.exists() {
		std::fs::create_dir_all(username).unwrap();
	}
	return path;
}

async fn download_image(url_to_download: &str, client: &reqwest::Client, folder: &Path, inc: i32) {
	let date = Local::now().format("%Y-%m-%d_%H-%M-%S").to_string();
	let output_name = "test";
	let pic_path = format!("{}/{}{}-{}{}", folder.to_str().unwrap(), output_name, date, inc, ".jpg");

	let mut file = File::create(pic_path).unwrap();
	let resp = client.get(url_to_download).send().await.unwrap();
	file.write(resp.bytes().await.unwrap().as_ref()).unwrap();
}

async fn download_video(url_to_download: &str, client: &reqwest::Client, folder: &Path, inc: i32){
	let date = Local::now().format("%Y-%m-%d_%H-%M-%S").to_string();
	let output_name = "test";
	let pic_path = format!("{}/{}{}-{}{}", folder.to_str().unwrap(), output_name, date, inc, ".mp4");

	let mut file = File::create(pic_path).unwrap();
	let resp = client.get(url_to_download).send().await.unwrap();
	file.write(resp.bytes().await.unwrap().as_ref()).unwrap();
}



async fn load_json(url: &str, client: &reqwest::Client) -> serde_json::Value{
	let res = client.get(url).send().await.unwrap();
	let body = res.text().await.unwrap();
	let parsed = serde_json::from_str::<serde_json::Value>(&body).unwrap();
	return parsed;
}


fn create_header(args: &Args)-> reqwest::header::HeaderMap {
	let mut header = reqwest::header::HeaderMap::new();
	//let sessiondid_cookie = "sessionid=".to_owned() + args.sessionid.as_str();
	//header.insert(
	//	reqwest::header::COOKIE,
	//	reqwest::header::HeaderValue::from_str(sessiondid_cookie.as_str()).unwrap()
	//);
	//header.insert(
	//	reqwest::header::ACCEPT,
	//	reqwest::header::HeaderValue::from_static("*/*")
	//);
	//header.insert(
	//	reqwest::header::ACCEPT_ENCODING,
	//	reqwest::header::HeaderValue::from_static("gzip, deflate, br")
	//);
	//header.insert(
	//	reqwest::header::ACCEPT_LANGUAGE,
	//	reqwest::header::HeaderValue::from_static("en-US,en;q=0.5")
	//);
	//header.insert(
	//	reqwest::header::UPGRADE_INSECURE_REQUESTS,
	//	reqwest::header::HeaderValue::from_static("1")
	//);
	//header.insert(
	//	reqwest::header::ACCEPT,
	//	reqwest::header::HeaderValue::from_static("text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8")
	//);
	return header;
}