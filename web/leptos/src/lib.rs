pub mod common;
mod contracts;
mod host;
mod metrics;
use common::*;
use leptos::prelude::*;
use serde_json::{Value, json};
use xcss_web::components::{AccountPage, AdminShell};
const INSTANCES: &str = "/api/v1/monitoring/client-instances";
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn start() {
    xcss_web::mount(
        "root",
        || view! {<AdminShell product="xsos" landing="#instances"><Application/></AdminShell>},
    );
}
fn pairing(v: &str) -> String {
    match v {
        "pending" => t("待配对", "Awaiting pairing"),
        "active" => t("已配对", "Paired"),
        "cancelled" => t("已取消", "Cancelled"),
        _ => t("未知", "Unknown"),
    }
}
#[component]
fn Application() -> impl IntoView {
    let location = route();
    let selected = RwSignal::new(
        location
            .get_untracked()
            .split('/')
            .nth(1)
            .unwrap_or_default()
            .to_string(),
    );
    Effect::new(move |_| {
        let p = location.get();
        if let Some(id) = p.split('/').nth(1) {
            selected.set(id.to_string());
        }
    });
    let revision = RwSignal::new(0);
    let busy = RwSignal::new(false);
    let error = RwSignal::new(None);
    let create = Callback::new(move |()| {
        navigate("instances");
        mutation(
            INSTANCES.into(),
            "POST",
            Some(json!({"display_name":t("新实例","New instance")})),
            busy,
            error,
            move |_| {
                xcss_web::components::notify(t(
                    "实例已创建，密码可在详细信息中查看。",
                    "Instance created. Its password is available in details.",
                ));
                revision.update(|v| *v += 1)
            },
        );
    });
    view! {<section id="hosts" class="xcss-content-stack"><HeaderActions create revision busy/><PageNav selected=Signal::derive(move||selected.get())/><h1 class="xcss-visually-hidden">{t("主机监控","Host monitoring")}</h1><Failure error message=t("实例未能创建，请刷新列表核对后重试。","The instance could not be created. Refresh the list before retrying.")/>{move||{let p=location.get().split('/').next().unwrap_or_default().to_string();match p.as_str(){"account"=>view!{<AccountPage/>}.into_any(),"details"=>view!{<InstanceDetails identity=selected.get() revision/>}.into_any(),"logs"=>view!{<Logs identity=selected.get() revision/>}.into_any(),_=>view!{<Instances revision selected/>}.into_any()}}}</section>}
}
#[component]
fn Instances(revision: RwSignal<u64>, selected: RwSignal<String>) -> impl IntoView {
    let cursor = RwSignal::new(String::new());
    let path = Signal::derive(move || {
        let c = cursor.get();
        format!(
            "{INSTANCES}{}",
            if c.is_empty() {
                String::new()
            } else {
                format!("?cursor={}", enc(&c))
            }
        )
    });
    let (data, error) = resource(path, revision, 0);
    let (stat, stat_error) = resource(
        Signal::derive(|| "/api/v1/monitoring/hosts".to_string()),
        revision,
        0,
    );
    let candidate = RwSignal::new(String::new());
    let dialog = RwSignal::new(None::<(String, Value)>);
    let busy = RwSignal::new(false);
    let action_error = RwSignal::new(None);
    let activation = RwSignal::new(
        web_sys::window()
            .unwrap()
            .location()
            .pathname()
            .ok()
            .and_then(|p| {
                p.strip_prefix("/activate/")
                    .map(|s| s.trim_end_matches('/').to_string())
            }),
    );
    view! {<Failure error message=t("无法加载实例","Unable to load instances")/><Failure error=stat_error message=t("无法加载主机列表","Unable to load hosts")/><Failure error=action_error message=t("操作未能确认，请刷新实例列表核对。","The operation could not be confirmed. Refresh and check the instance list.")/>{move||stat.get().map(|v|{let statistics=&v["statistics"];view!{<section class="xcss-content-stack" aria-label=t("实例统计","Instance statistics")><div class="xcss-table-scroll" role="region" tabindex="0" aria-label=t("实例统计","Instance statistics")><table class="xcss-table xcss-statistics-table" aria-label=t("实例统计","Instance statistics")><thead><tr><th>{t("统计项","Metric")}</th><th>{t("总数 / 在线","Total / online")}</th></tr></thead><tbody>{[(t("总数","Total"),"total"),("Windows".into(),"windows"),("Linux".into(),"linux"),("macOS".into(),"macos")].into_iter().map(|(l,k)|view!{<tr><th scope="row">{l}</th><td>{format!("{} / {}",num(&statistics[k],"total"),num(&statistics[k],"online"))}</td></tr>}).collect_view()}</tbody></table></div></section>}})}<section class="xcss-content-stack" aria-label=t("实例","Instances")>{move||data.get().map(|v|{if rows(&v,"instances").is_empty(){return view!{<div class="xcss-empty">{t("暂无实例","No instances yet")}</div>}.into_any();}let hosts=rows(&v,"hosts");view!{<div class="xcss-table-scroll" role="region" tabindex="0" aria-label=t("实例列表","Instance list")><table class="xcss-table" aria-label=t("实例列表","Instance list")><thead><tr>{[t("实例名称","Instance name"),t("配对状态","Pairing status"),t("在线状态","Online status"),t("操作系统/架构","Operating system / architecture"),t("操作","Actions"),t("删除","Delete")].into_iter().map(|l|view!{<th>{l}</th>}).collect_view()}</tr></thead><tbody>{rows(&v,"instances").into_iter().map(|row|{let host=hosts.iter().find(|h|text(h,"id")==text(&row,"instance_id")).cloned();let id=text(&row,"request_id");let id2=id.clone();let id3=id.clone();let id4=id.clone();let rotate=row.clone();let cancel=row.clone();view!{<tr><th scope="row"><a class="xcss-instance-link" aria-label=format!("{}{}",t("选择实例 ","Select instance "),text(&row,"display_name")) href=format!("#details/{id}") on:click=move |_|selected.set(id4.clone())>{text(&row,"display_name")}</a></th><td>{pairing(&text(&row,"status"))}</td><td>{host.as_ref().map(|h|if !h["data_error"].is_null(){t("数据不可用","Data unavailable")}else{metrics::label(&text(h,"status"))}).unwrap_or(if text(&row,"status")=="active"{t("等待首次上报","Waiting for first report")}else{"—".into()})}</td><td>{host.as_ref().map(|h|format!("{} / {}",text(h,"os"),text(h,"arch"))).unwrap_or("—".into())}</td><td><div class="xcss-actions">{(text(&row,"status")!="cancelled").then(||view!{<button class="xcss-button" on:click=move |_|dialog.set(Some(("rotate".into(),rotate.clone())))>{t("更换密码","Change password")}</button>})}{(text(&row,"status")=="pending").then(||view!{<button class="xcss-button" on:click=move |_|dialog.set(Some(("cancel".into(),cancel.clone())))>{t("取消配对","Cancel pairing")}</button>})}</div></td><td><div class="xcss-actions">{move||if candidate.get()==id{let target=id2.clone();view!{<button class="xcss-button" disabled=move||busy.get() on:click=move |_|candidate.set(String::new())>{t("取消","Cancel")}</button><button class="xcss-button xcss-danger" disabled=move||busy.get() on:click=move |_|mutation(format!("{INSTANCES}/{target}/delete"),"DELETE",None,busy,action_error,move |_|{candidate.set(String::new());cursor.set(String::new());revision.update(|v|*v+=1)})>{t("确认删除","Confirm delete")}</button>}.into_any()}else{let target=id3.clone();view!{<button class="xcss-button" disabled=move||busy.get() on:click=move |_|candidate.set(target.clone())>{t("删除","Delete")}</button>}.into_any()}}</div></td></tr>}}).collect_view()}</tbody></table></div>}.into_any()})}<Pagination data cursor revision/>{move||dialog.get().map(|(kind,row)|{let target=text(&row,"request_id");let rotate=kind=="rotate";view!{<Confirm title=if rotate{t("更换密码","Change password")}else{t("取消配对","Cancel pairing")} description=if rotate{t("更换密码会立即撤销现有客户端凭据。客户端必须取得新的密码并重新配对。","Changing the password immediately revokes the current client credential. The client must receive the new password and pair again.")}else{t("取消实例后，其密码将不可再用。重复取消会保留实例及历史数据；永久删除需在列表中单独确认。","Cancelling the instance invalidates its password. Repeating cancellation preserves its history; permanent deletion requires a separate confirmation.")} busy error=action_error error_message=if rotate{t("更换未能确认，请刷新实例核对密码和配对状态。","The change could not be confirmed. Refresh and verify the password and pairing state.")}else{t("取消未能确认，可重试取消或刷新核对状态；重试不会永久删除。","Cancellation could not be confirmed. Retry cancellation or refresh and check its state; retries do not permanently delete it.")} close=Callback::new(move|()|dialog.set(None)) confirm=Callback::new(move|()|{let path=if rotate{format!("{INSTANCES}/{target}/authorization")}else{format!("{INSTANCES}/{target}")};let body=rotate.then(||json!({"authorization_code":random_code()}));mutation(path,if rotate{"PUT"}else{"DELETE"},body,busy,action_error,move |_|{dialog.set(None);revision.update(|v|*v+=1);});})/>}})}{move||activation.get().map(|id|view!{<Activation initial_id=id close=Callback::new(move|()|{activation.set(None);let _=web_sys::window().unwrap().history().unwrap().replace_state_with_url(&wasm_bindgen::JsValue::NULL,"",Some("/#instances"));}) revision/>})}</section>}
}
fn random_code() -> String {
    let alphabet = b"abcdefghijklmnopqrstuvwxyz0123456789";
    let mut output = String::new();
    while output.len() < 36 {
        let mut bytes = [0u8; 64];
        web_sys::window()
            .unwrap()
            .crypto()
            .unwrap()
            .get_random_values_with_u8_array(&mut bytes)
            .unwrap();
        for b in bytes {
            if b < 252 {
                output.push(alphabet[(b % 36) as usize] as char)
            }
            if output.len() == 36 {
                break;
            }
        }
    }
    output
}
#[component]
fn Activation(initial_id: String, close: Callback<()>, revision: RwSignal<u64>) -> impl IntoView {
    let identity = RwSignal::new(initial_id);
    let detail = RwSignal::new(None::<Value>);
    let busy = RwSignal::new(false);
    let error = RwSignal::new(None);
    let password = RwSignal::new(String::new());
    let inspect = Callback::new(move |()| {
        let id = identity.get_untracked();
        if crate::contracts::uuid(&json!(id)) {
            mutation(
                format!("/api/v1/xsoc/pairing-requests/{id}"),
                "GET",
                None,
                busy,
                error,
                move |v| detail.set(Some(v)),
            );
        }
    });
    view! {<xcss_web::components::Dialog title=t("激活 客户端 配对","Activate client pairing") close=Callback::new(move|()|{if !busy.get_untracked(){close.run(())}})><form aria-busy=move||busy.get().to_string() on:submit=move|ev|{ev.prevent_default();if !crate::contracts::uuid(&json!(identity.get_untracked())){return;}
    if let Some(d)=detail.get_untracked(){let body=json!({"request_id":text(&d,"request_id"),"activation_code":password.get_untracked()});password.set(String::new());mutation("/api/v1/xsoc/activate-admin".into(),"POST",Some(body),busy,error,move |_|{close.run(());revision.update(|v|*v+=1)});}else{inspect.run(());}}><p>{t("填写 客户端 提供的配对请求标识（不是实例标识），读取并核对设备信息后，输入该实例的长期密码。","Enter the pairing request ID provided by the client (not the instance ID). Read and verify the device, then enter that instance's long-lived password.")}</p><Failure error message=t("请求未能确认。请检查设备请求是否超时及配对状态；密码本身不设有效期，输入框已清空。不要盲目重试激活。","The request could not be confirmed. Check request timeout and pairing state. The password itself has no expiry; the input has been cleared. Do not blindly retry activation.")/><label class="xcss-form-field">{t("配对请求标识","Pairing request ID")}<input class="xcss-input" prop:value=move||identity.get() required maxlength="36" data-xcss-initial-focus pattern="[0-9a-f]{8}-[0-9a-f]{4}-[1-8][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}" readonly=move||busy.get() on:input=move|ev|{identity.set(event_target_value(&ev));detail.set(None);}/></label>{move||detail.get().map(|d|view!{<section aria-label=t("待核对设备","Device to verify")><p>{format!("{}{} / {}",t("系统：","System:"),text(&d,"os"),text(&d,"arch"))}</p><p>{format!("{}{}{}{}{}",t("配对状态：","Pairing status:"),metrics::label(&text(&d,"status")),t("；设备请求会话截止：","; device request session deadline:"),timestamp(&text(&d,"expires_at")),t("（不是密码有效期）","(not the password expiry)"))}</p></section>{(text(&d,"status")=="waiting").then(||view!{<label class="xcss-form-field">{t("密码","Password")}<input class="xcss-input" name="activation_code" type="password" prop:value=move||password.get() required maxlength="256" autocomplete="off" readonly=move||busy.get() on:input=move|ev|password.set(event_target_value(&ev))/></label>})}})}<div class="xcss-actions"><button class="xcss-button" type="button" disabled=move||busy.get() on:click=move |_|close.run(())>{t("取消","Cancel")}</button>{move||detail.get().is_some().then(||view!{<button class="xcss-button" type="button" disabled=move||busy.get() on:click=move |_|inspect.run(())>{t("刷新配对状态","Refresh pairing status")}</button>})}<button class="xcss-button" type="submit" disabled=move||busy.get()||detail.get().is_some_and(|d|text(&d,"status")!="waiting")>{move||if busy.get(){t("正在处理…","Processing…")}else if detail.get().is_some(){t("确认设备并激活","Confirm device and activate")}else{t("读取配对请求","Read pairing request")}}</button></div></form></xcss_web::components::Dialog>}
}
#[component]
fn InstanceDetails(identity: String, revision: RwSignal<u64>) -> impl IntoView {
    let path = format!("{INSTANCES}?instance_id={}", enc(&identity));
    let (data, error) = resource(Signal::derive(move || path.clone()), revision, 0);
    let draft = RwSignal::new(None::<String>);
    view! {<section class="xcss-content-stack" aria-label=t("详细信息与设置","Details and settings")>{move||view!{<Failure error message=if data.get().is_some(){t("实例详情刷新失败，页面保留上次成功数据。","Instance details refresh failed. The last successful data is retained.")}else{t("无法读取实例详情","Unable to load instance details")}/>}}<Show when=move||data.get().is_none()&&error.get().is_none()><div class="xcss-loading">{t("正在读取实例详情…","Loading instance details…")}</div></Show><Show when=move||data.get().is_some_and(|v|rows(&v,"instances").is_empty())><div class="xcss-empty">{t("所选实例已不存在，请返回实例列表。","The selected instance no longer exists. Return to the instance list.")}</div></Show><Show when=move||data.get().is_some_and(|v|rows(&v,"hosts").iter().any(|h|!h["data_error"].is_null()))><p role="alert" class="xcss-error">{t("该实例的监控数据无法读取。","Monitoring data for this instance is unavailable.")}</p></Show><For each={move||data.get().map(|v|rows(&v,"instances")).unwrap_or_default()} key=|v|text(v,"request_id") children=move|instance|{let request_id=text(&instance,"request_id");let instance=Signal::derive(move||data.get().and_then(|v|rows(&v,"instances").into_iter().find(|i|text(i,"request_id")==request_id)).unwrap_or(Value::Null));view!{<InstanceBody instance revision draft/>}}/></section>}
}
#[component]
fn InstanceBody(
    instance: Signal<Value>,
    revision: RwSignal<u64>,
    draft: RwSignal<Option<String>>,
) -> impl IntoView {
    let host_id = Signal::derive(move || text(&instance.get(), "instance_id"));
    let overview = move || view! {<InstanceOverview instance revision draft/>};
    view! {<For each=move||vec![(host_id.get(),text(&instance.get(),"status"))] key=|(id,status)|(id.clone(),status.clone()) children=move|(id,status)|{if status=="active"{view!{<host::HostWorkspace host_id=id revision>{overview()}</host::HostWorkspace>}.into_any()}else{view!{<section class="xcss-content-panel host-overview" aria-label=t("实例概览","Instance overview")>{overview()}<section class="host-overview-section"><h3>{t("监控状态","Monitoring status")}</h3><p>{if status=="pending"{t("实例尚未配对，完成客户端配对后将显示监控详情。","This instance is not paired yet. Monitoring details will appear after client pairing.")}else{t("实例配对已取消。","Instance pairing has been cancelled.")}}</p></section></section>}.into_any()}}/>}
}
#[component]
fn InstanceOverview(
    instance: Signal<Value>,
    revision: RwSignal<u64>,
    draft: RwSignal<Option<String>>,
) -> impl IntoView {
    let busy = RwSignal::new(false);
    let error = RwSignal::new(None);
    view! {<section aria-label=t("配对账户信息","Pairing account information")><dl class="host-detail-list"><dt>{t("账户名","Account name")}</dt><dd>{move||text(&instance.get(),"display_name")}</dd><dt>{t("账户","Account")}</dt><dd><code>{move||text(&instance.get(),"instance_id")}</code></dd><dt>{t("密码","Password")}</dt><dd><code>{move||text(&instance.get(),"authorization_code")}</code></dd><dt>{t("配对状态","Pairing status")}</dt><dd>{move||pairing(&text(&instance.get(),"status"))}</dd></dl></section><section class="host-overview-section host-instance-settings" aria-label=t("实例设置","Instance settings")><form aria-busy=move||busy.get().to_string() on:submit=move|ev|{ev.prevent_default();let name=draft.get_untracked().unwrap_or_else(||text(&instance.get_untracked(),"display_name")).trim().to_string();if valid_name(&name){mutation(format!("{INSTANCES}/{}",text(&instance.get_untracked(),"request_id")),"PATCH",Some(json!({"display_name":name})),busy,error,move |_|{draft.set(None);revision.update(|v|*v+=1);xcss_web::components::notify(t("实例名称已保存","Instance name saved"));})}}><label class="xcss-form-field">{t("实例名称","Instance name")}<input class="xcss-input" name="display_name" prop:value=move||draft.get().unwrap_or_else(||text(&instance.get(),"display_name")) required readonly=move||busy.get() on:input=move|ev|draft.set(Some(event_target_value(&ev)))/></label>{move||draft.get().is_some_and(|n|!valid_name(&n)).then(||view!{<p role="alert">{t("名称须为 1–32 个字符，不能包含控制字符。","Use 1–32 characters without control characters.")}</p>})}<Failure error message=t("实例名称未能保存，请重试。","The instance name could not be saved. Please retry.")/><div class="xcss-actions"><button class="xcss-button" type="submit" disabled=move||busy.get()||draft.get().is_none_or(|n|!valid_name(&n)||n.trim()==text(&instance.get(),"display_name"))>{t("保存名称","Save name")}</button></div></form></section>}
}
#[component]
fn Logs(identity: String, revision: RwSignal<u64>) -> impl IntoView {
    let focused = format!("{INSTANCES}?instance_id={}", enc(&identity));
    let (instance, error) = resource(Signal::derive(move || focused.clone()), revision, 0);
    view! {<Failure error retry=Callback::new(move|()|revision.update(|v|*v+=1)) message=t("所选实例刷新失败，页面保留上次成功读取的实例信息。","Selected instance refresh failed. The last successful instance information is retained.")/><For each={move||instance.get().map(|v|rows(&v,"hosts")).unwrap_or_default()} key=|h|text(h,"id") children=move|host|view!{<HostLogs host revision/>}/>}
}
#[component]
fn HostLogs(host: Value, revision: RwSignal<u64>) -> impl IntoView {
    let (calendar, calendar_error) = resource(
        Signal::derive(|| "/api/v1/monitoring/logs/calendar".into()),
        RwSignal::new(0),
        0,
    );
    let range = RwSignal::new((String::new(), String::new()));
    let cursor = RwSignal::new(String::new());
    let local = RwSignal::new(0);
    Effect::new(move |_| {
        if let Some(c) = calendar.get() {
            let d = text(&c, "today");
            range.set((d.clone(), d));
        }
    });
    Effect::new(move |_| {
        revision.get();
        cursor.set(String::new());
        local.update(|v| *v += 1);
    });
    let id = text(&host, "id");
    let path = Signal::derive(move || {
        let (s, e) = range.get();
        if s.is_empty() {
            return String::new();
        }
        let q = if s == e {
            format!("date={}", enc(&s))
        } else {
            format!("start_date={}&end_date={}", enc(&s), enc(&e))
        };
        let c = cursor.get();
        format!(
            "/api/v1/monitoring/hosts/{id}/reports?{q}{}",
            if c.is_empty() {
                String::new()
            } else {
                format!("&cursor={}", enc(&c))
            }
        )
    });
    let (data, error) = resource(path, local, 0);
    view! {<section class="xcss-content-stack" aria-label=t("上报日志","Report logs")><div class="xcss-content-panel xcss-content-stack host-logs-panel"><p>{format!("{}{}",t("实例：","Instance: "),text(&host,"name"))}</p><div class="xcss-log-date-controls"><label for="host-log-date-start-year">{t("日志日期范围（UTC）","Log date range (UTC)")}</label><div class="xcss-actions"><button class="xcss-button" on:click=move |_|{cursor.set(String::new());local.update(|v|*v+=1)}>{t("刷新日志","Refresh logs")}</button></div><DateRange id="host-log-date" value=range apply=Callback::new(move|r|{range.set(r);cursor.set(String::new());local.update(|v|*v+=1);})/></div></div><Failure error=calendar_error message=t("无法读取UTC 当天日期。","Unable to load the current UTC date.")/><Failure error message=t("无法读取上报日志。","Unable to load report logs.")/>{move||data.get().map(|d|view!{<div class="xcss-table-scroll"><table class="xcss-table"><thead><tr><th>{t("服务端接收时间（UTC）","Server receipt time (UTC)")}</th><th>{t("客户端采集时间（UTC）","Client collection time (UTC)")}</th><th>{t("报告 ID","Report ID")}</th></tr></thead><tbody>{rows(&d,"reports").into_iter().map(|r|view!{<tr><td>{text(&r,"received_at")}</td><td>{text(&r,"collected_at")}</td><td><code>{text(&r,"report_id")}</code></td></tr>}).collect_view()}</tbody></table></div>})}<Pagination data cursor revision=local label=t("日志分页","Log pages")/></section>}
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn instance_names_reject_controls_and_count_scalars() {
        assert!(valid_name("测试实例"));
        assert!(!valid_name("\u{7f}bad"));
        assert!(!valid_name(&"🦀".repeat(33)));
    }
    #[test]
    fn leap_year_ranges() {
        assert!(date_validate(&["2000".into(), "2".into(), "29".into()]).is_ok());
        assert_eq!(
            date_validate(&["2100".into(), "2".into(), "29".into()]),
            Err(2)
        );
    }
}
