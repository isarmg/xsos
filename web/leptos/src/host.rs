use crate::{common::*, metrics::*};
use leptos::component::{component_props_builder, component_view};
use leptos::prelude::*;
use serde_json::{Value, json};
fn ms(v: &str) -> f64 {
    js_sys::Date::new(&wasm_bindgen::JsValue::from_str(v)).get_time()
}
#[component]
pub fn HostWorkspace(
    host_id: String,
    children: ChildrenFn,
    revision: RwSignal<u64>,
) -> impl IntoView {
    let id = StoredValue::new(host_id.clone());
    let path = format!("/api/v1/monitoring/hosts/{host_id}");
    let (detail, error) = resource(Signal::derive(move || path.clone()), revision, 2000);
    let updated_at = RwSignal::new(String::new());
    Effect::new(move |_| {
        if detail.get().is_some() {
            updated_at.set(
                js_sys::Date::new_0()
                    .to_iso_string()
                    .as_string()
                    .unwrap_or_default(),
            );
        }
    });
    let hours = RwSignal::new(1f64);
    let local = RwSignal::new(0);
    let history_path = Signal::derive(move || {
        format!(
            "/api/v1/monitoring/hosts/{}/history?window_seconds={}&resolution=auto&max_points=720",
            id.get_value(),
            (hours.get() * 3600.) as u64
        )
    });
    let (history, history_error) = resource_with_interval(
        history_path,
        local,
        Signal::derive(move || if hours.get() <= 1. { 5_000 } else { 30_000 }),
    );
    Effect::new(move |_| {
        revision.get();
        local.update(|v| *v += 1);
    });
    let category = RwSignal::new("history".to_string());
    let busy = RwSignal::new(false);
    let action_error = RwSignal::new(None);
    let deleting = RwSignal::new(false);
    let latest = Signal::derive(move || {
        detail
            .get()
            .map(|v| v["latest"].clone())
            .unwrap_or(Value::Null)
    });
    let categories = Signal::derive(move || {
        let report = latest.get();
        let mut cats = vec![
            ("history", t("历史趋势", "History trends")),
            ("instance", t("实例概览", "Instance overview")),
            ("cpu", "CPU".into()),
            ("memory", t("内存", "RAM")),
        ];
        if !report.is_null() {
            cats.extend(vec![
                ("physical-networks", t("网络硬件", "Network hardware")),
                ("thunderbolt", t("雷电 / USB4", "Thunderbolt / USB4")),
                ("monitor", t("显示器", "Monitors")),
                ("bluetooth", t("蓝牙", "Bluetooth")),
                ("usb_controller", t("USB 控制器", "USB controllers")),
                ("usb_device", t("USB 设备", "USB devices")),
                ("audio", t("音频设备", "Audio devices")),
            ]);
            if !report["system"]["hardware"].is_null() {
                cats.push(("sensors", t("硬件传感器", "Hardware sensors")));
                cats.push(("disk-health", t("磁盘健康", "Disk health")));
            }
            cats.extend(vec![
                ("networks", t("网络接口", "Network interfaces")),
                ("disks", t("磁盘", "Disks")),
                ("temperatures", t("温度传感器", "Temperature sensors")),
                ("gpus", t("显卡", "GPUs")),
                (
                    "diagnostics",
                    t("采集能力与诊断", "Collection capabilities and diagnostics"),
                ),
            ]);
        }
        cats.push(("monitoring", t("监控状态", "Monitoring status")));
        cats
    });
    Effect::new(move |_| {
        if !categories
            .get()
            .iter()
            .any(|(key, _)| *key == category.get())
        {
            category.set("history".to_string());
        }
    });
    let overview = StoredValue::new(children);
    View::new((
        IntoRender::into_render({
            move || {
                View::new({
                    #[allow(unreachable_code)]
                    #[allow(unused_mut)]
                    #[allow(clippy::let_and_return, clippy::let_unit_value, clippy::unit_arg)]
                    component_view(
                        #[allow(clippy::needless_borrows_for_generic_args)]
                        &Failure,
                        {
                            let mut props = component_props_builder(&Failure)
                .error(#[allow(unused_braces)] { error })
                .message(
                    #[allow(unused_braces)]
                    {
                        if detail.get().is_some() {
                            t(
                                "自动更新暂时失败，页面保留上次成功数据。",
                                "Automatic update failed temporarily. The last successful data is retained.",
                            )
                        } else {
                            t(
                                "无法读取实例详情",
                                "Unable to load instance details",
                            )
                        }
                    },
                )
                .build();
                            props
                        },
                    )
                })
            }
        }),
        {
            #[allow(unreachable_code)]
            #[allow(unused_mut)]
            #[allow(clippy::let_and_return, clippy::let_unit_value, clippy::unit_arg)]
            component_view(
                #[allow(clippy::needless_borrows_for_generic_args)]
                &Failure,
                {
                    let mut props = component_props_builder(&Failure)
                        .error(
                            #[allow(unused_braces)]
                            {
                                action_error
                            },
                        )
                        .build();
                    props
                },
            )
        },
        leptos::html::div()
            .child((
                leptos::html::nav()
                    .child(
                        #[allow(unused_braces)]
                        {
                            IntoRender::into_render({
                                move || {
                                    categories
                                        .get()
                                        .into_iter()
                                        .map(|(key, label)| {
                                            View::new(
    leptos::html::button()
        .child(
            #[allow(unused_braces)]
            { IntoRender::into_render({ label }) },
        )
        .class("xcss-button")
        .r#type("button")
        .aria_pressed(
            ::leptos::prelude::IntoAttributeValue::into_attribute_value(move || {
                (category.get() == key).to_string()
            }),
        )
        .aria_controls("host-device-content")
        .on(leptos::ev::click, move |_| category.set(key.into())),
)
                                        })
                                        .collect_view()
                                }
                            })
                        },
                    )
                    .class("host-device-navigation xcss-secondary-navigation")
                    .aria_label(::leptos::prelude::IntoAttributeValue::into_attribute_value(
                        t("设备信息分类", "Device information categories"),
                    )),
                leptos::html::section()
                    .child(
                        #[allow(unused_braces)]
                        {
                            leptos::html::section()
                                .child(
                                    #[allow(unused_braces)]
                                    {
                                        IntoRender::into_render({
                                            move || {
                                                match category.get().as_str() {
                                                "history" => {
                                                    View::new({
    #[allow(unreachable_code)] #[allow(unused_mut)] #[allow(clippy::let_and_return, clippy::let_unit_value, clippy::unit_arg)]
    component_view(
        #[allow(clippy::needless_borrows_for_generic_args)]
        &History,
        {
            let mut props = component_props_builder(&History)
                .response(#[allow(unused_braces)] { history })
                .error(#[allow(unused_braces)] { history_error })
                .hours(#[allow(unused_braces)] { hours })
                .revision(#[allow(unused_braces)] { local })
                .build();
            props
        },
    )
})
                                                        .into_any()
                                                }
                                                "instance" => {
                                                    View::new(
    leptos::html::section()
        .child((
            IntoRender::into_render({ overview.get_value()() }),
            leptos::html::section()
                .child(
                    #[allow(unused_braces)]
                    {
                        leptos::html::button()
                            .child(
                                #[allow(unused_braces)]
                                {
                                    IntoRender::into_render({
                                        t("删除实例", "Delete instance")
                                    })
                                },
                            )
                            .class("xcss-button")
                            .disabled(
                                ::leptos::prelude::IntoAttributeValue::into_attribute_value(move ||
                                busy.get()),
                            )
                            .on(
                                leptos::ev::click,
                                move |_| deleting.set(true),
                            )
                    },
                )
                .class("host-overview-section host-instance-actions")
                .aria_label(
                    ::leptos::prelude::IntoAttributeValue::into_attribute_value(
                        t("实例操作", "Instance actions"),
                    ),
                ),
        ))
        .class("host-overview"),
)
                                                        .into_any()
                                                }
                                                "monitoring" => {
                                                    View::new({
    #[allow(unreachable_code)] #[allow(unused_mut)] #[allow(clippy::let_and_return, clippy::let_unit_value, clippy::unit_arg)]
    component_view(
        #[allow(clippy::needless_borrows_for_generic_args)]
        &Monitoring,
        {
            let mut props = component_props_builder(&Monitoring)
                .detail(#[allow(unused_braces)] { detail })
                .updated_at(#[allow(unused_braces)] { updated_at })
                .error(#[allow(unused_braces)] { error })
                .build();
            props
        },
    )
})
                                                        .into_any()
                                                }
                                                other => {
                                                    View::new({
    #[allow(unreachable_code)] #[allow(unused_mut)] #[allow(clippy::let_and_return, clippy::let_unit_value, clippy::unit_arg)]
    component_view(
        #[allow(clippy::needless_borrows_for_generic_args)]
        &DeviceCategory,
        {
            let mut props = component_props_builder(
                    &DeviceCategory,
                )
                .report(#[allow(unused_braces)] { latest.get() })
                .category(#[allow(unused_braces)] { other.to_string() })
                .pending(#[allow(unused_braces)] { detail.get().is_none() })
                .build();
            props
        },
    )
})
                                                        .into_any()
                                                }
                                            }
                                            }
                                        })
                                    },
                                )
                                .id("host-device-content")
                                .aria_label(
                                    ::leptos::prelude::IntoAttributeValue::into_attribute_value(
                                        move || {
                                            categories
                                                .get()
                                                .into_iter()
                                                .find(|(id, _)| *id == category.get())
                                                .map(|(_, l)| l)
                                                .unwrap_or_default()
                                        },
                                    ),
                                )
                        },
                    )
                    .class(move || {
                        format!(
                            "host-details-panel{}",
                            if category.get() == "instance" || category.get() == "monitoring" {
                                " xcss-content-panel host-details-panel--contained"
                            } else {
                                ""
                            }
                        )
                    })
                    .aria_label(::leptos::prelude::IntoAttributeValue::into_attribute_value(
                        t("实例详情", "Instance details"),
                    )),
            ))
            .class("xcss-content-stack host-details-layout"),
        IntoRender::into_render({
            move || {
                deleting
                .get()
                .then(|| {
                    View::new({
    #[allow(unreachable_code)] #[allow(unused_mut)] #[allow(clippy::let_and_return, clippy::let_unit_value, clippy::unit_arg)]
    component_view(
        #[allow(clippy::needless_borrows_for_generic_args)]
        &Confirm,
        {
            let mut props = component_props_builder(&Confirm)
                .title(
                    #[allow(unused_braces)]
                    { t("删除监控实例", "Delete monitoring instance") },
                )
                .description(
                    #[allow(unused_braces)]
                    {
                        t(
                            "移除监控数据和绑定凭据。该客户端需要重新配对才能再次接入。",
                            "Remove monitoring data and bound credentials. The client must pair again to reconnect.",
                        )
                    },
                )
                .busy(#[allow(unused_braces)] { busy })
                .error(#[allow(unused_braces)] { action_error })
                .close(
                    #[allow(unused_braces)]
                    { Callback::new(move |()| deleting.set(false)) },
                )
                .confirm(
                    #[allow(unused_braces)]
                    {
                        Callback::new(move |()| mutation(
                            format!(
                                "/api/v1/monitoring/managed-instances/{}", id.get_value()
                            ),
                            "DELETE",
                            None,
                            busy,
                            action_error,
                            move |_| navigate("instances"),
                        ))
                    },
                )
                .build();
            props
        },
    )
})
                })
            }
        }),
    ))
}
#[component]
fn Monitoring(
    detail: RwSignal<Option<Value>>,
    updated_at: RwSignal<String>,
    error: RwSignal<Option<xcss_web::api::ApiError>>,
) -> impl IntoView {
    move || {
        let Some(d) = detail.get() else {
            return View::new({
                #[allow(unreachable_code)]
                #[allow(unused_mut)]
                #[allow(clippy::let_and_return, clippy::let_unit_value, clippy::unit_arg)]
                component_view(
                    #[allow(clippy::needless_borrows_for_generic_args)]
                    &Show,
                    {
                        let mut props = component_props_builder(&Show)
                            .when(
                                #[allow(unused_braces)]
                                {
                                    move || error.get().is_none()
                                },
                            )
                            .children({
                                ::leptos::children::ToChildren::to_children(move || {
                                    leptos::html::div()
                                        .child(
                                            #[allow(unused_braces)]
                                            {
                                                IntoRender::into_render({
                                                    t(
                                                        "正在读取实例详情…",
                                                        "Loading instance details…",
                                                    )
                                                })
                                            },
                                        )
                                        .class("xcss-loading")
                                })
                            })
                            .build();
                        props
                    },
                )
            })
            .into_any();
        };
        let report_time = |value: &Value| {
            value
                .as_str()
                .map(timestamp)
                .unwrap_or_else(|| t("尚未上报", "Not yet reported"))
        };
        {
            let host = &d["host"];
            let latest = &d["latest"];
            let lag = ms(&text(host, "last_seen_at")) - ms(&text(latest, "collected_at"));
            let online = text(host, "status") == "online" && !latest.is_null();
            View::new(
    leptos::html::div()
        .child((
            leptos::html::h4()
                .child(
                    #[allow(unused_braces)]
                    {
                        IntoRender::into_render({
                            text(host, "name")
                        })
                    },
                ),
            leptos::html::dl()
                .child(
                    #[allow(unused_braces)]
                    {
                        IntoRender::into_render({
                            vec![
                                (t("状态", "Status"), label(& text(host, "status"))),
                                (t("系统", "System"), format!("{} {} / {}", text(host,
                                "os"), host["os_version"].as_str().unwrap_or_default(),
                                text(host, "arch"))), (t("内核版本", "Kernel version"),
                                text(host, "kernel_version")), (t("客户端版本",
                                "Client version"), text(host, "client_version")),
                                (t("最新 CPU", "Latest CPU"),
                                format_value("usage_percent",& host["cpu_usage_percent"])),
                                (t("最新内存", "Latest memory"),
                                format_value("usage_percent",&
                                host["memory_usage_percent"])), (t("数据采集时间",
                                "Data collected at"), report_time(&
                                host["latest_collected_at"])), (t("服务端最近收到",
                                "Last received by server"), report_time(if latest.is_null()
                                { & Value::Null } else { & host["last_seen_at"] })),
                                (t("页面最近更新", "Page last updated"), timestamp(&
                                updated_at.get()))
                            ]
                                .into_iter()
                                .map(|(k, v)| {
                                    View::new((
    leptos::html::dt()
        .child(
            #[allow(unused_braces)]
            { IntoRender::into_render({ k }) },
        ),
    leptos::html::dd()
        .child(
            #[allow(unused_braces)]
            { IntoRender::into_render({ v }) },
        ),
))
                                })
                                .collect_view()
                        })
                    },
                )
                .class("host-detail-list"),
            IntoRender::into_render({
                (!latest.is_null())
                    .then(|| {
                        View::new({
    #[allow(unreachable_code)] #[allow(unused_mut)] #[allow(clippy::let_and_return, clippy::let_unit_value, clippy::unit_arg)]
    component_view(
        #[allow(clippy::needless_borrows_for_generic_args)]
        &Snapshot,
        {
            let mut props = component_props_builder(&Snapshot)
                .title(
                    #[allow(unused_braces)]
                    { t("客户端状态", "Client health") },
                )
                .record(
                    #[allow(unused_braces)]
                    {
                        merge(
                            &json!(
                                { "uptime_seconds" : latest["system"] ["uptime_seconds"] }
                            ),
                            &latest["client"],
                        )
                    },
                )
                .framed(#[allow(unused_braces)] { false })
                .build();
            props
        },
    )
})
                    })
            }),
            IntoRender::into_render({
                (online
                    && lag > 60_000f64.max(num(latest, "interval_seconds") * 2000.))
                    .then(|| {
                        View::new(
    leptos::html::p()
        .child(
            #[allow(unused_braces)]
            {
                IntoRender::into_render({
                    t(
                        "服务端仍在收到上报，但当前展示的指标采集时间明显较早；客户端可能正在补传或已调整时钟。",
                        "The server is still receiving reports, but the displayed metrics were collected much earlier. The client may be replaying queued reports or may have adjusted its clock.",
                    )
                })
            },
        )
        .role("status"),
)
                    })
            }),
            IntoRender::into_render({
                (online && lag < -60_000.)
                    .then(|| {
                        View::new(
    leptos::html::p()
        .child(
            #[allow(unused_braces)]
            {
                IntoRender::into_render({
                    t(
                        "指标采集时间明显晚于服务端接收时间；客户端时钟可能偏快，后续指标可能暂时不更新。",
                        "Metric collection time is well ahead of server receipt time. The client clock may be fast, and subsequent metrics may not update until it catches up.",
                    )
                })
            },
        )
        .role("status"),
)
                    })
            }),
        ))
        .class("xcss-content-stack xsos-status"),
).into_any()
        }
    }
}
#[component]
fn DeviceCategory(report: Value, category: String, pending: bool) -> impl IntoView {
    if report.is_null() {
        return View::new(
            leptos::html::p()
                .child(
                    #[allow(unused_braces)]
                    {
                        IntoRender::into_render({
                            if pending {
                                t("正在读取实例详情…", "Loading instance details…")
                            } else {
                                t("等待首次上报", "Waiting for first report")
                            }
                        })
                    },
                )
                .class("host-device-card host-device-empty"),
        )
        .into_any();
    }
    let sys = &report["system"];
    let h = &sys["hardware"];
    if category == "diagnostics" {
        return View::new(
            leptos::html::div()
                .child(
                    #[allow(unused_braces)]
                    {
                        IntoRender::into_render({
                            rows(&report, "capabilities")
                                .into_iter()
                                .map(|c| {
                                    View::new(
                                        leptos::html::article()
                                            .child((
                                                leptos::html::h4().child(
                                                    #[allow(unused_braces)]
                                                    {
                                                        IntoRender::into_render({
                                                            label(&text(&c, "name"))
                                                        })
                                                    },
                                                ),
                                                leptos::html::dl()
                                                    .child((
                                                        leptos::html::dt().child(
                                                            #[allow(unused_braces)]
                                                            {
                                                                IntoRender::into_render({
                                                                    t("状态", "Status")
                                                                })
                                                            },
                                                        ),
                                                        leptos::html::dd().child(
                                                            #[allow(unused_braces)]
                                                            {
                                                                IntoRender::into_render({
                                                                    if flag(&c, "available") {
                                                                        t("可用", "Available")
                                                                    } else {
                                                                        t("不可用", "Unavailable")
                                                                    }
                                                                })
                                                            },
                                                        ),
                                                        leptos::html::dt().child(
                                                            #[allow(unused_braces)]
                                                            {
                                                                IntoRender::into_render({
                                                                    t("来源", "Source")
                                                                })
                                                            },
                                                        ),
                                                        leptos::html::dd().child(
                                                            #[allow(unused_braces)]
                                                            {
                                                                IntoRender::into_render({
                                                                    text(&c, "source")
                                                                })
                                                            },
                                                        ),
                                                        leptos::html::dt().child(
                                                            #[allow(unused_braces)]
                                                            {
                                                                IntoRender::into_render({
                                                                    t("原因", "Reason")
                                                                })
                                                            },
                                                        ),
                                                        leptos::html::dd().child(
                                                            #[allow(unused_braces)]
                                                            {
                                                                IntoRender::into_render({
                                                                    label(&text(&c, "error_kind"))
                                                                })
                                                            },
                                                        ),
                                                        leptos::html::dt().child(
                                                            #[allow(unused_braces)]
                                                            {
                                                                IntoRender::into_render({
                                                                    t("说明", "Details")
                                                                })
                                                            },
                                                        ),
                                                        leptos::html::dd().child(
                                                            #[allow(unused_braces)]
                                                            {
                                                                IntoRender::into_render({
                                                                    text(&c, "message")
                                                                })
                                                            },
                                                        ),
                                                    ))
                                                    .class("host-device-details"),
                                            ))
                                            .class("host-device-card"),
                                    )
                                })
                                .collect_view()
                        })
                    },
                )
                .class("host-device-grid"),
        )
        .into_any();
    }
    let recs: Vec<Value> = match category.as_str() {
        "cpu" => vec![with_time(
            merge(&sys["cpu"], &h["cpu"]),
            "hardware_collected_at",
            &h["collected_at"],
        )],
        "memory" => {
            let mut r = vec![sys["memory"].clone()];
            r.extend(
                rows(h, "memory_modules")
                    .into_iter()
                    .map(|v| with_time(v, "collected_at", &h["inventory_collected_at"])),
            );
            r
        }
        "physical-networks" => rows(h, "physical_networks")
            .into_iter()
            .map(|v| with_time(v, "hardware_collected_at", &h["collected_at"]))
            .collect(),
        "networks" => network_records(
            rows(sys, "networks"),
            rows(h, "networks")
                .into_iter()
                .map(|v| with_time(v, "hardware_collected_at", &h["collected_at"]))
                .collect(),
        ),
        "disks" => rows(sys, "disks").into_iter().map(disk_record).collect(),
        "temperatures" => rows(sys, "temperatures"),
        "gpus" => rows(sys, "gpus"),
        "sensors" => rows(h, "sensors")
            .into_iter()
            .map(|v| {
                let kind = text(&v, "kind");
                let mut record = json!({"id":v["id"],"label":v["label"],"source":v["source"]});
                record[kind] = v["value"].clone();
                record
            })
            .collect(),
        "disk-health" => rows(h, "disk_health"),
        kind => rows(h, "devices")
            .into_iter()
            .filter(|v| text(v, "kind") == kind)
            .map(|mut v| {
                v.as_object_mut().map(|o| o.remove("kind"));
                with_time(v, "collected_at", &h["inventory_collected_at"])
            })
            .collect(),
    };
    let simple = category == "cpu" || category == "memory";
    if recs.is_empty() {
        return View::new(
    leptos::html::p()
        .child(
            #[allow(unused_braces)]
            {
                IntoRender::into_render({
                    match category.as_str() {
                        "physical-networks" => {
                            t(
                                "当前报告未提供可确认的物理网卡信息。",
                                "The current report does not provide verified physical network adapter information.",
                            )
                        }
                        "sensors" => {
                            t(
                                "未发现可读硬件传感器",
                                "No readable hardware sensors",
                            )
                        }
                        "disk-health" => {
                            t(
                                "暂无 SMART 数据，请查看采集诊断",
                                "No SMART data yet; see collection diagnostics",
                            )
                        }
                        _ => {
                            t(
                                "当前报告未识别到此类设备，请查看采集诊断。",
                                "No devices of this type identified in the current report; see collection diagnostics.",
                            )
                        }
                    }
                })
            },
        )
        .class("host-device-card host-device-empty"),
).into_any();
    }
    View::new((
    leptos::html::div()
        .child(
            #[allow(unused_braces)]
            {
                IntoRender::into_render({
                    recs.into_iter()
                        .enumerate()
                        .map(|(i, r)| {
                            let title = ["locator", "name", "label", "model", "id"]
                                .iter()
                                .find_map(|k| r[*k].as_str())
                                .unwrap_or(if category == "cpu" { "CPU" } else { "" })
                                .to_string();
                            View::new({
    #[allow(unreachable_code)] #[allow(unused_mut)] #[allow(clippy::let_and_return, clippy::let_unit_value, clippy::unit_arg)]
    component_view(
        #[allow(clippy::needless_borrows_for_generic_args)]
        &Snapshot,
        {
            let mut props = component_props_builder(&Snapshot)
                .title(#[allow(unused_braces)] { title })
                .record(#[allow(unused_braces)] { r })
                .show_title(#[allow(unused_braces)] { !(simple && i == 0) })
                .build();
            props
        },
    )
})
                        })
                        .collect_view()
                })
            },
        )
        .class("host-device-grid"),
    IntoRender::into_render({
        (category == "memory" && rows(h, "memory_modules").is_empty())
            .then(|| {
                View::new(
    leptos::html::p()
        .child(
            #[allow(unused_braces)]
            {
                IntoRender::into_render({
                    t(
                        "当前报告未提供内存模块规格，请查看采集诊断。",
                        "No memory module specifications in the current report; see collection diagnostics.",
                    )
                })
            },
        )
        .class("host-device-empty"),
)
            })
    }),
)).into_any()
}
#[component]
fn History(
    response: RwSignal<Option<Value>>,
    error: RwSignal<Option<xcss_web::api::ApiError>>,
    hours: RwSignal<f64>,
    revision: RwSignal<u64>,
) -> impl IntoView {
    let charts = vec![
        (
            "CPU".to_string(),
            "percent",
            vec![("cpu_usage_percent", t("使用率", "Usage"), "#2878b5")],
        ),
        (
            "GPU".to_string(),
            "percent",
            vec![
                ("gpu_utilization_percent", t("使用率", "Usage"), "#6f58a8"),
                (
                    "gpu_memory_usage_percent",
                    t("显存使用率", "Memory usage"),
                    "#bd6eaa",
                ),
            ],
        ),
        (
            t("磁盘 I/O", "Disk I/O"),
            "bytes",
            vec![
                ("disk_read_bytes_per_second", t("读取", "Read"), "#2c8c74"),
                (
                    "disk_written_bytes_per_second",
                    t("写入", "Write"),
                    "#d28a37",
                ),
            ],
        ),
        (
            t("CPU 频率", "CPU frequency"),
            "MHz",
            vec![(
                "cpu_frequency_mhz",
                t("平均频率", "Mean frequency"),
                "#2878b5",
            )],
        ),
        (
            t("GPU 功耗", "GPU power"),
            "W",
            vec![(
                "gpu_power_watts",
                t("最大设备功耗", "Maximum device power"),
                "#6f58a8",
            )],
        ),
        (
            t("GPU 核心频率", "GPU core clock"),
            "MHz",
            vec![(
                "gpu_core_clock_mhz",
                t("最大设备频率", "Maximum device frequency"),
                "#6f58a8",
            )],
        ),
        (
            t("风扇转速", "Fan speed"),
            "RPM",
            vec![("max_fan_rpm", t("最大转速", "Maximum speed"), "#2c8c74")],
        ),
        (
            t("磁盘温度", "Disk temperature"),
            "℃",
            vec![(
                "max_disk_temperature_celsius",
                t("最高温度", "Maximum temperature"),
                "#d28a37",
            )],
        ),
        (
            t("NVMe 寿命消耗", "NVMe endurance used"),
            "percent",
            vec![(
                "max_disk_percentage_used",
                t(
                    "最大已用比例（可超过 100%）",
                    "Maximum used (may exceed 100%)",
                ),
                "#d28a37",
            )],
        ),
        (
            "RAM".to_string(),
            "percent",
            vec![("memory_usage_percent", t("使用率", "Usage"), "#7a5aa6")],
        ),
    ];
    View::new(
    leptos::html::div()
        .child((
            leptos::html::div()
                .child((
                    leptos::html::p()
                        .child(
                            #[allow(unused_braces)]
                            {
                                IntoRender::into_render({
                                    move || {
                                        response
                                            .get()
                                            .map(|r| {
                                                format!(
                                                    "{} {}{}{}", num(& r, "step_seconds"),
                                                    t("秒/点，来源：", "seconds per point, source: "),
                                                    text(& r, "source"),
                                                    t("；横轴按采样时间，缺失区间不连线。",
                                                    "; the horizontal axis uses sample time and missing intervals are not connected.")
                                                )
                                            })
                                            .unwrap_or_else(|| {
                                                if error.get().is_none() {
                                                    t(
                                                        "正在读取历史趋势…",
                                                        "Loading history trends…",
                                                    )
                                                } else {
                                                    String::new()
                                                }
                                            })
                                    }
                                })
                            },
                        ),
                    leptos::html::div()
                        .child(
                            #[allow(unused_braces)]
                            {
                                IntoRender::into_render({
                                    [
                                        (0.25, "15m"),
                                        (1., "1h"),
                                        (6., "6h"),
                                        (24., "24h"),
                                        (168., "7d"),
                                        (720., "30d"),
                                    ]
                                        .into_iter()
                                        .map(|(h, l)| {
                                            View::new(
    leptos::html::button()
        .child(
            #[allow(unused_braces)]
            { IntoRender::into_render({ l }) },
        )
        .class("xcss-button")
        .aria_pressed(
            ::leptos::prelude::IntoAttributeValue::into_attribute_value(move || {
                (hours.get() == h).to_string()
            }),
        )
        .on(leptos::ev::click, move |_| hours.set(h)),
)
                                        })
                                        .collect_view()
                                })
                            },
                        )
                        .class("xcss-actions"),
                    {
                        #[allow(unreachable_code)] #[allow(unused_mut)]
                        #[allow(clippy::let_and_return, clippy::let_unit_value, clippy::unit_arg)]
                        component_view(
                            #[allow(clippy::needless_borrows_for_generic_args)]
                            &Failure,
                            {
                                let mut props = component_props_builder(
                                        &Failure,
                                    )
                                    .error(#[allow(unused_braces)] { error })
                                    .retry(
                                        #[allow(unused_braces)]
                                        { Callback::new(move |()| revision.update(|v| *v += 1)) },
                                    )
                                    .message(
                                        #[allow(unused_braces)]
                                        {
                                            t(
                                                "无法读取所选范围的历史趋势。",
                                                "Unable to load history trends for the selected range.",
                                            )
                                        },
                                    )
                                    .build();
                                props
                            },
                        )
                    },
                ))
                .class("host-history-toolbar"),
            leptos::html::div()
                .child(
                    #[allow(unused_braces)]
                    {
                        IntoRender::into_render({
                            charts
                                .into_iter()
                                .map(|(title, unit, lines)| {
                                    View::new({
    #[allow(unreachable_code)] #[allow(unused_mut)] #[allow(clippy::let_and_return, clippy::let_unit_value, clippy::unit_arg)]
    component_view(
        #[allow(clippy::needless_borrows_for_generic_args)]
        &Chart,
        {
            let mut props = component_props_builder(&Chart)
                .title(#[allow(unused_braces)] { title })
                .unit(#[allow(unused_braces)] { unit })
                .lines(#[allow(unused_braces)] { lines })
                .response(#[allow(unused_braces)] { response })
                .error(#[allow(unused_braces)] { error })
                .build();
            props
        },
    )
})
                                })
                                .collect_view()
                        })
                    },
                )
                .class("host-history-grid"),
        ))
        .class("xcss-content-stack host-history-content"),
)
}
pub fn chart_domain(values: &[f64], unit: &str) -> Option<(f64, f64)> {
    if values.is_empty() {
        return None;
    }
    let ceiling = values
        .iter()
        .copied()
        .fold(if unit == "percent" { 100. } else { 1. }, f64::max);
    let floor = if unit == "℃" {
        values.iter().copied().fold(0., f64::min)
    } else {
        0.
    };
    Some((floor, ceiling))
}
fn chart_y(value: f64, floor: f64, ceiling: f64) -> f64 {
    100. - ((value - floor) * 100. / (ceiling - floor)).clamp(0., 100.)
}
fn chart_segments(
    points: &[Value],
    key: &str,
    start: f64,
    end: f64,
    step: f64,
    floor: f64,
    ceiling: f64,
) -> Vec<Vec<(f64, f64)>> {
    let mut out = vec![];
    let mut segment = vec![];
    let mut previous: Option<f64> = None;
    for p in points {
        let at = ms(&text(p, "start"));
        let value = p[key]["avg"].as_f64();
        if !segment.is_empty() && (value.is_none() || previous.is_none_or(|t| at > t + step)) {
            out.push(std::mem::take(&mut segment));
        }
        if let Some(v) = value {
            let x = if end > start {
                ((at - start) * 100. / (end - start)).clamp(0., 100.)
            } else {
                0.
            };
            let y = chart_y(v, floor, ceiling);
            segment.push((x, y));
        }
        previous = Some(at);
    }
    if !segment.is_empty() {
        out.push(segment);
    }
    out
}
#[component]
fn Chart(
    title: String,
    unit: &'static str,
    lines: Vec<(&'static str, String, &'static str)>,
    response: RwSignal<Option<Value>>,
    error: RwSignal<Option<xcss_web::api::ApiError>>,
) -> impl IntoView {
    let title = StoredValue::new(title);
    let lines = StoredValue::new(lines);
    View::new(
        leptos::html::section()
            .child((
                leptos::html::h3().child(
                    #[allow(unused_braces)]
                    {
                        IntoRender::into_render({ title.get_value() })
                    },
                ),
                leptos::html::div()
                    .child(
                        #[allow(unused_braces)]
                        {
                            IntoRender::into_render({
                                lines
                                    .get_value()
                                    .into_iter()
                                    .map(|(_, l, color)| {
                                        View::new(
                                            leptos::html::span().child((
                                                leptos::html::i()
                                                    .aria_hidden("true")
                                                    .style(("background", color)),
                                                IntoRender::into_render({ l }),
                                            )),
                                        )
                                    })
                                    .collect_view()
                            })
                        },
                    )
                    .class("host-chart-legend"),
                IntoRender::into_render({
                    move || {
                        let Some(r) = response.get() else {
                            return View::new(leptos::html::p().child(
                                #[allow(unused_braces)]
                                {
                                    IntoRender::into_render({
                                        if error.get().is_some() {
                                            t("不可用", "Unavailable")
                                        } else {
                                            t("正在读取…", "Loading…")
                                        }
                                    })
                                },
                            ))
                            .into_any();
                        };
                        let points = rows(&r, "points");
                        let lines = lines.get_value();
                        let values: Vec<f64> = points
                            .iter()
                            .flat_map(|p| {
                                lines.iter().filter_map(|(k, _, _)| p[*k]["avg"].as_f64())
                            })
                            .collect();
                        let Some((floor, ceiling)) = chart_domain(&values, unit) else {
                            return View::new(leptos::html::p().child(
                                #[allow(unused_braces)]
                                {
                                    IntoRender::into_render({
                                        t(
                                            "所选范围没有历史样本",
                                            "No historical samples in this range",
                                        )
                                    })
                                },
                            ))
                            .into_any();
                        };
                        let start = ms(&text(&r, "requested_from"));
                        let end = ms(&text(&r, "requested_to"));
                        let segments = lines
                            .iter()
                            .flat_map(|(k, _, color)| {
                                chart_segments(
                                    &points,
                                    k,
                                    start,
                                    end,
                                    num(&r, "step_seconds") * 1000.,
                                    floor,
                                    ceiling,
                                )
                                .into_iter()
                                .map(move |seg| (seg, *color))
                            })
                            .collect::<Vec<_>>();
                        View::new((
                            leptos::svg::svg()
                                .child((
                                    IntoRender::into_render({
                                        [25, 50, 75]
                                            .into_iter()
                                            .map(|y| {
                                                View::new(
    leptos::svg::line()
        .class("host-chart-gridline")
        .attr("x1", "0")
        .attr("x2", "100")
        .attr("y1", ::leptos::prelude::IntoAttributeValue::into_attribute_value(y))
        .attr("y2", ::leptos::prelude::IntoAttributeValue::into_attribute_value(y))
        .attr("vector-effect", "non-scaling-stroke"),
)
                                            })
                                            .collect_view()
                                    }),
                                    IntoRender::into_render({
                                        segments
                                            .into_iter()
                                            .map(|(seg, color)| {
                                                if seg.len() == 1 {
                                                    View::new(
    leptos::svg::circle()
        .attr(
            "cx",
            ::leptos::prelude::IntoAttributeValue::into_attribute_value(seg[0].0),
        )
        .attr(
            "cy",
            ::leptos::prelude::IntoAttributeValue::into_attribute_value(seg[0].1),
        )
        .attr("r", "0.8")
        .attr(
            "fill",
            ::leptos::prelude::IntoAttributeValue::into_attribute_value(color),
        ),
)
                                .into_any()
                                                } else {
                                                    View::new(
    leptos::svg::polyline()
        .attr(
            "points",
            ::leptos::prelude::IntoAttributeValue::into_attribute_value({
                seg.iter()
                    .map(|(x, y)| format!("{x},{y}"))
                    .collect::<Vec<_>>()
                    .join(" ")
            }),
        )
        .attr("fill", "none")
        .attr(
            "stroke",
            ::leptos::prelude::IntoAttributeValue::into_attribute_value(color),
        )
        .attr("vector-effect", "non-scaling-stroke"),
)
                                .into_any()
                                                }
                                            })
                                            .collect_view()
                                    }),
                                ))
                                .attr("viewBox", "0 0 100 100")
                                .attr("role", "img")
                                .attr(
                                    "aria-label",
                                    ::leptos::prelude::IntoAttributeValue::into_attribute_value(
                                        format!(
                                            "{} {}",
                                            title.get_value(),
                                            t("历史图", "history chart")
                                        ),
                                    ),
                                )
                                .attr("preserveAspectRatio", "none"),
                            leptos::html::p()
                                .child(
                                    #[allow(unused_braces)]
                                    {
                                        IntoRender::into_render({
                                            if unit == "bytes" {
                                                format!(
                                                    "{} 0–{}/{}",
                                                    t("纵轴", "Vertical axis"),
                                                    bytes(ceiling),
                                                    t("秒", "s")
                                                )
                                            } else {
                                                format!(
                                                    "{} {floor}–{ceiling} {}",
                                                    t("纵轴", "Vertical axis"),
                                                    if unit == "percent" { "%" } else { unit }
                                                )
                                            }
                                        })
                                    },
                                )
                                .class("host-chart-scale"),
                        ))
                        .into_any()
                    }
                }),
            ))
            .class("xcss-content-panel host-history-card")
            .aria_label(::leptos::prelude::IntoAttributeValue::into_attribute_value(
                title.get_value(),
            )),
    )
}

#[cfg(test)]
mod chart_tests {
    use super::*;
    #[test]
    fn scales_follow_drawn_means_without_inventing_samples_or_peaks() {
        assert_eq!(chart_domain(&[], "bytes"), None);
        for (mean, ceiling, y) in [(0., 1., 100.), (0.25, 1., 75.), (1024., 1024., 0.)] {
            assert_eq!(chart_domain(&[mean], "bytes"), Some((0., ceiling)));
            assert_eq!(chart_y(mean, 0., ceiling), y);
        }
        assert_eq!(chart_domain(&[150.], "percent"), Some((0., 150.)));
        assert_eq!(chart_domain(&[-10.], "℃"), Some((-10., 1.)));
        assert_eq!(chart_y(-10., -10., 1.), 100.);
    }
}
