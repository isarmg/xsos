//! UI projections of the monitored-instance contract. Full report validation remains authoritative.
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, Deserialize)]
pub struct Instance {
    pub request_id: String,
    pub instance_id: String,
    pub display_name: String,
    pub status: String,
    pub authorization_code: String,
}
#[derive(Clone, Debug, Deserialize)]
pub struct HostSummary {
    pub id: String,
    pub os: String,
    pub arch: String,
    pub status: String,
    pub data_error: Option<String>,
}
#[derive(Clone, Debug, Deserialize)]
pub struct InstancesPage {
    pub instances: Vec<Instance>,
    pub hosts: Vec<HostSummary>,
    pub next_cursor: Option<String>,
    pub previous_cursor: Option<String>,
}
impl crate::common::PageCursors for InstancesPage {
    fn previous_cursor(&self) -> Option<String> {
        self.previous_cursor.clone()
    }
    fn next_cursor(&self) -> Option<String> {
        self.next_cursor.clone()
    }
}
#[derive(Clone, Copy, Debug, Deserialize)]
pub struct HostCount {
    pub total: u64,
    pub online: u64,
}
#[derive(Clone, Debug, Deserialize)]
pub struct HostStatistics {
    pub total: HostCount,
    pub windows: HostCount,
    pub linux: HostCount,
    pub macos: HostCount,
}
#[derive(Clone, Debug, Deserialize)]
pub struct HostsPage {
    pub statistics: HostStatistics,
}
#[derive(Clone, Debug, Deserialize)]
pub struct PairingRequest {
    pub request_id: String,
    pub os: String,
    pub arch: String,
    pub status: String,
    pub expires_at: String,
}
#[derive(Serialize)]
pub struct InstanceName {
    pub display_name: String,
}
#[derive(Serialize)]
pub struct ActivatePairing {
    pub request_id: String,
    pub activation_code: String,
}
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use xcss_web::api::decode_validated_response;
    #[test]
    fn focused_instance_page_keeps_binding_and_codes() {
        let id = "018f1f4b-7a5d-7b5f-8d31-123456789abc";
        let path = format!("/api/v1/monitoring/client-instances?instance_id={id}");
        let mut value = json!({"instances":[{"request_id":id,"instance_id":id,"display_name":"A","status":"pending","created_at":"2032-12-31T00:00:00Z","authorization_code":"a".repeat(36)}],"hosts":[],"next_cursor":null,"previous_cursor":null});
        let decode = |v| {
            decode_validated_response::<InstancesPage>(v, |v| crate::contracts::validate(&path, v))
        };
        assert_eq!(
            decode(value.clone()).unwrap().instances[0].authorization_code,
            "a".repeat(36)
        );
        value["next_cursor"] = json!("bmV4dA");
        assert!(decode(value).is_err());
    }
    #[test]
    fn statistics_keep_integer_counts_and_semantic_guard() {
        let mut v = json!({"hosts":[],"statistics":{"total":{"total":9007199254740991u64,"online":1},"windows":{"total":0,"online":0},"linux":{"total":0,"online":0},"macos":{"total":0,"online":0}},"next_cursor":null,"previous_cursor":null});
        let decode = |v| {
            decode_validated_response::<HostsPage>(v, |v| {
                crate::contracts::validate("/api/v1/monitoring/hosts", v)
            })
        };
        assert_eq!(
            decode(v.clone()).unwrap().statistics.total.total,
            9007199254740991
        );
        v["statistics"]["windows"]["online"] = json!(1);
        assert!(decode(v).is_err());
    }
}
