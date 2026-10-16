use crate::{common::*, metrics::*};
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
    view! {{move||view!{<Failure error message=if detail.get().is_some(){t("自动更新暂时失败，页面保留上次成功数据。","Automatic update failed temporarily. The last successful data is retained.")}else{t("无法读取实例详情","Unable to load instance details")}/>}}<Failure error=action_error/><div class="xcss-content-stack host-details-layout"><nav class="host-device-navigation xcss-secondary-navigation" aria-label=t("设备信息分类","Device information categories")>{move||categories.get().into_iter().map(|(key,label)|view!{<button class="xcss-button" type="button" aria-pressed=move||(category.get()==key).to_string() aria-controls="host-device-content" on:click=move |_|category.set(key.into())>{label}</button>}).collect_view()}</nav><section class=move||format!("host-details-panel{}",if category.get()=="instance"||category.get()=="monitoring"{" xcss-content-panel host-details-panel--contained"}else{""}) aria-label=t("实例详情","Instance details")><section id="host-device-content" aria-label=move||categories.get().into_iter().find(|(id,_)|*id==category.get()).map(|(_,l)|l).unwrap_or_default()>{move||{match category.get().as_str(){"history"=>view!{<History response=history error=history_error hours revision=local/>}.into_any(),"instance"=>view!{<section class="host-overview">{overview.get_value()()}<section class="host-overview-section host-instance-actions" aria-label=t("实例操作","Instance actions")><button class="xcss-button" disabled=move||busy.get() on:click=move |_|deleting.set(true)>{t("删除实例","Delete instance")}</button></section></section>}.into_any(),"monitoring"=>view!{<Monitoring detail updated_at error/>}.into_any(),other=>view!{<DeviceCategory report=latest.get() category=other.to_string() pending=detail.get().is_none()/>}.into_any()}}}</section></section></div>{move||deleting.get().then(||view!{<Confirm title=t("删除监控实例","Delete monitoring instance") description=t("移除监控数据和绑定凭据。该客户端需要重新配对才能再次接入。","Remove monitoring data and bound credentials. The client must pair again to reconnect.") busy error=action_error close=Callback::new(move|()|deleting.set(false)) confirm=Callback::new(move|()|mutation(format!("/api/v1/monitoring/managed-instances/{}",id.get_value()),"DELETE",None,busy,action_error,move |_|navigate("instances")))/>})}}
}
#[component]
fn Monitoring(
    detail: RwSignal<Option<Value>>,
    updated_at: RwSignal<String>,
    error: RwSignal<Option<xcss_web::api::ApiError>>,
) -> impl IntoView {
    move || {
        let Some(d) = detail.get() else {
            return view!{<Show when=move||error.get().is_none()><div class="xcss-loading">{t("正在读取实例详情…","Loading instance details…")}</div></Show>}.into_any();
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
            view!{<div class="xcss-content-stack xsos-status"><h4>{text(host,"name")}</h4><dl class="host-detail-list">{vec![(t("状态","Status"),label(&text(host,"status"))),(t("系统","System"),format!("{} {} / {}",text(host,"os"),host["os_version"].as_str().unwrap_or_default(),text(host,"arch"))),(t("内核版本","Kernel version"),text(host,"kernel_version")),(t("客户端版本","Client version"),text(host,"client_version")),(t("最新 CPU","Latest CPU"),format_value("usage_percent",&host["cpu_usage_percent"])),(t("最新内存","Latest memory"),format_value("usage_percent",&host["memory_usage_percent"])),(t("数据采集时间","Data collected at"),report_time(&host["latest_collected_at"])),(t("服务端最近收到","Last received by server"),report_time(if latest.is_null(){&Value::Null}else{&host["last_seen_at"]})),(t("页面最近更新","Page last updated"),timestamp(&updated_at.get()))].into_iter().map(|(k,v)|view!{<dt>{k}</dt><dd>{v}</dd>}).collect_view()}</dl>{(!latest.is_null()).then(||view!{<Snapshot title=t("客户端状态","Client health") record=merge(&json!({"uptime_seconds":latest["system"]["uptime_seconds"]}),&latest["client"]) framed=false/>})}{(online&&lag>60_000f64.max(num(latest,"interval_seconds")*2000.)).then(||view!{<p role="status">{t("服务端仍在收到上报，但当前展示的指标采集时间明显较早；客户端可能正在补传或已调整时钟。","The server is still receiving reports, but the displayed metrics were collected much earlier. The client may be replaying queued reports or may have adjusted its clock.")}</p>})}{(online&&lag< -60_000.).then(||view!{<p role="status">{t("指标采集时间明显晚于服务端接收时间；客户端时钟可能偏快，后续指标可能暂时不更新。","Metric collection time is well ahead of server receipt time. The client clock may be fast, and subsequent metrics may not update until it catches up.")}</p>})}</div>}.into_any()
        }
    }
}
#[component]
fn DeviceCategory(report: Value, category: String, pending: bool) -> impl IntoView {
    if report.is_null() {
        return view!{<p class="host-device-card host-device-empty">{if pending{t("正在读取实例详情…","Loading instance details…")}else{t("等待首次上报","Waiting for first report")}}</p>}.into_any();
    }
    let sys = &report["system"];
    let h = &sys["hardware"];
    if category == "diagnostics" {
        return view!{<div class="host-device-grid">{rows(&report,"capabilities").into_iter().map(|c|view!{<article class="host-device-card"><h4>{label(&text(&c,"name"))}</h4><dl class="host-device-details"><dt>{t("状态","Status")}</dt><dd>{if flag(&c,"available"){t("可用","Available")}else{t("不可用","Unavailable")}}</dd><dt>{t("来源","Source")}</dt><dd>{text(&c,"source")}</dd><dt>{t("原因","Reason")}</dt><dd>{label(&text(&c,"error_kind"))}</dd><dt>{t("说明","Details")}</dt><dd>{text(&c,"message")}</dd></dl></article>}).collect_view()}</div>}.into_any();
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
        return view!{<p class="host-device-card host-device-empty">{match category.as_str(){"physical-networks"=>t("当前报告未提供可确认的物理网卡信息。","The current report does not provide verified physical network adapter information."),"sensors"=>t("未发现可读硬件传感器","No readable hardware sensors"),"disk-health"=>t("暂无 SMART 数据，请查看采集诊断","No SMART data yet; see collection diagnostics"),_=>t("当前报告未识别到此类设备，请查看采集诊断。","No devices of this type identified in the current report; see collection diagnostics.")}}</p>}.into_any();
    }
    view!{<div class="host-device-grid">{recs.into_iter().enumerate().map(|(i,r)|{let title=["locator","name","label","model","id"].iter().find_map(|k|r[*k].as_str()).unwrap_or(if category=="cpu"{"CPU"}else{""}).to_string();view!{<Snapshot title record=r show_title=!(simple&&i==0)/>}}).collect_view()}</div>{(category=="memory"&&rows(h,"memory_modules").is_empty()).then(||view!{<p class="host-device-empty">{t("当前报告未提供内存模块规格，请查看采集诊断。","No memory module specifications in the current report; see collection diagnostics.")}</p>})}}.into_any()
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
    view! {<div class="xcss-content-stack host-history-content"><div class="host-history-toolbar"><p>{move||response.get().map(|r|format!("{} {}{}{}",num(&r,"step_seconds"),t("秒/点，来源：","seconds per point, source: "),text(&r,"source"),t("；横轴按采样时间，缺失区间不连线。","; the horizontal axis uses sample time and missing intervals are not connected."))).unwrap_or_else(||if error.get().is_none(){t("正在读取历史趋势…","Loading history trends…")}else{String::new()})}</p><div class="xcss-actions">{[(0.25,"15m"),(1.,"1h"),(6.,"6h"),(24.,"24h"),(168.,"7d"),(720.,"30d")].into_iter().map(|(h,l)|view!{<button class="xcss-button" aria-pressed=move||(hours.get()==h).to_string() on:click=move |_|hours.set(h)>{l}</button>}).collect_view()}</div><Failure error retry=Callback::new(move|()|revision.update(|v|*v+=1)) message=t("无法读取所选范围的历史趋势。","Unable to load history trends for the selected range.")/></div><div class="host-history-grid">{charts.into_iter().map(|(title,unit,lines)|view!{<Chart title unit lines response error/>}).collect_view()}</div></div>}
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
    view! {<section class="xcss-content-panel host-history-card" aria-label=title.get_value()><h3>{title.get_value()}</h3><div class="host-chart-legend">{lines.get_value().into_iter().map(|(_,l,color)|view!{<span><i aria-hidden="true" style:background=color/>{l}</span>}).collect_view()}</div>{move||{let Some(r)=response.get()else{return view!{<p>{if error.get().is_some(){t("不可用","Unavailable")}else{t("正在读取…","Loading…")}}</p>}.into_any()};let points=rows(&r,"points");let lines=lines.get_value();let values:Vec<f64>=points.iter().flat_map(|p|lines.iter().filter_map(|(k,_,_)|p[*k]["avg"].as_f64())).collect();let Some((floor,ceiling))=chart_domain(&values,unit) else{return view!{<p>{t("所选范围没有历史样本","No historical samples in this range")}</p>}.into_any()};let start=ms(&text(&r,"requested_from"));let end=ms(&text(&r,"requested_to"));let segments=lines.iter().flat_map(|(k,_,color)|chart_segments(&points,k,start,end,num(&r,"step_seconds")*1000.,floor,ceiling).into_iter().map(move|seg|(seg,*color))).collect::<Vec<_>>();view!{<svg viewBox="0 0 100 100" role="img" aria-label=format!("{} {}",title.get_value(),t("历史图","history chart")) preserveAspectRatio="none">{[25,50,75].into_iter().map(|y|view!{<line x1="0" x2="100" y1=y y2=y class="host-chart-gridline" vector-effect="non-scaling-stroke"/>}).collect_view()}{segments.into_iter().map(|(seg,color)|if seg.len()==1{view!{<circle cx=seg[0].0 cy=seg[0].1 r="0.8" fill=color/>}.into_any()}else{view!{<polyline points={seg.iter().map(|(x,y)|format!("{x},{y}")).collect::<Vec<_>>().join(" ")} fill="none" stroke=color vector-effect="non-scaling-stroke"/>}.into_any()}).collect_view()}</svg><p class="host-chart-scale">{if unit=="bytes"{format!("{} 0–{}/{}",t("纵轴","Vertical axis"),bytes(ceiling),t("秒","s"))}else{format!("{} {floor}–{ceiling} {}",t("纵轴","Vertical axis"),if unit=="percent"{"%"}else{unit})}}</p>}.into_any()}}</section>}
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
