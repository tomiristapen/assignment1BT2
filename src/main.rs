use actix_files::Files;
use actix_web::{web, App, HttpResponse, HttpServer, Responder, Result};
use dotenv::dotenv;
use reqwest;
use serde::Deserialize;
use std::{collections::HashMap, env, fs};

#[derive(Deserialize)]
struct Query {
    symbol: Option<String>,
}

#[derive(Deserialize, Debug)]
struct NewsApiResponse {
    results: Vec<NewsItem>,
}

#[derive(Deserialize, Debug)]
struct NewsItem {
    title: String,
    link: String,
    pubDate: String,
    source_url: Option<String>,
}

#[derive(Deserialize, Debug)]
struct CoinMarketCapResponse {
    data: HashMap<String, CoinInfo>,
}

#[derive(Deserialize, Debug)]
struct CoinInfo {
    name: String,
    symbol: String,
    description: String,
    urls: CoinUrls,
}

#[derive(Deserialize, Debug)]
struct CoinUrls {
    website: Vec<String>,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    HttpServer::new(|| {
        App::new()
            .route("/", web::get().to(get_news))  // Set "/" to show news
            .route("/news", web::get().to(get_news))
            .route("/info", web::get().to(get_info))
            .service(Files::new("/static", "./static").index_file("index.html"))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}

async fn get_news(query: web::Query<Query>) -> Result<HttpResponse> {
    let api_key = env::var("NEWSDATA_API_KEY").unwrap();

    let url = if let Some(symbol) = &query.symbol {
        format!(
            "https://newsdata.io/api/1/news?apikey={}&q={}&category=business&language=en",
            api_key, symbol
        )
    } else {
        format!(
            "https://newsdata.io/api/1/news?apikey={}&category=business&language=en",
            api_key
        )
    };

    let response = reqwest::get(&url).await;

    match response {
        Ok(res) => {
            if res.status().is_success() {
                match res.json::<NewsApiResponse>().await {
                    Ok(parsed) => {
                        let mut news_html = String::new();

                        if let Some(symbol) = &query.symbol {
                            news_html += &format!(
                                "<h2>{} Crypto News</h2>
                                 <form action=\"/news\" method=\"get\">
                                    <button type=\"submit\" class=\"btn btn-secondary mb-4\">Back to General News</button>
                                 </form>",
                                symbol.to_uppercase()
                            );
                        } else {
                            news_html += "<h2>Latest Crypto News</h2>";
                        }

                        news_html += "<ul>";
                        for article in parsed.results.iter().take(10) {
                            news_html += &format!(
                                "<li><a href=\"{}\">{}</a><br><small>{} | Source: {}</small></li><br>",
                                article.link,
                                article.title,
                                article.pubDate,
                                article.source_url.clone().unwrap_or("Unknown".to_string())
                            );
                        }
                        news_html += "</ul>";

                        let template = fs::read_to_string("./static/news_st.html").unwrap_or_default();
                        let final_html = template.replace("<!-- News will be populated here from the backend -->", &news_html);

                        Ok(HttpResponse::Ok().content_type("text/html").body(final_html))
                    }
                    Err(_) => Ok(HttpResponse::InternalServerError().body("Failed to parse response")),
                }
            } else {
                Ok(HttpResponse::InternalServerError().body("Failed to fetch news"))
            }
        }
        Err(_) => Ok(HttpResponse::InternalServerError().body("Request error")),
    }
}

async fn get_info(query: web::Query<Query>) -> impl Responder {
    let api_key = env::var("CMC_API_KEY").unwrap_or_else(|_| {
        eprintln!("API key not set!");
        String::new()
    });

    let symbol = query.symbol.clone().unwrap_or_default().to_uppercase();

    if symbol.is_empty() {
        // Render the `info.html` page with an empty `#info-container`
        let template = fs::read_to_string("./static/info.html")
            .unwrap_or_else(|_| "Error loading the page.".to_string());

        let final_html = template.replace(
            "{info_html}",
            "<div id=\"info-container\" class=\"mt-4\"></div>",
        );

        return HttpResponse::Ok().content_type("text/html").body(final_html);
    }

    let url = format!(
        "https://pro-api.coinmarketcap.com/v1/cryptocurrency/info?symbol={}",
        symbol
    );

    eprintln!("Requesting info for symbol: {}", symbol);
    eprintln!("API URL: {}", url);

    let client = reqwest::Client::new();
    let res = client
        .get(&url)
        .header("X-CMC_PRO_API_KEY", api_key)
        .send()
        .await;

    match res {
        Ok(resp) => {
            if resp.status().is_success() {
                let response_text = resp.text().await.unwrap_or_else(|_| "No response body".to_string());
                eprintln!("API Response: {}", response_text);

                match serde_json::from_str::<CoinMarketCapResponse>(&response_text) {
                    Ok(json_data) => {
                        if let Some(coin) = json_data.data.get(&symbol) {
                            let info_html = format!(
                                "<h2>{} ({}) Info</h2>
                                <p>{}</p>
                                <p>Website: <a href=\"{}\" target=\"_blank\">{}</a></p>",
                                coin.name,
                                coin.symbol,
                                coin.description,
                                coin.urls.website.join(", "),
                                coin.urls.website.join(", ")
                            );

                            // Read the static `info.html` template
                            let template = fs::read_to_string("./static/info.html")
                                .unwrap_or_else(|_| "Error loading the page.".to_string());

                            // Replace the placeholder `{info_html}` with the dynamically generated content
                            let final_html = template.replace("{info_html}", &info_html);

                            HttpResponse::Ok().content_type("text/html").body(final_html)
                        } else {
                            HttpResponse::Ok().body("No info found for this symbol.")
                        }
                    }
                    Err(err) => {
                        eprintln!("Failed to parse API response: {}", err);
                        HttpResponse::InternalServerError().body("Failed to parse info response")
                    }
                }
            } else {
                eprintln!("API returned error status: {}", resp.status());
                HttpResponse::InternalServerError().body("Failed to fetch info")
            }
        }
        Err(err) => {
            eprintln!("Error sending request: {}", err);
            HttpResponse::InternalServerError().body("Request error")
        }
    }
}
