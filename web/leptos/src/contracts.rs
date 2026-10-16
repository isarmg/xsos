use serde_json::Value;
const METRICS: &[&str] = &[
    "cpu_usage_percent",
    "memory_usage_percent",
    "network_received_bytes_per_second",
    "network_transmitted_bytes_per_second",
    "disk_read_bytes_per_second",
    "disk_written_bytes_per_second",
    "max_temperature_celsius",
    "gpu_utilization_percent",
    "gpu_memory_usage_percent",
    "cpu_frequency_mhz",
    "gpu_power_watts",
    "gpu_core_clock_mhz",
    "max_fan_rpm",
    "max_disk_temperature_celsius",
    "max_disk_percentage_used",
];
fn fields(v: &Value, keys: &[&str], test: fn(&Value) -> bool) -> bool {
    v.is_object() && keys.iter().all(|k| v.get(k).is_some_and(test))
}
fn nullable_string(v: &Value) -> bool {
    v.is_null() || v.is_string()
}
fn number(v: &Value) -> bool {
    v.is_null() || v.as_f64().is_some_and(f64::is_finite)
}
fn integer(v: &Value) -> bool {
    v.as_u64().is_some_and(|v| v <= 9_007_199_254_740_991)
}
fn exact(v: &Value, keys: &[&str]) -> bool {
    v.as_object()
        .is_some_and(|o| o.len() == keys.len() && keys.iter().all(|k| o.contains_key(*k)))
}
fn cursor(v: &Value) -> bool {
    v.is_null()
        || v.as_str().is_some_and(|s| {
            (1..=512).contains(&s.len())
                && s.bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b"_-".contains(&b))
        })
}
fn cursors(v: &Value) -> bool {
    fields(v, &["previous_cursor", "next_cursor"], cursor)
}
fn query<'a>(path: &'a str, key: &str) -> Option<&'a str> {
    path.split_once('?')?
        .1
        .split('&')
        .filter_map(|part| part.split_once('='))
        .find_map(|(k, v)| (k == key).then_some(v))
}
fn member(v: &Value, options: &[&str]) -> bool {
    v.as_str().is_some_and(|s| options.contains(&s))
}
pub fn uuid(v: &Value) -> bool {
    v.as_str().is_some_and(|s| {
        let b = s.as_bytes();
        b.len() == 36
            && [8, 13, 18, 23].iter().all(|i| b[*i] == b'-')
            && b.iter().enumerate().all(|(i, v)| {
                [8, 13, 18, 23].contains(&i) || v.is_ascii_hexdigit() && !v.is_ascii_uppercase()
            })
            && (b'1'..=b'8').contains(&b[14])
            && b"89ab".contains(&b[19])
    })
}
fn utc(v: &Value) -> bool {
    v.as_str().is_some_and(|s| {
        if !s.is_ascii() || s.len() < 20 || !s.ends_with('Z') {
            return false;
        }
        let b = s.as_bytes();
        if b[10] != b'T' || b[13] != b':' || b[16] != b':' {
            return false;
        }
        if !crate::common::date_validate(&crate::common::date_parts(&s[..10]))
            .is_ok_and(|v| v == s[..10])
        {
            return false;
        }
        let number = |a, b, max| {
            s[a..b].bytes().all(|v| v.is_ascii_digit())
                && s[a..b].parse::<u32>().is_ok_and(|v| v < max)
        };
        number(11, 13, 24)
            && number(14, 16, 60)
            && number(17, 19, 60)
            && (s.len() == 20
                || b[19] == b'.'
                    && s.len() > 21
                    && s[20..s.len() - 1].bytes().all(|v| v.is_ascii_digit()))
    })
}
fn code(v: &Value) -> bool {
    v.as_str().is_some_and(|s| {
        s.len() == 36
            && s.bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit())
    })
}
fn date(v: &Value) -> bool {
    v.as_str().is_some_and(|s| {
        crate::common::date_validate(&crate::common::date_parts(s)).is_ok_and(|v| v == s)
    })
}
fn capability(v: &Value) -> bool {
    exact(v, &["name", "source", "available", "message", "error_kind"])
        && fields(v, &["name", "source"], |v| {
            v.as_str().is_some_and(|s| !s.is_empty())
        })
        && v["available"].is_boolean()
        && nullable_string(&v["message"])
        && (v["error_kind"].is_null()
            || member(
                &v["error_kind"],
                &[
                    "unsupported",
                    "not_present",
                    "driver_missing",
                    "permission_denied",
                    "transient",
                    "invalid_data",
                ],
            ))
}
fn host(v: &Value) -> bool {
    exact(
        v,
        &[
            "id",
            "name",
            "os",
            "os_version",
            "kernel_version",
            "arch",
            "client_version",
            "registered_at",
            "last_seen_at",
            "latest_collected_at",
            "status",
            "capabilities",
            "cpu_usage_percent",
            "memory_usage_percent",
            "network_received_bytes_per_second",
            "network_transmitted_bytes_per_second",
            "disk_read_bytes_per_second",
            "disk_written_bytes_per_second",
            "max_temperature_celsius",
            "gpu_utilization_percent",
            "gpu_memory_usage_percent",
            "cpu_frequency_mhz",
            "gpu_power_watts",
            "gpu_core_clock_mhz",
            "max_fan_rpm",
            "max_disk_temperature_celsius",
            "max_disk_percentage_used",
            "data_error",
        ],
    ) && uuid(&v["id"])
        && fields(
            v,
            &["name", "os", "arch", "client_version"],
            Value::is_string,
        )
        && fields(v, &["os_version", "kernel_version"], nullable_string)
        && utc(&v["registered_at"])
        && utc(&v["last_seen_at"])
        && (v["latest_collected_at"].is_null() || utc(&v["latest_collected_at"]))
        && member(&v["status"], &["online", "stale", "offline", "unavailable"])
        && (v["data_error"].is_null() || v["data_error"] == "stored_host_data_invalid")
        && v["capabilities"]
            .as_array()
            .is_some_and(|a| a.iter().all(capability))
        && fields(v, METRICS, number)
}
fn object_rows(v: &Value) -> bool {
    v.as_array().is_some_and(|a| a.iter().all(Value::is_object))
}
fn report(v: &Value) -> bool {
    let system = &v["system"];
    let hardware = if let Some(h) = system.get("hardware") {
        exact(
            h,
            &[
                "collected_at",
                "cpu",
                "networks",
                "physical_networks",
                "sensors",
                "disk_health",
                "inventory_collected_at",
                "memory_modules",
                "devices",
            ],
        ) && utc(&h["collected_at"])
            && (h["inventory_collected_at"].is_null() || utc(&h["inventory_collected_at"]))
            && h["cpu"].is_object()
            && fields(
                h,
                &[
                    "networks",
                    "physical_networks",
                    "sensors",
                    "disk_health",
                    "memory_modules",
                    "devices",
                ],
                object_rows,
            )
    } else {
        true
    };
    let system_keys = if system.get("hardware").is_some() {
        vec![
            "uptime_seconds",
            "cpu",
            "memory",
            "networks",
            "disks",
            "temperatures",
            "gpus",
            "hardware",
        ]
    } else {
        vec![
            "uptime_seconds",
            "cpu",
            "memory",
            "networks",
            "disks",
            "temperatures",
            "gpus",
        ]
    };
    exact(
        v,
        &[
            "schema_version",
            "report_id",
            "collected_at",
            "host",
            "interval_seconds",
            "system",
            "capabilities",
            "client",
        ],
    ) && v["schema_version"] == 1
        && uuid(&v["report_id"])
        && utc(&v["collected_at"])
        && v["interval_seconds"].as_f64().is_some_and(f64::is_finite)
        && exact(
            &v["host"],
            &[
                "id",
                "os",
                "os_version",
                "kernel_version",
                "arch",
                "client_version",
            ],
        )
        && uuid(&v["host"]["id"])
        && fields(&v["host"], &["os", "arch", "client_version"], |v| {
            v.as_str().is_some_and(|s| !s.is_empty())
        })
        && fields(
            &v["host"],
            &["os_version", "kernel_version"],
            nullable_string,
        )
        && exact(system, &system_keys)
        && hardware
        && system["cpu"].is_object()
        && system["memory"].is_object()
        && fields(
            system,
            &["networks", "disks", "temperatures", "gpus"],
            object_rows,
        )
        && crate::metrics::parse_u64(&system["uptime_seconds"]).is_some()
        && v["capabilities"]
            .as_array()
            .is_some_and(|a| a.iter().all(capability))
        && exact(&v["client"], &["spool_pending_batches", "collector_errors"])
        && fields(
            &v["client"],
            &["spool_pending_batches", "collector_errors"],
            |v| crate::metrics::parse_u64(v).is_some(),
        )
}
fn instance(v: &Value) -> bool {
    exact(
        v,
        &[
            "request_id",
            "instance_id",
            "display_name",
            "status",
            "created_at",
            "authorization_code",
        ],
    ) && instance_fields(v)
}
fn instance_fields(v: &Value) -> bool {
    fields(v, &["request_id", "instance_id"], uuid)
        && v["display_name"].is_string()
        && member(&v["status"], &["pending", "active", "cancelled"])
        && utc(&v["created_at"])
        && code(&v["authorization_code"])
}
fn unique(a: &[Value], key: &str) -> bool {
    let mut ids = std::collections::HashSet::new();
    a.iter().all(|v| ids.insert(v[key].as_str()))
}
pub fn validate(path: &str, v: &Value) -> bool {
    if v.is_null() {
        return false;
    }
    let base = path.split('?').next().unwrap_or(path);
    if base.ends_with("/logs/calendar") {
        return exact(v, &["today"]) && date(&v["today"]);
    }
    if base.ends_with("/reports") {
        let fields = if v.get("end_date").is_some() {
            vec![
                "host_id",
                "date",
                "end_date",
                "reports",
                "next_cursor",
                "previous_cursor",
            ]
        } else {
            vec![
                "host_id",
                "date",
                "reports",
                "next_cursor",
                "previous_cursor",
            ]
        };
        let expected_host = base
            .strip_suffix("/reports")
            .and_then(|p| p.rsplit('/').next());
        return exact(v, &fields)
            && uuid(&v["host_id"])
            && expected_host.is_none_or(|id| v["host_id"] == id)
            && query(path, "date")
                .or_else(|| query(path, "start_date"))
                .is_none_or(|d| v["date"] == d)
            && query(path, "end_date").is_none_or(|d| v["end_date"] == d)
            && date(&v["date"])
            && v.get("end_date").is_none_or(date)
            && cursors(v)
            && v["reports"].as_array().is_some_and(|a| {
                a.len() <= 50
                    && unique(a, "report_id")
                    && a.iter().all(|r| {
                        exact(r, &["report_id", "collected_at", "received_at"])
                            && uuid(&r["report_id"])
                            && utc(&r["collected_at"])
                            && utc(&r["received_at"])
                    })
            });
    }
    if base.ends_with("/history") {
        return exact(
            v,
            &[
                "host_id",
                "requested_from",
                "requested_to",
                "actual_from",
                "actual_to",
                "step_seconds",
                "source",
                "points",
            ],
        ) && uuid(&v["host_id"])
            && base
                .strip_suffix("/history")
                .and_then(|p| p.rsplit('/').next())
                .is_none_or(|id| v["host_id"] == id)
            && fields(
                v,
                &["requested_from", "requested_to", "actual_from", "actual_to"],
                utc,
            )
            && v["step_seconds"]
                .as_u64()
                .is_some_and(|v| v > 0 && v <= 9_007_199_254_740_991)
            && member(&v["source"], &["raw", "hourly", "mixed"])
            && v["points"].as_array().is_some_and(|a| {
                a.len() <= 1000
                    && a.iter().all(|p| {
                        exact(
                            p,
                            &[&["start", "end", "sample_count"][..], METRICS].concat(),
                        ) && fields(p, &["start", "end"], utc)
                            && p["sample_count"]
                                .as_u64()
                                .is_some_and(|v| v > 0 && v <= 9_007_199_254_740_991)
                            && METRICS.iter().all(|k| {
                                exact(&p[*k], &["min", "max", "avg", "count"])
                                    && fields(&p[*k], &["min", "max", "avg"], number)
                                    && integer(&p[*k]["count"])
                            })
                    })
            });
    }
    if base.ends_with("/hosts") {
        return exact(
            v,
            &["hosts", "statistics", "next_cursor", "previous_cursor"],
        ) && cursors(v)
            && v["hosts"]
                .as_array()
                .is_some_and(|a| a.len() <= 50 && unique(a, "id") && a.iter().all(host))
            && exact(&v["statistics"], &["total", "windows", "linux", "macos"])
            && ["total", "windows", "linux", "macos"].iter().all(|k| {
                exact(&v["statistics"][*k], &["total", "online"])
                    && fields(&v["statistics"][*k], &["total", "online"], integer)
                    && v["statistics"][*k]["online"].as_u64()
                        <= v["statistics"][*k]["total"].as_u64()
            });
    }
    if base.contains("/hosts/") {
        return exact(v, &["host", "latest"])
            && host(&v["host"])
            && base
                .rsplit('/')
                .next()
                .is_none_or(|id| v["host"]["id"] == id)
            && (v["latest"].is_null()
                || report(&v["latest"]) && v["latest"]["host"]["id"] == v["host"]["id"]);
    }
    if base.ends_with("/client-instances") {
        if v.get("instances").is_some() {
            let Some(a) = v["instances"].as_array() else {
                return false;
            };
            let Some(h) = v["hosts"].as_array() else {
                return false;
            };
            let focused = query(path, "instance_id");
            return exact(v, &["instances", "hosts", "next_cursor", "previous_cursor"])
                && cursors(v)
                && focused.is_none_or(|id| {
                    a.len() <= 1
                        && v["next_cursor"].is_null()
                        && v["previous_cursor"].is_null()
                        && a.iter()
                            .all(|i| i["request_id"] == id || i["instance_id"] == id)
                })
                && a.len() <= 50
                && unique(a, "request_id")
                && a.iter().all(instance)
                && h.len() <= a.len()
                && unique(h, "id")
                && h.iter()
                    .all(|h| host(h) && a.iter().any(|i| i["instance_id"] == h["id"]));
        }
        return exact(
            v,
            &[
                "request_id",
                "instance_id",
                "display_name",
                "status",
                "created_at",
                "authorization_code",
                "activation_code",
            ],
        ) && instance_fields(v)
            && v["status"] == "pending"
            && code(&v["activation_code"]);
    }
    if base.contains("/pairing-requests/") {
        return exact(
            v,
            &[
                "request_id",
                "os",
                "arch",
                "client_version",
                "status",
                "expires_at",
            ],
        ) && uuid(&v["request_id"])
            && v["request_id"].as_str() == base.rsplit('/').next()
            && fields(v, &["os", "arch", "client_version"], Value::is_string)
            && member(&v["status"], &["waiting", "active", "denied", "expired"])
            && utc(&v["expires_at"]);
    }
    if base.ends_with("/activate-admin") {
        return exact(v, &["instance_id", "status"])
            && uuid(&v["instance_id"])
            && v["status"] == "active";
    }
    instance(v)
}
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn pairing_response_is_bound_to_requested_device() {
        let path = "/api/v1/xsoc/pairing-requests/018f1f4b-7a5d-7b5f-8d31-123456789abc";
        let mut response = json!({"request_id":"018f1f4b-7a5d-7b5f-8d31-123456789abc","os":"linux","arch":"x86_64","client_version":"0.8.1","status":"waiting","expires_at":"2099-01-01T00:00:00Z"});
        assert!(validate(path, &response));
        response["request_id"] = json!("018f1f4b-7a5d-7b5f-8d31-123456789abd");
        assert!(!validate(path, &response));
    }
    #[test]
    fn uuid_contract() {
        assert!(uuid(&json!("018f1f4b-7a5d-7b5f-8d31-123456789abc")));
        assert!(!uuid(&json!("018f1f4b-7a5d-0b5f-8d31-123456789abc")));
    }
    #[test]
    fn calendar_validation() {
        assert!(validate(
            "/monitoring/logs/calendar",
            &json!({"today":"2032-12-31"})
        ));
        assert!(!validate(
            "/monitoring/logs/calendar",
            &json!({"today":"2032-02-30"})
        ));
    }
    #[test]
    fn real_monitoring_fixture_preserves_full_hardware_contract() {
        let value: Value =
            serde_json::from_str(include_str!("../tests/fixtures/host-detail.json")).unwrap();
        let path = format!(
            "/api/v1/monitoring/hosts/{}",
            value["host"]["id"].as_str().unwrap()
        );
        assert!(validate(&path, &value));
        for field in ["inventory_collected_at", "memory_modules", "devices"] {
            let mut missing = value.clone();
            missing["latest"]["system"]["hardware"]
                .as_object_mut()
                .unwrap()
                .remove(field);
            assert!(!validate(&path, &missing), "missing {field}");
        }
        let mut malformed = value.clone();
        malformed["latest"]["system"]["hardware"]["devices"] = Value::Null;
        assert!(!validate(&path, &malformed));
        let mut wrong_host = value.clone();
        wrong_host["latest"]["host"]["id"] = json!("018f1f4b-7a5d-7b5f-8d31-123456789abc");
        assert!(!validate(&path, &wrong_host));
        assert!(!validate(
            "/api/v1/monitoring/hosts/018f1f4b-7a5d-7b5f-8d31-123456789abc",
            &value
        ));
    }
    #[test]
    fn invalid_envelopes_and_malformed_utf8_dates_fail_closed() {
        assert!(!validate("/api/v1/monitoring/hosts", &Value::Null));
        assert!(!validate(
            "/api/v1/monitoring/logs/calendar",
            &json!({"today":"2032-12-31", "unknown":true})
        ));
        for value in [
            "éééééééééééé",
            "2024-13-01T00:00:00Z",
            "2024-02-30T00:00:00Z",
            "2024-01-01T24:00:00Z",
        ] {
            assert!(!utc(&json!(value)));
        }
        let value = json!({"host_id":"018f1f4b-7a5d-7b5f-8d31-123456789abc", "date":"2032-12-31", "reports":[], "next_cursor":null, "previous_cursor":null});
        assert!(validate(
            "/api/v1/monitoring/hosts/018f1f4b-7a5d-7b5f-8d31-123456789abc/reports?date=2032-12-31",
            &value
        ));
        assert!(!validate(
            "/api/v1/monitoring/hosts/018f1f4b-7a5d-7b5f-8d31-123456789abc/reports?date=2032-12-30",
            &value
        ));
        let mut value = value;
        value["next_cursor"] = json!("../foreign");
        assert!(!validate(
            "/api/v1/monitoring/hosts/018f1f4b-7a5d-7b5f-8d31-123456789abc/reports?date=2032-12-31",
            &value
        ));
    }
    #[test]
    fn bounded_pages_reject_duplicate_rows_and_nonopaque_cursors() {
        let identity = |n| format!("018f1f4b-7a5d-7b5f-8d31-{n:012}");
        let report = |n| json!({"report_id":identity(n),"collected_at":"2032-12-31T00:00:00Z","received_at":"2032-12-31T01:00:00Z"});
        let path = format!(
            "/api/v1/monitoring/hosts/{}/reports?date=2032-12-31",
            identity(100)
        );
        let page = json!({"host_id":identity(100),"date":"2032-12-31","next_cursor":"bmV4dA","previous_cursor":null,"reports":(0..50).map(report).collect::<Vec<_>>()});
        assert!(validate(&path, &page));
        let mut too_many = page.clone();
        too_many["reports"].as_array_mut().unwrap().push(report(50));
        assert!(!validate(&path, &too_many));
        let mut duplicate = page.clone();
        duplicate["reports"] = json!([report(1), report(1)]);
        assert!(!validate(&path, &duplicate));
        for cursor in ["A".repeat(513), "é".into(), "not/a/cursor".into()] {
            let mut invalid = page.clone();
            invalid["next_cursor"] = json!(cursor);
            assert!(!validate(&path, &invalid));
        }
        let mut missing = page.clone();
        missing.as_object_mut().unwrap().remove("previous_cursor");
        assert!(!validate(&path, &missing));
        let mut extra = page;
        extra["unknown"] = json!(true);
        assert!(!validate(&path, &extra));
    }
    #[test]
    fn focused_instances_bind_both_request_and_host_id_without_list_cursors() {
        let identity = |n| format!("018f1f4b-7a5d-7b5f-8d31-{n:012}");
        let item = |n| json!({"request_id":identity(n),"instance_id":identity(n+100),"display_name":format!("Host-{n}"),"status":"active","created_at":"2032-12-31T00:00:00Z","authorization_code":"a".repeat(36)});
        let base = "/api/v1/monitoring/client-instances";
        let mut page = json!({"instances":(0..50).map(item).collect::<Vec<_>>(),"hosts":[],"next_cursor":null,"previous_cursor":null});
        assert!(validate(base, &page));
        page["instances"].as_array_mut().unwrap().push(item(50));
        assert!(!validate(base, &page));
        page["instances"] = json!([item(1), item(1)]);
        assert!(!validate(base, &page));
        page["instances"] = json!([item(0)]);
        for n in [0, 100] {
            assert!(validate(
                &format!("{base}?instance_id={}", identity(n)),
                &page
            ));
        }
        assert!(!validate(
            &format!("{base}?instance_id={}", identity(1)),
            &page
        ));
        page["next_cursor"] = json!("bmV4dA");
        assert!(!validate(
            &format!("{base}?instance_id={}", identity(0)),
            &page
        ));
        page["next_cursor"] = Value::Null;
        page["instances"] = json!([item(0), item(1)]);
        assert!(!validate(
            &format!("{base}?instance_id={}", identity(0)),
            &page
        ));
    }
}
