#[derive(Debug, Clone)]
pub struct Endpoints {
    pub base_info_url: String,
    pub base_options_url: String,
    pub base_options_gw_url: String,
    pub base_paper_url: String,
    pub base_quote_url: String,
    pub base_securities_url: String,
    pub base_trade_url: String,
    pub base_user_url: String,
    pub base_userbroker_url: String,
    pub base_ustrade_url: String,
    pub base_paperfintech_url: String,
    pub base_fintech_gw_url: String,
    pub base_userfintech_url: String,
    pub base_new_trade_url: String,
    pub base_ustradebroker_url: String,
    pub base_securitiesfintech_url: String,
}

impl Default for Endpoints {
    fn default() -> Self {
        Self {
            base_info_url: "https://infoapi.webull.com/api".to_string(),
            base_options_url: "https://quoteapi.webullbroker.com/api".to_string(),
            base_options_gw_url: "https://quotes-gw.webullbroker.com/api".to_string(),
            base_paper_url: "https://act.webullbroker.com/webull-paper-center/api".to_string(),
            base_quote_url: "https://quoteapi.webullbroker.com/api".to_string(),
            base_securities_url: "https://securitiesapi.webullbroker.com/api".to_string(),
            base_trade_url: "https://tradeapi.webullbroker.com/api/trade".to_string(),
            base_user_url: "https://userapi.webull.com/api".to_string(),
            base_userbroker_url: "https://userapi.webullbroker.com/api".to_string(),
            base_ustrade_url: "https://ustrade.webullfinance.com/api".to_string(),
            base_paperfintech_url: "https://act.webullfintech.com/webull-paper-center/api".to_string(),
            base_fintech_gw_url: "https://quotes-gw.webullfintech.com/api".to_string(),
            base_userfintech_url: "https://u1suser.webullfintech.com/api".to_string(),
            base_new_trade_url: "https://trade.webullfintech.com/api".to_string(),
            base_ustradebroker_url: "https://ustrade.webullbroker.com/api".to_string(),
            base_securitiesfintech_url: "https://securitiesapi.webullfintech.com/api".to_string(),
        }
    }
}

impl Endpoints {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn account(&self, account_id: &str) -> String {
        format!("{}/v3/home/{}", self.base_trade_url, account_id)
    }

    pub fn account_id(&self) -> String {
        format!("{}/account/getSecAccountList/v5", self.base_trade_url)
    }

    pub fn account_activities(&self, account_id: &str) -> String {
        format!("{}/trade/v2/funds/{}/activities", self.base_ustrade_url, account_id)
    }

    pub fn active_gainers_losers(&self, direction: &str, region_code: i32, rank_type: &str, num: i32) -> String {
        let url = match direction {
            "gainer" => "topGainers",
            "loser" => "dropGainers",
            _ => "topActive",
        };
        format!(
            "{}/wlas/ranking/{}?regionId={}&rankType={}&pageIndex=1&pageSize={}",
            self.base_fintech_gw_url, url, region_code, rank_type, num
        )
    }

    pub fn add_alert(&self) -> String {
        format!("{}/user/warning/v2/manage/overlap", self.base_userbroker_url)
    }

    pub fn analysis(&self, stock: &str) -> String {
        format!("{}/securities/ticker/v5/analysis/{}", self.base_securities_url, stock)
    }

    pub fn analysis_shortinterest(&self, stock: &str) -> String {
        format!("{}/securities/stock/{}/shortInterest", self.base_securities_url, stock)
    }

    pub fn analysis_institutional_holding(&self, stock: &str) -> String {
        format!("{}/securities/stock/v5/{}/institutionalHolding", self.base_securities_url, stock)
    }

    pub fn analysis_etf_holding(&self, stock: &str, has_num: i32, page_size: i32) -> String {
        format!(
            "{}/securities/stock/v5/{}/belongEtf?hasNum={}&pageSize={}",
            self.base_securities_url, stock, has_num, page_size
        )
    }

    pub fn analysis_capital_flow(&self, stock: &str, show_hist: bool) -> String {
        format!(
            "{}/wlas/capitalflow/ticker?tickerId={}&showHis={}",
            self.base_securities_url, stock, show_hist
        )
    }

    pub fn bars(&self, stock: &str, interval: &str, count: i32, timestamp: Option<i64>) -> String {
        let ts = timestamp.map(|t| format!("&timestamp={}", t)).unwrap_or_default();
        format!(
            "{}/quote/charts/query?tickerIds={}&type={}&count={}{}",
            self.base_fintech_gw_url, stock, interval, count, ts
        )
    }

    pub fn bars_crypto(&self, stock: &str) -> String {
        format!("{}/crypto/charts/query?tickerIds={}", self.base_fintech_gw_url, stock)
    }

    pub fn cancel_order(&self, account_id: &str) -> String {
        format!("{}/trade/order/{}/cancelStockOrder/", self.base_ustrade_url, account_id)
    }

    pub fn modify_otoco_orders(&self, account_id: &str) -> String {
        format!("{}/trade/v2/corder/stock/modify/{}", self.base_ustrade_url, account_id)
    }

    pub fn cancel_otoco_orders(&self, account_id: &str, combo_id: &str) -> String {
        format!("{}/trade/v2/corder/stock/cancel/{}/{}", self.base_ustrade_url, account_id, combo_id)
    }

    pub fn check_otoco_orders(&self, account_id: &str) -> String {
        format!("{}/trade/v2/corder/stock/check/{}", self.base_ustrade_url, account_id)
    }

    pub fn place_otoco_orders(&self, account_id: &str) -> String {
        format!("{}/trade/v2/corder/stock/place/{}", self.base_ustrade_url, account_id)
    }

    pub fn dividends(&self, account_id: &str) -> String {
        format!("{}/v2/account/{}/dividends?direct=in", self.base_trade_url, account_id)
    }

    pub fn fundamentals(&self, stock: &str) -> String {
        format!("{}/securities/financial/index/{}", self.base_securities_url, stock)
    }

    pub fn is_tradable(&self, stock: &str) -> String {
        format!("{}/ticker/broker/permissionV2?tickerId={}", self.base_trade_url, stock)
    }

    pub fn list_alerts(&self) -> String {
        format!("{}/user/warning/v2/query/tickers", self.base_userbroker_url)
    }

    pub fn login(&self) -> String {
        format!("{}/user/v1/login/account/v2", self.base_userfintech_url)
    }

    pub fn get_mfa(&self) -> String {
        format!("{}/user/v1/verificationCode/send/v2", self.base_user_url)
    }

    pub fn check_mfa(&self) -> String {
        format!("{}/user/v1/verificationCode/checkCode", self.base_userfintech_url)
    }

    pub fn get_security(&self, username: &str, account_type: i32, region_code: i32, event: &str, time: i64, url_type: i32) -> String {
        let url = if url_type == 1 {
            "getPrivacyQuestion"
        } else {
            "getSecurityQuestion"
        };
        format!(
            "{}/user/risk/{}?account={}&accountType={}&regionId={}&event={}&v={}",
            self.base_user_url, url, username, account_type, region_code, event, time
        )
    }

    pub fn next_security(&self, username: &str, account_type: i32, region_code: i32, event: &str, time: i64, url_type: i32) -> String {
        let url = if url_type == 1 {
            "nextPrivacyQuestion"
        } else {
            "nextSecurityQuestion"
        };
        format!(
            "{}/user/risk/{}?account={}&accountType={}&regionId={}&event={}&v={}",
            self.base_user_url, url, username, account_type, region_code, event, time
        )
    }

    pub fn check_security(&self) -> String {
        format!("{}/user/risk/checkAnswer", self.base_user_url)
    }

    pub fn logout(&self) -> String {
        format!("{}/user/v1/logout", self.base_userfintech_url)
    }

    pub fn news(&self, stock: &str, id: i64, items: i32) -> String {
        format!(
            "{}/information/news/tickerNews?tickerId={}&currentNewsId={}&pageSize={}",
            self.base_fintech_gw_url, stock, id, items
        )
    }

    pub fn option_quotes(&self) -> String {
        format!("{}/quote/option/query/list", self.base_options_gw_url)
    }

    pub fn options(&self, stock: &str) -> String {
        format!("{}/quote/option/{}/list", self.base_options_url, stock)
    }

    pub fn options_exp_date(&self, stock: &str) -> String {
        format!("{}/quote/option/{}/list", self.base_options_url, stock)
    }

    pub fn options_exp_date_new(&self) -> String {
        format!("{}/quote/option/strategy/list", self.base_fintech_gw_url)
    }

    pub fn options_bars(&self, derivative_id: &str) -> String {
        format!("{}/quote/option/chart/query?derivativeId={}", self.base_options_gw_url, derivative_id)
    }

    pub fn orders(&self, account_id: &str, page_size: i32) -> String {
        format!(
            "{}/trade/v2/option/list?secAccountId={}&startTime=1970-0-1&dateType=ORDER&pageSize={}&status=",
            self.base_ustradebroker_url, account_id, page_size
        )
    }

    pub fn history(&self, account_id: &str) -> String {
        format!("{}/trading/v1/webull/order/list?secAccountId={}", self.base_ustrade_url, account_id)
    }

    pub fn paper_orders(&self, paper_account_id: &str, page_size: i32) -> String {
        format!(
            "{}/paper/1/acc/{}/order?&startTime=1970-0-1&dateType=ORDER&pageSize={}&status=",
            self.base_paper_url, paper_account_id, page_size
        )
    }

    pub fn paper_account(&self, paper_account_id: &str) -> String {
        format!("{}/paper/1/acc/{}", self.base_paperfintech_url, paper_account_id)
    }

    pub fn paper_account_id(&self) -> String {
        format!("{}/myaccounts/true", self.base_paperfintech_url)
    }

    pub fn paper_cancel_order(&self, paper_account_id: &str, order_id: &str) -> String {
        format!("{}/paper/1/acc/{}/orderop/cancel/{}", self.base_paper_url, paper_account_id, order_id)
    }

    pub fn paper_modify_order(&self, paper_account_id: &str, order_id: &str) -> String {
        format!("{}/paper/1/acc/{}/orderop/modify/{}", self.base_paper_url, paper_account_id, order_id)
    }

    pub fn paper_place_order(&self, paper_account_id: &str, stock: &str) -> String {
        format!("{}/paper/1/acc/{}/orderop/place/{}", self.base_paper_url, paper_account_id, stock)
    }

    pub fn place_option_orders(&self, account_id: &str) -> String {
        format!("{}/trade/v2/option/placeOrder/{}", self.base_ustrade_url, account_id)
    }

    pub fn place_orders(&self, account_id: &str) -> String {
        format!("{}/trade/order/{}/placeStockOrder", self.base_ustrade_url, account_id)
    }

    pub fn modify_order(&self, account_id: &str, _order_id: &str) -> String {
        format!("{}/trading/v1/webull/order/stockOrderModify?secAccountId={}", self.base_ustrade_url, account_id)
    }

    pub fn quotes(&self, stock: &str) -> String {
        format!("{}/quotes/ticker/getTickerRealTime?tickerId={}&includeSecu=1&includeQuote=1", self.base_options_gw_url, stock)
    }

    pub fn rankings(&self) -> String {
        format!("{}/securities/market/v5/6/portal", self.base_securities_url)
    }

    pub fn refresh_login(&self, refresh_token: &str) -> String {
        format!("{}/passport/refreshToken?refreshToken={}", self.base_user_url, refresh_token)
    }

    pub fn remove_alert(&self) -> String {
        format!("{}/user/warning/v2/manage/overlap", self.base_userbroker_url)
    }

    pub fn replace_option_orders(&self, account_id: &str) -> String {
        format!("{}/v2/option/replaceOrder/{}", self.base_trade_url, account_id)
    }

    pub fn stock_detail(&self, stock: &str) -> String {
        format!("{}/stock/tickerRealTime/getQuote?tickerId={}&includeSecu=1&includeQuote=1&more=1", self.base_fintech_gw_url, stock)
    }

    pub fn stock_id(&self, stock: &str, region_code: i32) -> String {
        format!("{}/search/pc/tickers?keyword={}&pageIndex=1&pageSize=20&regionId={}", self.base_options_gw_url, stock, region_code)
    }

    pub fn trade_token(&self) -> String {
        format!("{}/trading/v1/global/trade/login", self.base_new_trade_url)
    }

    pub fn user(&self) -> String {
        format!("{}/user", self.base_user_url)
    }

    pub fn screener(&self) -> String {
        format!("{}/wlas/screener/ng/query", self.base_userbroker_url)
    }

    pub fn social_posts(&self, topic: &str, num: i32) -> String {
        format!("{}/social/feed/topic/{}/posts?size={}", self.base_user_url, topic, num)
    }

    pub fn social_home(&self, topic: &str, num: i32) -> String {
        format!("{}/social/feed/topic/{}/home?size={}", self.base_user_url, topic, num)
    }

    pub fn portfolio_lists(&self) -> String {
        format!("{}/personal/portfolio/v2/check", self.base_options_gw_url)
    }

    pub fn press_releases(&self, stock: &str, type_ids: Option<&str>, num: i32) -> String {
        let type_ids_string = type_ids.map(|t| format!("&typeIds={}", t)).unwrap_or_default();
        format!(
            "{}/securities/announcement/{}/list?lastAnnouncementId=0&limit={}{}&options=2",
            self.base_securitiesfintech_url, stock, num, type_ids_string
        )
    }

    pub fn calendar_events(&self, event: &str, region_code: i32, start_date: &str, page: i32, num: i32) -> String {
        format!(
            "{}/bgw/explore/calendar/{}?regionId={}&pageIndex={}&pageSize={}&startDate={}",
            self.base_fintech_gw_url, event, region_code, page, num, start_date
        )
    }

    pub fn get_all_tickers(&self, region_code: i32, user_region_code: i32) -> String {
        format!(
            "{}/securities/market/v5/card/stockActivityPc.advanced/list?regionId={}&userRegionId={}&hasNum=0&pageSize=9999",
            self.base_securitiesfintech_url, region_code, user_region_code
        )
    }
}