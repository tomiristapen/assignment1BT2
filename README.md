#  **Cryptocurrency News Aggregator**

*A lightweight Rust-based web application that aggregates the latest cryptocurrency news, price data, and token information. It allows users to search by crypto symbol and presents results in a clean, user-friendly HTML interface.*

---

##  **Features**

-  **Search by Symbol** – Enter a crypto symbol (e.g., `BTC`) to get relevant news, prices, or info  
-  **Latest Crypto News** – Displays news from *NewsData.io* including title, date, and source  
-  **Top Prices** – View prices of top 12 coins or search by symbol  
-  **Crypto Info** – Get token descriptions and links from *CoinMarketCap*   

---

##  **Tech Stack**

- **Rust** (`actix-web`, `reqwest`, `serde`, `dotenv`)  
- **HTML/CSS** with *Bootstrap 5*  
- **External APIs**:  
  - [NewsData.io](https://newsdata.io/)  
  - [CoinMarketCap](https://coinmarketcap.com/)  

---

##  **Screenshots**

>

![Снимок экрана 2025-04-05 150725](https://github.com/user-attachments/assets/d7f7c3f4-1c6e-442c-846f-da6f3c9eb3c3)

![image2](https://github.com/user-attachments/assets/273a30e4-bce8-4942-b8b9-de5adf93b51c)

![image3](https://github.com/user-attachments/assets/252d1197-da20-4974-bc78-e367e0687b91)

![image4](https://github.com/user-attachments/assets/6843331d-aed6-475a-9084-4d494cab47e9)

![image5](https://github.com/user-attachments/assets/816bb6a0-eb80-44ac-b6d1-b713dd8987b1)


---

## 🧑‍💻 **Usage**

**1. Clone the repository**
```bash
git clone https://github.com/tomiristapen/assignment1BT2.git
cd assignment1BT2
```
**2. Create a .env file and add your API keys**
```bash
NEWSDATA_API_KEY=your_newsdata_api_key
CMC_API_KEY=your_coinmarketcap_api_key
```

**3. Run the server**
```bash
cargo run
```

**4. Open in browser**
```bash
http://127.0.0.1:8080
```

