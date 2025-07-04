use crate::dto::{
    MemberResponse, NetworkIPAssignmentPool, NetworkResponse, NetworkRoute, NetworkV4AssignMode,
    StatusResponse,
};
use reqwest::{Client, Error, Response, Url, header};
use serde::Serialize;
use serde_json::json;
use std::collections::HashMap;

#[derive(Clone)]
pub struct APIClient {
    client: Client,
    base_url: Url,
}

#[derive(Serialize)]
pub struct EditMember {
    pub authorized: bool,
    #[serde(rename = "ipAssignments")]
    pub ip_assignments: Vec<String>,
    pub name: Option<String>,
}

#[derive(Serialize)]
pub struct EditNetwork {
    pub name: Option<String>,
    pub private: bool,
    #[serde(rename = "ipAssignmentPools")]
    pub ip_assignment_pools: Vec<NetworkIPAssignmentPool>,
    pub routes: Vec<NetworkRoute>,
    #[serde(rename = "v4AssignMode")]
    pub v4_assign_mode: NetworkV4AssignMode,
}

impl APIClient {
    pub fn new(token: &str, url: &str) -> Result<Self, Box<dyn std::error::Error>> {
        const PKG_NAME: &str = env!("CARGO_PKG_NAME");
        const PKG_VERSION: &str = env!("CARGO_PKG_VERSION");
        const PKG_REPO: &str = env!("CARGO_PKG_REPOSITORY");

        let mut headers = header::HeaderMap::new();
        headers.insert("X-ZT1-AUTH", header::HeaderValue::from_str(token).unwrap());

        let base_url = Url::parse(url)?;

        Ok(Self {
            client: Client::builder()
                .user_agent(format!("{}/{} (+{})", PKG_NAME, PKG_VERSION, PKG_REPO))
                .default_headers(headers)
                .build()
                .unwrap_or_default(),
            base_url,
        })
    }

    pub async fn status(&self) -> Result<StatusResponse, Error> {
        let url = self.base_url.join("status").unwrap();

        self.client
            .get(url)
            .send()
            .await?
            .json::<StatusResponse>()
            .await
    }

    pub async fn networks(&self) -> Result<Vec<String>, Error> {
        let url = self.base_url.join("controller/network").unwrap();

        self.client
            .get(url)
            .send()
            .await?
            .json::<Vec<String>>()
            .await
    }

    pub async fn network(&self, id: &str) -> Result<NetworkResponse, Error> {
        let url = self
            .base_url
            .join(&format!("controller/network/{}", id))
            .unwrap();

        self.client
            .get(url)
            .body(json!({}).to_string())
            .send()
            .await?
            .json::<NetworkResponse>()
            .await
    }

    pub async fn edit_network(&self, id: &str, data: EditNetwork) -> Result<Response, Error> {
        let url = self
            .base_url
            .join(&format!("controller/network/{}", id))
            .unwrap();

        self.client
            .post(url)
            .body(json!(&data).to_string())
            .send()
            .await
    }

    pub async fn delete_network(&self, id: &str) -> Result<Response, Error> {
        let url = self
            .base_url
            .join(&format!("controller/network/{}", id))
            .unwrap();

        self.client.delete(url).send().await
    }

    pub async fn create_network(&self, node_id: &str) -> Result<NetworkResponse, Error> {
        let url = self
            .base_url
            .join(&format!("controller/network/{}______", node_id))
            .unwrap();

        self.client
            .post(url)
            .body("{}")
            .send()
            .await?
            .json::<NetworkResponse>()
            .await
    }

    pub async fn members(&self, nwid: &str) -> Result<HashMap<String, u8>, Error> {
        let url = self
            .base_url
            .join(&format!("controller/network/{}/member", nwid))
            .unwrap();

        self.client
            .get(url)
            .send()
            .await?
            .json::<HashMap<String, u8>>()
            .await
    }

    pub async fn member(&self, nwid: &str, id: &str) -> Result<MemberResponse, Error> {
        let url = self
            .base_url
            .join(&format!("controller/network/{}/member/{}", nwid, id))
            .unwrap();
        self.client
            .get(url)
            .send()
            .await?
            .json::<MemberResponse>()
            .await
    }

    pub async fn edit_member(
        &self,
        nwid: &str,
        id: &str,
        data: EditMember,
    ) -> Result<Response, Error> {
        let url = self
            .base_url
            .join(&format!("controller/network/{}/member/{}", nwid, id))
            .unwrap();

        self.client
            .post(url)
            .body(json!(&data).to_string())
            .send()
            .await
    }

    pub async fn delete_member(&self, nwid: &str, id: &str) -> Result<Response, Error> {
        let url = self
            .base_url
            .join(&format!("controller/network/{}/member/{}", nwid, id))
            .unwrap();

        self.client.delete(url).send().await
    }
}
