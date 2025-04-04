use actix_files::Files;
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use serde::Deserialize;
use std::env;
use dotenv::dotenv;
use reqwest;

#[derive(Deserialize)]
struct Query {
    symbol: String,
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
    data: std::collections::HashMap<String, CoinInfo>,
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

#[derive(Deserialize, Debug)]
struct CryptoPriceResponse {
    symbol: String,
    price: f64,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    HttpServer::new(|| {
        App::new()
            .route("/", web::get().to(index)) // Route to homepage
            .route("/news", web::get().to(get_news)) // News route
            .route("/info", web::get().to(get_info)) // CoinMarketCap route
            .route("/prices", web::get().to(get_prices)) // Prices route
            .service(Files::new("/static", "./static").show_files_listing()) // Serve static files (HTML)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}

// Serve the index.html on the homepage
async fn index() -> impl Responder {
    let html = std::fs::read_to_string("./static/index.html")
        .unwrap_or_else(|_| "Error loading the page.".to_string());
    HttpResponse::Ok().content_type("text/html").body(html)
}

async fn get_news(query: web::Query<Query>) -> impl Responder {
    let api_key = env::var("NEWSDATA_API_KEY").unwrap();

    // If symbol is not provided, fetch the latest 10 news articles
    let url = if query.symbol.is_empty() {
        format!(
            "https://newsdata.io/api/1/news?apikey={}&category=business&language=en",
            api_key
        )
    } else {
        format!(
            "https://newsdata.io/api/1/news?apikey={}&q={}&category=business&language=en",
            api_key, query.symbol
        )
    };

    let response = reqwest::get(&url).await;

    match response {
        Ok(res) => {
            if res.status().is_success() {
                match res.json::<NewsApiResponse>().await {
                    Ok(parsed) => {
                        let mut html = if query.symbol.is_empty() {
                            String::from("<h2>Latest Crypto News</h2><ul>")
                        } else {
                            format!(
                                "<h2>{} Crypto News</h2><button onclick=\"window.location.href='/news'\" class=\"btn btn-secondary mb-4\">Back to Recent News</button><ul>",
                                query.symbol.to_uppercase()
                            )
                        };

                        for article in parsed.results.iter().take(10) {
                            html += &format!(
                                "<li><a href=\"{}\">{}</a><br><small>{} | Source: {}</small></li><br>",
                                article.link,
                                article.title,
                                article.pubDate,
                                article.source_url.clone().unwrap_or_default()
                            );
                        }
                        html += "</ul>";
                        HttpResponse::Ok()
                            .content_type("text/html")
                            .body(html)
                    }
                    Err(_) => HttpResponse::InternalServerError().body("Failed to parse response"),
                }
            } else {
                eprintln!("Error fetching news, status: {}", res.status());
                HttpResponse::InternalServerError().body("Error fetching news")
            }
        }
        Err(e) => {
            eprintln!("Error sending request: {}", e);
            HttpResponse::InternalServerError().body("Error fetching news")
        }
    }
}

async fn get_info(query: web::Query<Query>) -> impl Responder {
    let api_key = env::var("CMC_API_KEY").unwrap();
    let url = format!(
        "https://pro-api.coinmarketcap.com/v1/cryptocurrency/info?symbol={}",
        query.symbol.to_uppercase()
    );

    let client = reqwest::Client::new();
    let res = client
        .get(&url)
        .header("X-CMC_PRO_API_KEY", api_key)
        .send()
        .await;

    match res {
        Ok(resp) => {
            if resp.status().is_success() {
                match resp.json::<CoinMarketCapResponse>().await {
                    Ok(json_data) => {
                        if let Some(coin) = json_data.data.get(&query.symbol.to_uppercase()) {
                            let name = &coin.name;
                            let symbol = &coin.symbol;
                            let description = &coin.description;
                            let website = &coin.urls.website.join(", ");

                            let html = format!(
                                "<h2>{} ({}) Info</h2><p>{}</p><p>Website: <a href=\"{}\" target=\"_blank\">{}</a></p>",
                                name, symbol, description, website, website
                            );
                            HttpResponse::Ok().content_type("text/html").body(html)
                        } else {
                            HttpResponse::InternalServerError().body("Error fetching info")
                        }
                    }
                    Err(_) => HttpResponse::InternalServerError().body("Error parsing CoinMarketCap response"),
                }
            } else {
                eprintln!("Error fetching info, status: {}", resp.status());
                HttpResponse::InternalServerError().body("Error fetching info")
            }
        }
        Err(e) => {
            eprintln!("Error sending request: {}", e);
            HttpResponse::InternalServerError().body("Error fetching info")
        }
    }
}

async fn get_prices(query: web::Query<Query>) -> impl Responder {
    let api_key = env::var("CMC_API_KEY").unwrap();

    // Check if a symbol is provided. If not, return the top 10 cryptocurrencies.
    let url = if query.symbol.is_empty() {
        // Fetch top 10 cryptocurrencies if no symbol is provided
        "https://pro-api.coinmarketcap.com/v1/cryptocurrency/listings/latest?limit=10".to_string()
    } else {
        // Search specific cryptocurrency if a symbol is provided
        format!(
            "https://pro-api.coinmarketcap.com/v1/cryptocurrency/listings/latest?symbol={}&limit=1",
            query.symbol.to_uppercase()
        )
    };

    let client = reqwest::Client::new();
    let res = client
        .get(&url)
        .header("X-CMC_PRO_API_KEY", api_key)
        .send()
        .await;

    match res {
        Ok(resp) => {
            let status = resp.status();
            let response_body = resp.text().await.unwrap_or_else(|_| String::from("No body"));

            if status.is_success() {
                match serde_json::from_str::<Vec<CryptoPriceResponse>>(&response_body) {
                    Ok(prices) => {
                        let mut html = String::from("<h2>Cryptocurrency Prices</h2><ul>");
                        for price in prices.iter() {
                            html += &format!(
                                "<li>{}: ${}</li><br>",
                                price.symbol.to_uppercase(),
                                price.price
                            );
                        }
                        html += "</ul>";
                        HttpResponse::Ok()
                            .content_type("text/html")
                            .body(html)
                    }
                    Err(_) => {
                        eprintln!("Failed to parse response");
                        HttpResponse::InternalServerError().body("Failed to parse response")
                    }
                }
            } else {
                eprintln!("Error fetching prices, status: {}, body: {}", status, response_body);
                HttpResponse::InternalServerError().body("Error fetching prices")
            }
        }
        Err(e) => {
            eprintln!("Error sending request: {}", e);
            HttpResponse::InternalServerError().body("Error fetching prices")
        }
    }
}
