use crate::common::*;
use leptos::prelude::*;
use serde_json::Value;
pub fn metric_label(key: &str) -> String {
    match key {
        "hardware_collected_at" => t("硬件采样时间", "Hardware sampled at"),
        "module_version" => t("模块 / 固件版本", "Module / firmware version"),
        "locator" => t("插槽 / 位置", "Slot / location"),
        "memory_type" => t("内存代际", "Memory generation"),
        "form_factor" => t("封装类型", "Form factor"),
        "capacity_bytes" => t("模块容量", "Module capacity"),
        "speed_mt_s" => t("标称传输速率", "Rated transfer rate"),
        "configured_speed_mt_s" => t("配置传输速率", "Configured transfer rate"),
        "reported_speed" => t("系统报告速率", "Reported rate"),
        "vendor_id" => t("厂商 ID", "Vendor ID"),
        "product_id" => t("产品 ID", "Product ID"),
        "revision" => t("设备 / 固件修订", "Device / firmware revision"),
        "version" => t("协议版本", "Protocol version"),
        "bus" => t("总线 / 传输方式", "Bus / transport"),
        "driver" => t("驱动", "Driver"),
        "connection" => t("连接接口", "Connection"),
        "speed_mbps" => t("设备链路速率", "Device link speed"),
        "model" => t("型号", "Model"),
        "frequency_mhz" => t("平均频率", "Mean frequency"),
        "max_frequency_mhz" => t("硬件最高频率", "Maximum hardware frequency"),
        "per_core_frequency_mhz" => t("各核心频率", "Per-core frequencies"),
        "load_average" => t("负载（1/5/15 分钟）", "Load (1/5/15 minutes)"),
        "mac_address" => t("MAC 地址", "MAC address"),
        "ip_addresses" => t("IP 地址", "IP addresses"),
        "interface_name" => t("接口名称", "Interface name"),
        "mtu" => t("MTU", "MTU"),
        "link_speed_mbps" => t("链路速率", "Link speed"),
        "operational_state" => t("链路状态", "Link state"),
        "fan_rpm" => t("风扇转速", "Fan speed"),
        "voltage_volts" => t("电压", "Voltage"),
        "current_amps" => t("电流", "Current"),
        "energy_joules" => t("累计能量", "Energy"),
        "device" => t("物理设备", "Physical device"),
        "serial_number" => t("序列号", "Serial number"),
        "protocol" => t("接口协议", "Protocol"),
        "collected_at" => t("采集时间", "Collected at"),
        "healthy" => t("SMART 健康检查通过", "SMART health passed"),
        "percentage_used" => t("NVMe 寿命已消耗", "NVMe endurance used"),
        "available_spare_percent" => t("可用备用空间", "Available spare"),
        "critical_warning" => t("NVMe 严重警告位掩码", "NVMe critical warning bits"),
        "power_on_hours" => t("通电小时", "Power-on hours"),
        "power_cycles" => t("通电次数", "Power cycles"),
        "unsafe_shutdowns" => t("非安全关机", "Unsafe shutdowns"),
        "media_errors" => t("介质错误", "Media errors"),
        "bytes_read" => t("累计读取", "Total read"),
        "bytes_written" => t("累计写入", "Total written"),
        "usage_percent" => t("使用率", "Usage"),
        "logical_count" => t("逻辑核心", "Logical cores"),
        "physical_count" => t("物理核心", "Physical cores"),
        "per_core_percent" => t("各核心使用率", "Per-core usage"),
        "total_bytes" => t("总容量", "Total"),
        "used_bytes" => t("已使用", "Used"),
        "available_bytes" => t("可用", "Available"),
        "swap_total_bytes" => t("交换空间总量", "Swap total"),
        "swap_used_bytes" => t("交换空间已用", "Swap used"),
        "name" => t("名称", "Name"),
        "id" => t("标识", "ID"),
        "related_ids" => t("相关读数标识", "Related reading IDs"),
        "vendor" => t("厂商", "Vendor"),
        "mount_point" => t("挂载点", "Mount point"),
        "file_system" => t("文件系统", "File system"),
        "is_read_only" => t("只读", "Read only"),
        "received_bytes_total" => t("累计接收", "Total received"),
        "transmitted_bytes_total" => t("累计发送", "Total sent"),
        "received_bytes_per_second" => t("接收速率", "Receive rate"),
        "transmitted_bytes_per_second" => t("发送速率", "Send rate"),
        "packets_received_total" => t("接收数据包", "Packets received"),
        "packets_transmitted_total" => t("发送数据包", "Packets sent"),
        "receive_errors_total" => t("接收错误", "Receive errors"),
        "transmit_errors_total" => t("发送错误", "Transmit errors"),
        "read_bytes_total" => t("累计读取", "Total read"),
        "written_bytes_total" => t("累计写入", "Total written"),
        "read_bytes_per_second" => t("读取速率", "Read rate"),
        "written_bytes_per_second" => t("写入速率", "Write rate"),
        "label" => t("标签", "Label"),
        "celsius" => t("当前温度", "Temperature"),
        "max_celsius" => t("最高温度", "Maximum"),
        "critical_celsius" => t("临界温度", "Critical"),
        "source" => t("来源", "Source"),
        "utilization_percent" => t("使用率", "Usage"),
        "memory_total_bytes" => t("显存总量", "Memory total"),
        "memory_used_bytes" => t("显存已用", "Memory used"),
        "temperature_celsius" => t("温度", "Temperature"),
        "power_watts" => t("功耗", "Power"),
        "core_clock_mhz" => t("核心频率", "Core clock"),
        "memory_clock_mhz" => t("显存频率", "Memory clock"),
        "pcie_rx_bytes_per_second" => t("PCIe 接收速率", "PCIe receive rate"),
        "pcie_tx_bytes_per_second" => t("PCIe 发送速率", "PCIe transmit rate"),
        "uptime_seconds" => t("运行时间", "Uptime"),
        "spool_pending_batches" => t("待发送批次", "Pending batches"),
        "collector_errors" => t("采集错误", "Collection errors"),
        _ => key.replace("_", " "),
    }
}
pub fn label(key: &str) -> String {
    let key = key.strip_prefix("system.").unwrap_or(key);
    if key.starts_with("gpu.") {
        let vendor = match key.split('.').nth(1).unwrap_or_default() {
            "nvidia" => "NVIDIA",
            "amd" => "AMD",
            "intel" => "Intel",
            "apple" => "Apple",
            "windows" => "Windows WDDM",
            _ => "",
        };
        if !vendor.is_empty() {
            return format!("{vendor} {}", t("显卡采集", "GPU collection"));
        }
    }
    match key {
        "hardware.memory" => t("内存模块识别", "Memory module inventory"),
        "hardware.thunderbolt" => t("雷电 / USB4 识别", "Thunderbolt / USB4 inventory"),
        "hardware.monitors" => t("显示器识别", "Monitor inventory"),
        "hardware.bluetooth" => t("蓝牙识别", "Bluetooth inventory"),
        "hardware.usb_controllers" => t("USB 控制器识别", "USB controller inventory"),
        "hardware.usb_devices" => t("USB 设备识别", "USB device inventory"),
        "hardware.audio" => t("音频设备识别", "Audio device inventory"),
        "id" => t("标识", "ID"),
        "name" => t("名称", "Name"),
        "os" => t("操作系统", "Operating system"),
        "arch" => t("架构", "Architecture"),
        "registered_at" => t("注册时间", "Registered at"),
        "last_seen_at" => t("最近连接", "Last connection"),
        "latest_collected_at" => t("最近上报", "Last report"),
        "status" => t("状态", "Status"),
        "capabilities" => t("采集能力", "Collection capabilities"),
        "cpu_usage_percent" => t("处理器使用率（%）", "CPU usage (%)"),
        "memory_usage_percent" => t("内存使用率（%）", "Memory usage (%)"),
        "network_received_bytes_per_second" => {
            t("网络接收（字节/秒）", "Network receive (bytes/s)")
        }
        "network_transmitted_bytes_per_second" => {
            t("网络发送（字节/秒）", "Network transmit (bytes/s)")
        }
        "disk_read_bytes_per_second" => t("磁盘读取（字节/秒）", "Disk read (bytes/s)"),
        "disk_written_bytes_per_second" => t("磁盘写入（字节/秒）", "Disk write (bytes/s)"),
        "max_temperature_celsius" => t("最高温度（℃）", "Maximum temperature (°C)"),
        "gpu_utilization_percent" => t("显卡使用率（%）", "GPU usage (%)"),
        "gpu_memory_usage_percent" => t("显存使用率（%）", "GPU memory usage (%)"),
        "online" => t("在线", "Online"),
        "stale" => t("上报延迟", "Stale"),
        "offline" => t("离线", "Offline"),
        "waiting" => t("等待确认", "Waiting for confirmation"),
        "activated" => t("已激活", "Activated"),
        "consumed" => t("已使用", "Consumed"),
        "expired" => t("已过期", "Expired"),
        "denied" => t("已拒绝", "Denied"),
        "unsupported" => t("不受支持", "Unsupported"),
        "not_present" => t("设备不存在", "Not present"),
        "driver_missing" => t("缺少驱动", "Driver missing"),
        "permission_denied" => t("权限不足", "Permission denied"),
        "transient" => t("暂时不可用", "Temporarily unavailable"),
        "invalid_data" => t("数据无效", "Invalid data"),
        "cpu" => t("处理器", "CPU"),
        "memory" => t("内存", "Memory"),
        "network" => t("网络", "Network"),
        "disk" => t("磁盘", "Disk"),
        "temperature" => t("温度", "Temperature"),
        "gpu" => t("显卡", "GPU"),
        "nvidia" => t("NVIDIA 显卡", "NVIDIA GPU"),
        _ => t("未识别的项目", "Unrecognized item"),
    }
}
pub fn parse_u64(v: &Value) -> Option<u64> {
    match v {
        Value::Number(n) => n.as_u64(),
        Value::String(s) => {
            if s == "0" || !s.starts_with('0') && s.bytes().all(|b| b.is_ascii_digit()) {
                s.parse().ok()
            } else {
                None
            }
        }
        _ => None,
    }
}
pub fn disk_record(mut v: Value) -> Value {
    if let (Some(total), Some(available)) = (
        parse_u64(&v["total_bytes"]),
        parse_u64(&v["available_bytes"]),
    ) && total > 0
        && available <= total
    {
        let used = total - available;
        v["used_bytes"] = if used > 9_007_199_254_740_991 {
            Value::String(used.to_string())
        } else {
            Value::from(used)
        };
        v["usage_percent"] = Value::from(used as f64 * 100. / total as f64);
    }
    v
}
fn localized_number(value: f64) -> String {
    #[cfg(target_arch = "wasm32")]
    {
        js_sys::Number::from(value)
            .to_locale_string(&xcss_web::i18n::locale())
            .into()
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        value.to_string()
    }
}
pub fn format_value(k: &str, v: &Value) -> String {
    if v.is_null() {
        return t("不可用", "Unavailable");
    }
    if let Some(a) = v.as_array() {
        return if a.is_empty() {
            t("无", "None")
        } else {
            a.iter()
                .map(|v| {
                    if let Some(n) = v.as_f64() {
                        format!(
                            "{n:.1}{}",
                            if k.contains("percent") {
                                "%"
                            } else if k.ends_with("_mhz") {
                                " MHz"
                            } else {
                                ""
                            }
                        )
                    } else if v.is_null() {
                        t("不可用", "Unavailable")
                    } else {
                        v.as_str()
                            .map(str::to_string)
                            .unwrap_or_else(|| v.to_string())
                    }
                })
                .collect::<Vec<_>>()
                .join(" · ")
        };
    }
    if k == "collected_at" || k == "hardware_collected_at" {
        return timestamp(v.as_str().unwrap_or_default());
    }
    if let Some(b) = v.as_bool() {
        return if b { t("是", "Yes") } else { t("否", "No") };
    }
    if k == "uptime_seconds" {
        return parse_u64(v)
            .map(|s| {
                if s >= 86400 {
                    format!(
                        "{} {} {} {}",
                        s / 86400,
                        t("天", "d"),
                        s % 86400 / 3600,
                        t("小时", "h")
                    )
                } else {
                    format!(
                        "{} {} {} {}",
                        s / 3600,
                        t("小时", "h"),
                        s % 3600 / 60,
                        t("分钟", "m")
                    )
                }
            })
            .unwrap_or(t("不可用", "Unavailable"));
    }
    if k.contains("bytes") {
        let n = parse_u64(v)
            .map(|n| n as f64)
            .or_else(|| v.as_f64())
            .unwrap_or(0.);
        return format!(
            "{}{}",
            bytes(n),
            if k.ends_with("per_second") {
                t("/秒", "/s")
            } else {
                String::new()
            }
        );
    }
    if let Some(n) = v.as_f64() {
        let unit = match k {
            "fan_rpm" => "RPM",
            "voltage_volts" => "V",
            "current_amps" => "A",
            "energy_joules" => "J",
            "link_speed_mbps" | "speed_mbps" => "Mbps",
            "speed_mt_s" | "configured_speed_mt_s" => "MT/s",
            _ => "",
        };
        if !unit.is_empty() {
            return format!("{} {unit}", localized_number(n));
        }
        if k.contains("percent") {
            return format!("{n:.1}%");
        }
        if k.contains("celsius") {
            return format!("{n:.1} ℃");
        }
        if k.ends_with("_watts") {
            return format!("{n:.1} W");
        }
        if k.ends_with("_mhz") {
            return format!("{} MHz", localized_number(n));
        }
        return localized_number(n);
    }
    v.as_str()
        .map(str::to_string)
        .unwrap_or_else(|| v.to_string())
}
#[component]
pub fn Snapshot(
    title: String,
    record: Value,
    #[prop(default = true)] show_title: bool,
    #[prop(default = true)] framed: bool,
) -> impl IntoView {
    view! {<article class=if framed{"host-device-card"}else{"host-device-card host-device-card--plain"}>{show_title.then(||view!{<h4>{title}</h4>})}<dl class="host-device-details">{record.as_object().cloned().unwrap_or_default().into_iter().map(|(k,v)|view!{<dt>{metric_label(&k)}</dt><dd>{format_value(&k,&v)}</dd>}).collect_view()}</dl></article>}
}
pub fn merge(a: &Value, b: &Value) -> Value {
    let mut out = a.as_object().cloned().unwrap_or_default();
    if let Some(b) = b.as_object() {
        out.extend(b.clone());
    }
    Value::Object(out)
}
pub fn with_time(mut v: Value, k: &str, time: &Value) -> Value {
    if let Some(o) = v.as_object_mut()
        && !time.is_null()
    {
        o.insert(k.into(), time.clone());
    }
    v
}
pub fn network_records(interfaces: Vec<Value>, hardware: Vec<Value>) -> Vec<Value> {
    let mut seen = std::collections::HashSet::new();
    let mut out = Vec::new();
    for record in &interfaces {
        let name = text(record, "name");
        let matches: Vec<_> = hardware
            .iter()
            .filter(|v| text(v, "name") == name)
            .collect();
        if name != "—"
            && interfaces
                .iter()
                .filter(|v| text(v, "name") == name)
                .count()
                == 1
            && matches.len() == 1
        {
            out.push(merge(record, matches[0]));
            seen.insert(name);
        } else {
            out.push(record.clone());
        }
    }
    out.extend(
        hardware
            .into_iter()
            .filter(|v| !seen.contains(&text(v, "name"))),
    );
    out
}
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn exact_u64_contract() {
        assert_eq!(parse_u64(&json!("18446744073709551615")), Some(u64::MAX));
        assert_eq!(parse_u64(&json!("18446744073709551616")), None);
        assert_eq!(parse_u64(&json!("01")), None);
    }
    #[test]
    fn disk_usage_derivation() {
        let d = disk_record(
            json!({"total_bytes":"18446744073709551615","available_bytes":"9223372036854775807"}),
        );
        assert_eq!(d["used_bytes"], "9223372036854775808");
        assert_eq!(d["usage_percent"], 50.);
    }
    #[test]
    fn ambiguous_networks_stay_separate() {
        let list = network_records(
            vec![json!({"name":"x"}), json!({"name":"x"})],
            vec![json!({"name":"x","model":"adapter"})],
        );
        assert_eq!(list.len(), 3);
        assert!(list[0]["model"].is_null());
    }
}
