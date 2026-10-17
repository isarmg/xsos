use leptos::component::{component_props_builder, component_view};
pub mod common;
mod contracts;
mod host;
mod metrics;
mod models;
use common::*;
use leptos::prelude::*;
use models::*;
use serde_json::{Value, json};
use xcss_web::components::{AccountPage, AdminShell};
const INSTANCES: &str = "/api/v1/monitoring/client-instances";
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn start() {
    xcss_web::mount("root", || {
        View::new({
            #[allow(unreachable_code)]
            #[allow(unused_mut)]
            #[allow(clippy::let_and_return, clippy::let_unit_value, clippy::unit_arg)]
            component_view(
                #[allow(clippy::needless_borrows_for_generic_args)]
                &AdminShell,
                {
                    let mut props = component_props_builder(&AdminShell)
                        .product(
                            #[allow(unused_braces)]
                            {
                                "xsos"
                            },
                        )
                        .landing(
                            #[allow(unused_braces)]
                            {
                                "#instances"
                            },
                        )
                        .children({
                            ::leptos::children::ToChildren::to_children(move || {
                                #[allow(unreachable_code)]
                                #[allow(unused_mut)]
                                #[allow(
                                    clippy::let_and_return,
                                    clippy::let_unit_value,
                                    clippy::unit_arg
                                )]
                                component_view(
                                    #[allow(clippy::needless_borrows_for_generic_args)]
                                    &Application,
                                    {
                                        let mut props =
                                            component_props_builder(&Application).build();
                                        props
                                    },
                                )
                            })
                        })
                        .build();
                    props
                },
            )
        })
    });
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
        mutation_json::<Instance>(
            INSTANCES.into(),
            "POST",
            InstanceName {
                display_name: t("新实例", "New instance"),
            },
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
    View::new(
    leptos::html::section()
        .child((
            {
                #[allow(unreachable_code)] #[allow(unused_mut)]
                #[allow(clippy::let_and_return, clippy::let_unit_value, clippy::unit_arg)]
                component_view(
                    #[allow(clippy::needless_borrows_for_generic_args)]
                    &HeaderActions,
                    {
                        let mut props = component_props_builder(
                                &HeaderActions,
                            )
                            .create(#[allow(unused_braces)] { create })
                            .revision(#[allow(unused_braces)] { revision })
                            .busy(#[allow(unused_braces)] { busy })
                            .build();
                        props
                    },
                )
            },
            {
                #[allow(unreachable_code)] #[allow(unused_mut)]
                #[allow(clippy::let_and_return, clippy::let_unit_value, clippy::unit_arg)]
                component_view(
                    #[allow(clippy::needless_borrows_for_generic_args)]
                    &PageNav,
                    {
                        let mut props = component_props_builder(
                                &PageNav,
                            )
                            .selected(
                                #[allow(unused_braces)]
                                { Signal::derive(move || selected.get()) },
                            )
                            .build();
                        props
                    },
                )
            },
            leptos::html::h1()
                .child(
                    #[allow(unused_braces)]
                    {
                        IntoRender::into_render({
                            t("主机监控", "Host monitoring")
                        })
                    },
                )
                .class("xcss-visually-hidden"),
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
                            .message(
                                #[allow(unused_braces)]
                                {
                                    t(
                                        "实例未能创建，请刷新列表核对后重试。",
                                        "The instance could not be created. Refresh the list before retrying.",
                                    )
                                },
                            )
                            .build();
                        props
                    },
                )
            },
            IntoRender::into_render({
                move || {
                    let p = location
                        .get()
                        .split('/')
                        .next()
                        .unwrap_or_default()
                        .to_string();
                    match p.as_str() {
                        "account" => {
                            View::new({
    #[allow(unreachable_code)] #[allow(unused_mut)] #[allow(clippy::let_and_return, clippy::let_unit_value, clippy::unit_arg)]
    component_view(
        #[allow(clippy::needless_borrows_for_generic_args)]
        &AccountPage,
        {
            let mut props = component_props_builder(
                    &AccountPage,
                )
                .build();
            props
        },
    )
})
                                .into_any()
                        }
                        "details" => {
                            View::new({
    #[allow(unreachable_code)] #[allow(unused_mut)] #[allow(clippy::let_and_return, clippy::let_unit_value, clippy::unit_arg)]
    component_view(
        #[allow(clippy::needless_borrows_for_generic_args)]
        &InstanceDetails,
        {
            let mut props = component_props_builder(
                    &InstanceDetails,
                )
                .identity(#[allow(unused_braces)] { selected.get() })
                .revision(#[allow(unused_braces)] { revision })
                .build();
            props
        },
    )
})
                                .into_any()
                        }
                        "logs" => {
                            View::new({
    #[allow(unreachable_code)] #[allow(unused_mut)] #[allow(clippy::let_and_return, clippy::let_unit_value, clippy::unit_arg)]
    component_view(
        #[allow(clippy::needless_borrows_for_generic_args)]
        &Logs,
        {
            let mut props = component_props_builder(&Logs)
                .identity(#[allow(unused_braces)] { selected.get() })
                .revision(#[allow(unused_braces)] { revision })
                .build();
            props
        },
    )
})
                                .into_any()
                        }
                        _ => {
                            View::new({
    #[allow(unreachable_code)] #[allow(unused_mut)] #[allow(clippy::let_and_return, clippy::let_unit_value, clippy::unit_arg)]
    component_view(
        #[allow(clippy::needless_borrows_for_generic_args)]
        &Instances,
        {
            let mut props = component_props_builder(&Instances)
                .revision(#[allow(unused_braces)] { revision })
                .selected(#[allow(unused_braces)] { selected })
                .build();
            props
        },
    )
})
                                .into_any()
                        }
                    }
                }
            }),
        ))
        .class("xcss-content-stack")
        .id("hosts"),
)
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
    let (data, error) = resource_typed::<InstancesPage>(path, revision, 0);
    let (stat, stat_error) = resource_typed::<HostsPage>(
        Signal::derive(|| "/api/v1/monitoring/hosts".to_string()),
        revision,
        0,
    );
    let candidate = RwSignal::new(String::new());
    let dialog = RwSignal::new(None::<(String, Instance)>);
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
    View::new((
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
                    .message(
                        #[allow(unused_braces)]
                        { t("无法加载实例", "Unable to load instances") },
                    )
                    .build();
                props
            },
        )
    },
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
                    .error(#[allow(unused_braces)] { stat_error })
                    .message(
                        #[allow(unused_braces)]
                        { t("无法加载主机列表", "Unable to load hosts") },
                    )
                    .build();
                props
            },
        )
    },
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
                    .error(#[allow(unused_braces)] { action_error })
                    .message(
                        #[allow(unused_braces)]
                        {
                            t(
                                "操作未能确认，请刷新实例列表核对。",
                                "The operation could not be confirmed. Refresh and check the instance list.",
                            )
                        },
                    )
                    .build();
                props
            },
        )
    },
    IntoRender::into_render({
        move || {
            stat
                .get()
                .map(|v| {
                    let statistics = v.statistics;
                    View::new(
    leptos::html::section()
        .child(
            #[allow(unused_braces)]
            {
                leptos::html::div()
                    .child(
                        #[allow(unused_braces)]
                        {
                            leptos::html::table()
                                .child((
                                    leptos::html::thead()
                                        .child(
                                            #[allow(unused_braces)]
                                            {
                                                leptos::html::tr()
                                                    .child((
                                                        leptos::html::th()
                                                            .child(
                                                                #[allow(unused_braces)]
                                                                {
                                                                    IntoRender::into_render({
                                                                        t("统计项", "Metric")
                                                                    })
                                                                },
                                                            ),
                                                        leptos::html::th()
                                                            .child(
                                                                #[allow(unused_braces)]
                                                                {
                                                                    IntoRender::into_render({
                                                                        t("总数 / 在线", "Total / online")
                                                                    })
                                                                },
                                                            ),
                                                    ))
                                            },
                                        ),
                                    leptos::html::tbody()
                                        .child(
                                            #[allow(unused_braces)]
                                            {
                                                IntoRender::into_render({
                                                    [
                                                        (t("总数", "Total"), statistics.total),
                                                        ("Windows".into(), statistics.windows),
                                                        ("Linux".into(), statistics.linux),
                                                        ("macOS".into(), statistics.macos),
                                                    ]
                                                        .into_iter()
                                                        .map(|(l, k)| {
                                                            View::new(
    leptos::html::tr()
        .child((
            leptos::html::th()
                .child(
                    #[allow(unused_braces)]
                    { IntoRender::into_render({ l }) },
                )
                .scope("row"),
            leptos::html::td()
                .child(
                    #[allow(unused_braces)]
                    {
                        IntoRender::into_render({
                            format!("{} / {}", k.total, k.online)
                        })
                    },
                ),
        )),
)
                                                        })
                                                        .collect_view()
                                                })
                                            },
                                        ),
                                ))
                                .class("xcss-table xcss-statistics-table")
                                .aria_label(
                                    ::leptos::prelude::IntoAttributeValue::into_attribute_value(
                                        t("实例统计", "Instance statistics"),
                                    ),
                                )
                        },
                    )
                    .class("xcss-table-scroll")
                    .role("region")
                    .tabindex("0")
                    .aria_label(
                        ::leptos::prelude::IntoAttributeValue::into_attribute_value(
                            t("实例统计", "Instance statistics"),
                        ),
                    )
            },
        )
        .class("xcss-content-stack")
        .aria_label(
            ::leptos::prelude::IntoAttributeValue::into_attribute_value(
                t("实例统计", "Instance statistics"),
            ),
        ),
)
                })
        }
    }),
    leptos::html::section()
        .child((
            IntoRender::into_render({
                move || {
                    data
                        .get()
                        .map(|v| {
                            if v.instances.is_empty() {
                                return View::new(
    leptos::html::div()
        .child(
            #[allow(unused_braces)]
            {
                IntoRender::into_render({
                    t("暂无实例", "No instances yet")
                })
            },
        )
        .class("xcss-empty"),
)
                                    .into_any();
                            }
                            let hosts = v.hosts;
                            View::new(
    leptos::html::div()
        .child(
            #[allow(unused_braces)]
            {
                leptos::html::table()
                    .child((
                        leptos::html::thead()
                            .child(
                                #[allow(unused_braces)]
                                {
                                    leptos::html::tr()
                                        .child(
                                            #[allow(unused_braces)]
                                            {
                                                IntoRender::into_render({
                                                    [
                                                        t("实例名称", "Instance name"),
                                                        t("配对状态", "Pairing status"),
                                                        t("在线状态", "Online status"),
                                                        t("操作系统/架构", "Operating system / architecture"),
                                                        t("操作", "Actions"),
                                                        t("删除", "Delete"),
                                                    ]
                                                        .into_iter()
                                                        .map(|l| {
                                                            View::new(
    leptos::html::th()
        .child(
            #[allow(unused_braces)]
            { IntoRender::into_render({ l }) },
        ),
)
                                                        })
                                                        .collect_view()
                                                })
                                            },
                                        )
                                },
                            ),
                        leptos::html::tbody()
                            .child(
                                #[allow(unused_braces)]
                                {
                                    IntoRender::into_render({
                                        v.instances
                                            .into_iter()
                                            .map(|row| {
                                                let host = hosts
                                                    .iter()
                                                    .find(|h| h.id.clone() == row.instance_id.clone())
                                                    .cloned();
                                                let id = row.request_id.clone();
                                                let id2 = id.clone();
                                                let id3 = id.clone();
                                                let id4 = id.clone();
                                                let rotate = row.clone();
                                                let cancel = row.clone();
                                                View::new(
    leptos::html::tr()
        .child((
            leptos::html::th()
                .child(
                    #[allow(unused_braces)]
                    {
                        leptos::html::a()
                            .child(
                                #[allow(unused_braces)]
                                {
                                    IntoRender::into_render({
                                        row.display_name.clone()
                                    })
                                },
                            )
                            .class("xcss-instance-link")
                            .aria_label(
                                ::leptos::prelude::IntoAttributeValue::into_attribute_value(
                                    format!(
                                        "{}{}", t("选择实例 ", "Select instance "), row
                                        .display_name.clone()
                                    ),
                                ),
                            )
                            .href(
                                ::leptos::prelude::IntoAttributeValue::into_attribute_value(
                                    format!("#details/{id}"),
                                ),
                            )
                            .on(
                                leptos::ev::click,
                                move |_| selected.set(id4.clone()),
                            )
                    },
                )
                .scope("row"),
            leptos::html::td()
                .child(
                    #[allow(unused_braces)]
                    {
                        IntoRender::into_render({
                            pairing(&row.status.clone())
                        })
                    },
                ),
            leptos::html::td()
                .child(
                    #[allow(unused_braces)]
                    {
                        IntoRender::into_render({
                            host.as_ref()
                                .map(|h| {
                                    if h.data_error.is_some() {
                                        t("数据不可用", "Data unavailable")
                                    } else {
                                        metrics::label(&h.status.clone())
                                    }
                                })
                                .unwrap_or(
                                    if row.status.clone() == "active" {
                                        t("等待首次上报", "Waiting for first report")
                                    } else {
                                        "—".into()
                                    },
                                )
                        })
                    },
                ),
            leptos::html::td()
                .child(
                    #[allow(unused_braces)]
                    {
                        IntoRender::into_render({
                            host.as_ref()
                                .map(|h| format!("{} / {}", h.os.clone(), h.arch.clone()))
                                .unwrap_or("—".into())
                        })
                    },
                ),
            leptos::html::td()
                .child(
                    #[allow(unused_braces)]
                    {
                        leptos::html::div()
                            .child((
                                IntoRender::into_render({
                                    (row.status.clone() != "cancelled")
                                        .then(|| {
                                            View::new(
    leptos::html::button()
        .child(
            #[allow(unused_braces)]
            {
                IntoRender::into_render({
                    t("更换密码", "Change password")
                })
            },
        )
        .class("xcss-button")
        .on(
            leptos::ev::click,
            move |_| dialog.set(Some(("rotate".into(), rotate.clone()))),
        ),
)
                                        })
                                }),
                                IntoRender::into_render({
                                    (row.status.clone() == "pending")
                                        .then(|| {
                                            View::new(
    leptos::html::button()
        .child(
            #[allow(unused_braces)]
            {
                IntoRender::into_render({
                    t("取消配对", "Cancel pairing")
                })
            },
        )
        .class("xcss-button")
        .on(
            leptos::ev::click,
            move |_| dialog.set(Some(("cancel".into(), cancel.clone()))),
        ),
)
                                        })
                                }),
                            ))
                            .class("xcss-actions")
                    },
                ),
            leptos::html::td()
                .child(
                    #[allow(unused_braces)]
                    {
                        leptos::html::div()
                            .child(
                                #[allow(unused_braces)]
                                {
                                    IntoRender::into_render({
                                        move || {
                                            if candidate.get() == id {
                                                let target = id2.clone();
                                                View::new((
    leptos::html::button()
        .child(
            #[allow(unused_braces)]
            { IntoRender::into_render({ t("取消", "Cancel") }) },
        )
        .class("xcss-button")
        .disabled(
            ::leptos::prelude::IntoAttributeValue::into_attribute_value(move || {
                busy.get()
            }),
        )
        .on(
            leptos::ev::click,
            move |_| candidate.set(String::new()),
        ),
    leptos::html::button()
        .child(
            #[allow(unused_braces)]
            {
                IntoRender::into_render({
                    t("确认删除", "Confirm delete")
                })
            },
        )
        .class("xcss-button xcss-danger")
        .disabled(
            ::leptos::prelude::IntoAttributeValue::into_attribute_value(move || {
                busy.get()
            }),
        )
        .on(
            leptos::ev::click,
            move |_| mutation(
                format!("{INSTANCES}/{target}/delete"),
                "DELETE",
                None,
                busy,
                action_error,
                move |_| {
                    candidate.set(String::new());
                    cursor.set(String::new());
                    revision.update(|v| *v += 1)
                },
            ),
        ),
))
                                                    .into_any()
                                            } else {
                                                let target = id3.clone();
                                                View::new(
    leptos::html::button()
        .child(
            #[allow(unused_braces)]
            { IntoRender::into_render({ t("删除", "Delete") }) },
        )
        .class("xcss-button")
        .disabled(
            ::leptos::prelude::IntoAttributeValue::into_attribute_value(move || {
                busy.get()
            }),
        )
        .on(
            leptos::ev::click,
            move |_| candidate.set(target.clone()),
        ),
)
                                                    .into_any()
                                            }
                                        }
                                    })
                                },
                            )
                            .class("xcss-actions")
                    },
                ),
        )),
)
                                            })
                                            .collect_view()
                                    })
                                },
                            ),
                    ))
                    .class("xcss-table")
                    .aria_label(
                        ::leptos::prelude::IntoAttributeValue::into_attribute_value(
                            t("实例列表", "Instance list"),
                        ),
                    )
            },
        )
        .class("xcss-table-scroll")
        .role("region")
        .tabindex("0")
        .aria_label(
            ::leptos::prelude::IntoAttributeValue::into_attribute_value(
                t("实例列表", "Instance list"),
            ),
        ),
)
                                .into_any()
                        })
                }
            }),
            {
                #[allow(unreachable_code)] #[allow(unused_mut)]
                #[allow(clippy::let_and_return, clippy::let_unit_value, clippy::unit_arg)]
                component_view(
                    #[allow(clippy::needless_borrows_for_generic_args)]
                    &Pagination,
                    {
                        let mut props = component_props_builder(
                                &Pagination,
                            )
                            .data(#[allow(unused_braces)] { data })
                            .cursor(#[allow(unused_braces)] { cursor })
                            .revision(#[allow(unused_braces)] { revision })
                            .build();
                        props
                    },
                )
            },
            IntoRender::into_render({
                move || {
                    dialog
                        .get()
                        .map(|(kind, row)| {
                            let target = row.request_id.clone();
                            let rotate = kind == "rotate";
                            View::new({
    #[allow(unreachable_code)] #[allow(unused_mut)] #[allow(clippy::let_and_return, clippy::let_unit_value, clippy::unit_arg)]
    component_view(
        #[allow(clippy::needless_borrows_for_generic_args)]
        &Confirm,
        {
            let mut props = component_props_builder(&Confirm)
                .title(
                    #[allow(unused_braces)]
                    {
                        if rotate {
                            t("更换密码", "Change password")
                        } else {
                            t("取消配对", "Cancel pairing")
                        }
                    },
                )
                .description(
                    #[allow(unused_braces)]
                    {
                        if rotate {
                            t(
                                "更换密码会立即撤销现有客户端凭据。客户端必须取得新的密码并重新配对。",
                                "Changing the password immediately revokes the current client credential. The client must receive the new password and pair again.",
                            )
                        } else {
                            t(
                                "取消实例后，其密码将不可再用。重复取消会保留实例及历史数据；永久删除需在列表中单独确认。",
                                "Cancelling the instance invalidates its password. Repeating cancellation preserves its history; permanent deletion requires a separate confirmation.",
                            )
                        }
                    },
                )
                .busy(#[allow(unused_braces)] { busy })
                .error(#[allow(unused_braces)] { action_error })
                .error_message(
                    #[allow(unused_braces)]
                    {
                        if rotate {
                            t(
                                "更换未能确认，请刷新实例核对密码和配对状态。",
                                "The change could not be confirmed. Refresh and verify the password and pairing state.",
                            )
                        } else {
                            t(
                                "取消未能确认，可重试取消或刷新核对状态；重试不会永久删除。",
                                "Cancellation could not be confirmed. Retry cancellation or refresh and check its state; retries do not permanently delete it.",
                            )
                        }
                    },
                )
                .close(
                    #[allow(unused_braces)]
                    { Callback::new(move |()| dialog.set(None)) },
                )
                .confirm(
                    #[allow(unused_braces)]
                    {
                        Callback::new(move |()| {
                            let path = if rotate {
                                format!("{INSTANCES}/{target}/authorization")
                            } else {
                                format!("{INSTANCES}/{target}")
                            };
                            let body = rotate
                                .then(|| json!({ "authorization_code" : random_code() }));
                            mutation(
                                path,
                                if rotate { "PUT" } else { "DELETE" },
                                body,
                                busy,
                                action_error,
                                move |_| {
                                    dialog.set(None);
                                    revision.update(|v| *v += 1);
                                },
                            );
                        })
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
            IntoRender::into_render({
                move || {
                    activation
                        .get()
                        .map(|id| {
                            View::new({
    #[allow(unreachable_code)] #[allow(unused_mut)] #[allow(clippy::let_and_return, clippy::let_unit_value, clippy::unit_arg)]
    component_view(
        #[allow(clippy::needless_borrows_for_generic_args)]
        &Activation,
        {
            let mut props = component_props_builder(&Activation)
                .initial_id(#[allow(unused_braces)] { id })
                .close(
                    #[allow(unused_braces)]
                    {
                        Callback::new(move |()| {
                            activation.set(None);
                            let _ = web_sys::window()
                                .unwrap()
                                .history()
                                .unwrap()
                                .replace_state_with_url(
                                    &wasm_bindgen::JsValue::NULL,
                                    "",
                                    Some("/#instances"),
                                );
                        })
                    },
                )
                .revision(#[allow(unused_braces)] { revision })
                .build();
            props
        },
    )
})
                        })
                }
            }),
        ))
        .class("xcss-content-stack")
        .aria_label(
            ::leptos::prelude::IntoAttributeValue::into_attribute_value(
                t("实例", "Instances"),
            ),
        ),
))
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
    let detail = RwSignal::new(None::<PairingRequest>);
    let busy = RwSignal::new(false);
    let error = RwSignal::new(None);
    let password = RwSignal::new(String::new());
    let inspect = Callback::new(move |()| {
        let id = identity.get_untracked();
        if crate::contracts::uuid(&json!(id)) {
            mutation_typed::<PairingRequest>(
                format!("/api/v1/xsoc/pairing-requests/{id}"),
                "GET",
                None,
                busy,
                error,
                move |v| detail.set(Some(v)),
            );
        }
    });
    View::new({
        #[allow(unreachable_code)]
        #[allow(unused_mut)]
        #[allow(clippy::let_and_return, clippy::let_unit_value, clippy::unit_arg)]
        component_view(
            #[allow(clippy::needless_borrows_for_generic_args)]
            &xcss_web::components::Dialog,
            {
                let mut props = component_props_builder(
                    &xcss_web::components::Dialog,
                )
                .title(
                    #[allow(unused_braces)]
                    { t("激活 客户端 配对", "Activate client pairing") },
                )
                .close(
                    #[allow(unused_braces)]
                    {
                        Callback::new(move |()| {
                            if !busy.get_untracked() {
                                close.run(())
                            }
                        })
                    },
                )
                .children({
                    ::leptos::children::ToChildren::to_children(move || {
                        leptos::html::form()
                            .child((
                                leptos::html::p()
                                    .child(
                                        #[allow(unused_braces)]
                                        {
                                            IntoRender::into_render({
                                                t(
                                                    "填写 客户端 提供的配对请求标识（不是实例标识），读取并核对设备信息后，输入该实例的长期密码。",
                                                    "Enter the pairing request ID provided by the client (not the instance ID). Read and verify the device, then enter that instance's long-lived password.",
                                                )
                                            })
                                        },
                                    ),
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
                                                .message(
                                                    #[allow(unused_braces)]
                                                    {
                                                        t(
                                                            "请求未能确认。请检查设备请求是否超时及配对状态；密码本身不设有效期，输入框已清空。不要盲目重试激活。",
                                                            "The request could not be confirmed. Check request timeout and pairing state. The password itself has no expiry; the input has been cleared. Do not blindly retry activation.",
                                                        )
                                                    },
                                                )
                                                .build();
                                            props
                                        },
                                    )
                                },
                                leptos::html::label()
                                    .child((
                                        IntoRender::into_render({
                                            t("配对请求标识", "Pairing request ID")
                                        }),
                                        leptos::html::input()
                                            .class("xcss-input")
                                            .prop("value", move || identity.get())
                                            .required(true)
                                            .maxlength("36")
                                            .attr("data-xcss-initial-focus", true)
                                            .pattern(
                                                "[0-9a-f]{8}-[0-9a-f]{4}-[1-8][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}",
                                            )
                                            .readonly(
                                                ::leptos::prelude::IntoAttributeValue::into_attribute_value(move ||
                                                busy.get()),
                                            )
                                            .on(
                                                leptos::ev::input,
                                                move |ev| {
                                                    identity.set(event_target_value(&ev));
                                                    detail.set(None);
                                                },
                                            ),
                                    ))
                                    .class("xcss-form-field"),
                                IntoRender::into_render({
                                    move || {
                                        detail
                                            .get()
                                            .map(|d| {
                                                View::new((
    leptos::html::section()
        .child((
            leptos::html::p()
                .child(
                    #[allow(unused_braces)]
                    {
                        IntoRender::into_render({
                            format!(
                                "{}{} / {}", t("系统：", "System:"), d.os.clone(), d.arch
                                .clone()
                            )
                        })
                    },
                ),
            leptos::html::p()
                .child(
                    #[allow(unused_braces)]
                    {
                        IntoRender::into_render({
                            format!(
                                "{}{}{}{}{}", t("配对状态：", "Pairing status:"),
                                metrics::label(& d.status.clone()),
                                t("；设备请求会话截止：",
                                "; device request session deadline:"), timestamp(& d
                                .expires_at.clone()), t("（不是密码有效期）",
                                "(not the password expiry)")
                            )
                        })
                    },
                ),
        ))
        .aria_label(
            ::leptos::prelude::IntoAttributeValue::into_attribute_value(
                t("待核对设备", "Device to verify"),
            ),
        ),
    IntoRender::into_render({
        (d.status.clone() == "waiting")
            .then(|| {
                View::new(
    leptos::html::label()
        .child((
            IntoRender::into_render(t("密码", "Password")),
            leptos::html::input()
                .class("xcss-input")
                .name("activation_code")
                .r#type("password")
                .prop("value", move || password.get())
                .required(true)
                .maxlength("256")
                .autocomplete("off")
                .readonly(
                    ::leptos::prelude::IntoAttributeValue::into_attribute_value(move ||
                    busy.get()),
                )
                .on(
                    leptos::ev::input,
                    move |ev| password.set(event_target_value(&ev)),
                ),
        ))
        .class("xcss-form-field"),
)
            })
    }),
))
                                            })
                                    }
                                }),
                                leptos::html::div()
                                    .child((
                                        leptos::html::button()
                                            .child(
                                                #[allow(unused_braces)]
                                                {
                                                    IntoRender::into_render({
                                                        t("取消", "Cancel")
                                                    })
                                                },
                                            )
                                            .class("xcss-button")
                                            .r#type("button")
                                            .disabled(
                                                ::leptos::prelude::IntoAttributeValue::into_attribute_value(move ||
                                                busy.get()),
                                            )
                                            .on(
                                                leptos::ev::click,
                                                move |_| close.run(()),
                                            ),
                                        IntoRender::into_render({
                                            move || {
                                                detail
                                                    .get()
                                                    .is_some()
                                                    .then(|| {
                                                        View::new(
    leptos::html::button()
        .child(
            #[allow(unused_braces)]
            {
                IntoRender::into_render({
                    t("刷新配对状态", "Refresh pairing status")
                })
            },
        )
        .class("xcss-button")
        .r#type("button")
        .disabled(
            ::leptos::prelude::IntoAttributeValue::into_attribute_value(move || {
                busy.get()
            }),
        )
        .on(leptos::ev::click, move |_| inspect.run(())),
)
                                                    })
                                            }
                                        }),
                                        leptos::html::button()
                                            .child(
                                                #[allow(unused_braces)]
                                                {
                                                    IntoRender::into_render({
                                                        move || {
                                                            if busy.get() {
                                                                t("正在处理…", "Processing…")
                                                            } else if detail.get().is_some() {
                                                                t("确认设备并激活", "Confirm device and activate")
                                                            } else {
                                                                t("读取配对请求", "Read pairing request")
                                                            }
                                                        }
                                                    })
                                                },
                                            )
                                            .class("xcss-button")
                                            .r#type("submit")
                                            .disabled(
                                                ::leptos::prelude::IntoAttributeValue::into_attribute_value(move ||
                                                {
                                                    busy.get()
                                                        || detail
                                                            .get()
                                                            .is_some_and(|d| d.status.clone() != "waiting")
                                                }),
                                            ),
                                    ))
                                    .class("xcss-actions"),
                            ))
                            .aria_busy(
                                ::leptos::prelude::IntoAttributeValue::into_attribute_value(move ||
                                busy.get().to_string()),
                            )
                            .on(
                                leptos::ev::submit,
                                move |ev| {
                                    ev.prevent_default();
                                    if !crate::contracts::uuid(
                                        &json!(identity.get_untracked()),
                                    ) {
                                        return;
                                    }
                                    if let Some(d) = detail.get_untracked() {
                                        let body = ActivatePairing {
                                            request_id: d.request_id.clone(),
                                            activation_code: password.get_untracked(),
                                        };
                                        password.set(String::new());
                                        mutation_json::<
                                            Value,
                                        >(
                                            "/api/v1/xsoc/activate-admin".into(),
                                            "POST",
                                            body,
                                            busy,
                                            error,
                                            move |_| {
                                                close.run(());
                                                revision.update(|v| *v += 1)
                                            },
                                        );
                                    } else {
                                        inspect.run(());
                                    }
                                },
                            )
                    })
                })
                .build();
                props
            },
        )
    })
}
#[component]
fn InstanceDetails(identity: String, revision: RwSignal<u64>) -> impl IntoView {
    let path = format!("{INSTANCES}?instance_id={}", enc(&identity));
    let (data, error) =
        resource_typed::<InstancesPage>(Signal::derive(move || path.clone()), revision, 0);
    let draft = RwSignal::new(None::<String>);
    View::new(
    leptos::html::section()
        .child((
            IntoRender::into_render({
                move || {
                    View::new({
    #[allow(unreachable_code)] #[allow(unused_mut)] #[allow(clippy::let_and_return, clippy::let_unit_value, clippy::unit_arg)]
    component_view(
        #[allow(clippy::needless_borrows_for_generic_args)]
        &Failure,
        {
            let mut props = component_props_builder(&Failure)
                .error(#[allow(unused_braces)] { error })
                .message(
                    #[allow(unused_braces)]
                    {
                        if data.get().is_some() {
                            t(
                                "实例详情刷新失败，页面保留上次成功数据。",
                                "Instance details refresh failed. The last successful data is retained.",
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
                #[allow(unreachable_code)] #[allow(unused_mut)]
                #[allow(clippy::let_and_return, clippy::let_unit_value, clippy::unit_arg)]
                component_view(
                    #[allow(clippy::needless_borrows_for_generic_args)]
                    &Show,
                    {
                        let mut props = component_props_builder(
                                &Show,
                            )
                            .when(
                                #[allow(unused_braces)]
                                { move || data.get().is_none() && error.get().is_none() },
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
            },
            {
                #[allow(unreachable_code)] #[allow(unused_mut)]
                #[allow(clippy::let_and_return, clippy::let_unit_value, clippy::unit_arg)]
                component_view(
                    #[allow(clippy::needless_borrows_for_generic_args)]
                    &Show,
                    {
                        let mut props = component_props_builder(
                                &Show,
                            )
                            .when(
                                #[allow(unused_braces)]
                                {
                                    move || data.get().is_some_and(|v| v.instances.is_empty())
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
                                                        "所选实例已不存在，请返回实例列表。",
                                                        "The selected instance no longer exists. Return to the instance list.",
                                                    )
                                                })
                                            },
                                        )
                                        .class("xcss-empty")
                                })
                            })
                            .build();
                        props
                    },
                )
            },
            {
                #[allow(unreachable_code)] #[allow(unused_mut)]
                #[allow(clippy::let_and_return, clippy::let_unit_value, clippy::unit_arg)]
                component_view(
                    #[allow(clippy::needless_borrows_for_generic_args)]
                    &Show,
                    {
                        let mut props = component_props_builder(
                                &Show,
                            )
                            .when(
                                #[allow(unused_braces)]
                                {
                                    move || {
                                        data
                                            .get()
                                            .is_some_and(|v| {
                                                v.hosts.iter().any(|h| h.data_error.is_some())
                                            })
                                    }
                                },
                            )
                            .children({
                                ::leptos::children::ToChildren::to_children(move || {
                                    leptos::html::p()
                                        .child(
                                            #[allow(unused_braces)]
                                            {
                                                IntoRender::into_render({
                                                    t(
                                                        "该实例的监控数据无法读取。",
                                                        "Monitoring data for this instance is unavailable.",
                                                    )
                                                })
                                            },
                                        )
                                        .class("xcss-error")
                                        .role("alert")
                                })
                            })
                            .build();
                        props
                    },
                )
            },
            {
                #[allow(unreachable_code)] #[allow(unused_mut)]
                #[allow(clippy::let_and_return, clippy::let_unit_value, clippy::unit_arg)]
                component_view(
                    #[allow(clippy::needless_borrows_for_generic_args)]
                    &For,
                    {
                        let mut props = component_props_builder(
                                &For,
                            )
                            .each(
                                #[allow(unused_braces)]
                                {
                                    {
                                        move || data.get().map(|v| v.instances).unwrap_or_default()
                                    }
                                },
                            )
                            .key(#[allow(unused_braces)] { |v| v.request_id.clone() })
                            .children(
                                #[allow(unused_braces)]
                                {
                                    move |instance| {
                                        let request_id = instance.request_id.clone();
                                        let initial = instance.clone();
                                        let instance = Signal::derive(move || {
                                            data
                                                .get()
                                                .and_then(|v| {
                                                    v.instances.into_iter().find(|i| i.request_id == request_id)
                                                })
                                                .unwrap_or_else(|| initial.clone())
                                        });
                                        View::new({
    #[allow(unreachable_code)] #[allow(unused_mut)] #[allow(clippy::let_and_return, clippy::let_unit_value, clippy::unit_arg)]
    component_view(
        #[allow(clippy::needless_borrows_for_generic_args)]
        &InstanceBody,
        {
            let mut props = component_props_builder(
                    &InstanceBody,
                )
                .instance(#[allow(unused_braces)] { instance })
                .revision(#[allow(unused_braces)] { revision })
                .draft(#[allow(unused_braces)] { draft })
                .build();
            props
        },
    )
})
                                    }
                                },
                            )
                            .build();
                        props
                    },
                )
            },
        ))
        .class("xcss-content-stack")
        .aria_label(
            ::leptos::prelude::IntoAttributeValue::into_attribute_value(
                t("详细信息与设置", "Details and settings"),
            ),
        ),
)
}
#[component]
fn InstanceBody(
    instance: Signal<Instance>,
    revision: RwSignal<u64>,
    draft: RwSignal<Option<String>>,
) -> impl IntoView {
    let host_id = Signal::derive(move || instance.with(|i| i.instance_id.clone()));
    let overview = move || {
        View::new({
            #[allow(unreachable_code)]
            #[allow(unused_mut)]
            #[allow(clippy::let_and_return, clippy::let_unit_value, clippy::unit_arg)]
            component_view(
                #[allow(clippy::needless_borrows_for_generic_args)]
                &InstanceOverview,
                {
                    let mut props = component_props_builder(&InstanceOverview)
                        .instance(
                            #[allow(unused_braces)]
                            {
                                instance
                            },
                        )
                        .revision(
                            #[allow(unused_braces)]
                            {
                                revision
                            },
                        )
                        .draft(
                            #[allow(unused_braces)]
                            {
                                draft
                            },
                        )
                        .build();
                    props
                },
            )
        })
    };
    View::new({
        #[allow(unreachable_code)]
        #[allow(unused_mut)]
        #[allow(clippy::let_and_return, clippy::let_unit_value, clippy::unit_arg)]
        component_view(
            #[allow(clippy::needless_borrows_for_generic_args)]
            &For,
            {
                let mut props = component_props_builder(&For)
                .each(
                    #[allow(unused_braces)]
                    {
                        move || {
                            vec![(host_id.get(), instance.with(| i | i.status.clone()))]
                        }
                    },
                )
                .key(
                    #[allow(unused_braces)]
                    { |(id, status)| (id.clone(), status.clone()) },
                )
                .children(
                    #[allow(unused_braces)]
                    {
                        move |(id, status)| {
                            if status == "active" {
                                View::new({
    #[allow(unreachable_code)] #[allow(unused_mut)] #[allow(clippy::let_and_return, clippy::let_unit_value, clippy::unit_arg)]
    component_view(
        #[allow(clippy::needless_borrows_for_generic_args)]
        &host::HostWorkspace,
        {
            let mut props = component_props_builder(
                    &host::HostWorkspace,
                )
                .host_id(#[allow(unused_braces)] { id })
                .revision(#[allow(unused_braces)] { revision })
                .children({
                    ::leptos::children::ToChildren::to_children(move || IntoRender::into_render({
                        overview()
                    }))
                })
                .build();
            props
        },
    )
})
                                    .into_any()
                            } else {
                                View::new(
    leptos::html::section()
        .child((
            IntoRender::into_render({ overview() }),
            leptos::html::section()
                .child((
                    leptos::html::h3()
                        .child(
                            #[allow(unused_braces)]
                            {
                                IntoRender::into_render({
                                    t("监控状态", "Monitoring status")
                                })
                            },
                        ),
                    leptos::html::p()
                        .child(
                            #[allow(unused_braces)]
                            {
                                IntoRender::into_render({
                                    if status == "pending" {
                                        t(
                                            "实例尚未配对，完成客户端配对后将显示监控详情。",
                                            "This instance is not paired yet. Monitoring details will appear after client pairing.",
                                        )
                                    } else {
                                        t(
                                            "实例配对已取消。",
                                            "Instance pairing has been cancelled.",
                                        )
                                    }
                                })
                            },
                        ),
                ))
                .class("host-overview-section"),
        ))
        .class("xcss-content-panel host-overview")
        .aria_label(
            ::leptos::prelude::IntoAttributeValue::into_attribute_value(
                t("实例概览", "Instance overview"),
            ),
        ),
)
                                    .into_any()
                            }
                        }
                    },
                )
                .build();
                props
            },
        )
    })
}
#[component]
fn InstanceOverview(
    instance: Signal<Instance>,
    revision: RwSignal<u64>,
    draft: RwSignal<Option<String>>,
) -> impl IntoView {
    let busy = RwSignal::new(false);
    let error = RwSignal::new(None);
    View::new((
    leptos::html::section()
        .child(
            #[allow(unused_braces)]
            {
                leptos::html::dl()
                    .child((
                        leptos::html::dt()
                            .child(
                                #[allow(unused_braces)]
                                {
                                    IntoRender::into_render({
                                        t("账户名", "Account name")
                                    })
                                },
                            ),
                        leptos::html::dd()
                            .child(
                                #[allow(unused_braces)]
                                {
                                    IntoRender::into_render({
                                        move || instance.with(|i| i.display_name.clone())
                                    })
                                },
                            ),
                        leptos::html::dt()
                            .child(
                                #[allow(unused_braces)]
                                {
                                    IntoRender::into_render({
                                        t("账户", "Account")
                                    })
                                },
                            ),
                        leptos::html::dd()
                            .child(
                                #[allow(unused_braces)]
                                {
                                    leptos::html::code()
                                        .child(
                                            #[allow(unused_braces)]
                                            {
                                                IntoRender::into_render({
                                                    move || instance.with(|i| i.instance_id.clone())
                                                })
                                            },
                                        )
                                },
                            ),
                        leptos::html::dt()
                            .child(
                                #[allow(unused_braces)]
                                {
                                    IntoRender::into_render({
                                        t("密码", "Password")
                                    })
                                },
                            ),
                        leptos::html::dd()
                            .child(
                                #[allow(unused_braces)]
                                {
                                    leptos::html::code()
                                        .child(
                                            #[allow(unused_braces)]
                                            {
                                                IntoRender::into_render({
                                                    move || instance.with(|i| i.authorization_code.clone())
                                                })
                                            },
                                        )
                                },
                            ),
                        leptos::html::dt()
                            .child(
                                #[allow(unused_braces)]
                                {
                                    IntoRender::into_render({
                                        t("配对状态", "Pairing status")
                                    })
                                },
                            ),
                        leptos::html::dd()
                            .child(
                                #[allow(unused_braces)]
                                {
                                    IntoRender::into_render({
                                        move || pairing(&instance.with(|i| i.status.clone()))
                                    })
                                },
                            ),
                    ))
                    .class("host-detail-list")
            },
        )
        .aria_label(
            ::leptos::prelude::IntoAttributeValue::into_attribute_value(
                t("配对账户信息", "Pairing account information"),
            ),
        ),
    leptos::html::section()
        .child(
            #[allow(unused_braces)]
            {
                leptos::html::form()
                    .child((
                        leptos::html::label()
                            .child((
                                IntoRender::into_render({
                                    t("实例名称", "Instance name")
                                }),
                                leptos::html::input()
                                    .class("xcss-input")
                                    .name("display_name")
                                    .prop(
                                        "value",
                                        move || {
                                            draft
                                                .get()
                                                .unwrap_or_else(|| {
                                                    instance.with(|i| i.display_name.clone())
                                                })
                                        },
                                    )
                                    .required(true)
                                    .readonly(
                                        ::leptos::prelude::IntoAttributeValue::into_attribute_value(move ||
                                        busy.get()),
                                    )
                                    .on(
                                        leptos::ev::input,
                                        move |ev| draft.set(Some(event_target_value(&ev))),
                                    ),
                            ))
                            .class("xcss-form-field"),
                        IntoRender::into_render({
                            move || {
                                draft
                                    .get()
                                    .is_some_and(|n| !valid_name(&n))
                                    .then(|| {
                                        View::new(
    leptos::html::p()
        .child(
            #[allow(unused_braces)]
            {
                IntoRender::into_render({
                    t(
                        "名称须为 1–32 个字符，不能包含控制字符。",
                        "Use 1–32 characters without control characters.",
                    )
                })
            },
        )
        .role("alert"),
)
                                    })
                            }
                        }),
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
                                        .message(
                                            #[allow(unused_braces)]
                                            {
                                                t(
                                                    "实例名称未能保存，请重试。",
                                                    "The instance name could not be saved. Please retry.",
                                                )
                                            },
                                        )
                                        .build();
                                    props
                                },
                            )
                        },
                        leptos::html::div()
                            .child(
                                #[allow(unused_braces)]
                                {
                                    leptos::html::button()
                                        .child(
                                            #[allow(unused_braces)]
                                            {
                                                IntoRender::into_render({
                                                    t("保存名称", "Save name")
                                                })
                                            },
                                        )
                                        .class("xcss-button")
                                        .r#type("submit")
                                        .disabled(
                                            ::leptos::prelude::IntoAttributeValue::into_attribute_value(move ||
                                            {
                                                busy.get()
                                                    || draft
                                                        .get()
                                                        .is_none_or(|n| {
                                                            !valid_name(&n)
                                                                || n.trim() == instance.with(|i| i.display_name.clone())
                                                        })
                                            }),
                                        )
                                },
                            )
                            .class("xcss-actions"),
                    ))
                    .aria_busy(
                        ::leptos::prelude::IntoAttributeValue::into_attribute_value(move ||
                        busy.get().to_string()),
                    )
                    .on(
                        leptos::ev::submit,
                        move |ev| {
                            ev.prevent_default();
                            let name = draft
                                .get_untracked()
                                .unwrap_or_else(|| {
                                    instance.with_untracked(|i| i.display_name.clone())
                                })
                                .trim()
                                .to_string();
                            if valid_name(&name) {
                                mutation_json::<
                                    (),
                                >(
                                    format!(
                                        "{INSTANCES}/{}", instance.with_untracked(| i | i.request_id
                                        .clone())
                                    ),
                                    "PATCH",
                                    InstanceName { display_name: name },
                                    busy,
                                    error,
                                    move |_| {
                                        draft.set(None);
                                        revision.update(|v| *v += 1);
                                        xcss_web::components::notify(
                                            t("实例名称已保存", "Instance name saved"),
                                        );
                                    },
                                )
                            }
                        },
                    )
            },
        )
        .class("host-overview-section host-instance-settings")
        .aria_label(
            ::leptos::prelude::IntoAttributeValue::into_attribute_value(
                t("实例设置", "Instance settings"),
            ),
        ),
))
}
#[component]
fn Logs(identity: String, revision: RwSignal<u64>) -> impl IntoView {
    let focused = format!("{INSTANCES}?instance_id={}", enc(&identity));
    let (instance, error) = resource(Signal::derive(move || focused.clone()), revision, 0);
    View::new((
        {
            #[allow(unreachable_code)]
            #[allow(unused_mut)]
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
                                "所选实例刷新失败，页面保留上次成功读取的实例信息。",
                                "Selected instance refresh failed. The last successful instance information is retained.",
                            )
                        },
                    )
                    .build();
                    props
                },
            )
        },
        {
            #[allow(unreachable_code)]
            #[allow(unused_mut)]
            #[allow(clippy::let_and_return, clippy::let_unit_value, clippy::unit_arg)]
            component_view(
                #[allow(clippy::needless_borrows_for_generic_args)]
                &For,
                {
                    let mut props = component_props_builder(&For)
                        .each(
                            #[allow(unused_braces)]
                            {
                                {
                                    move || {
                                        instance
                                            .get()
                                            .map(|v| rows(&v, "hosts"))
                                            .unwrap_or_default()
                                    }
                                }
                            },
                        )
                        .key(
                            #[allow(unused_braces)]
                            {
                                |h| text(h, "id")
                            },
                        )
                        .children(
                            #[allow(unused_braces)]
                            {
                                move |host| {
                                    View::new({
                                        #[allow(unreachable_code)]
                                        #[allow(unused_mut)]
                                        #[allow(
                                            clippy::let_and_return,
                                            clippy::let_unit_value,
                                            clippy::unit_arg
                                        )]
                                        component_view(
                                            #[allow(clippy::needless_borrows_for_generic_args)]
                                            &HostLogs,
                                            {
                                                let mut props = component_props_builder(&HostLogs)
                                                    .host(
                                                        #[allow(unused_braces)]
                                                        {
                                                            host
                                                        },
                                                    )
                                                    .revision(
                                                        #[allow(unused_braces)]
                                                        {
                                                            revision
                                                        },
                                                    )
                                                    .build();
                                                props
                                            },
                                        )
                                    })
                                }
                            },
                        )
                        .build();
                    props
                },
            )
        },
    ))
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
    View::new(
        leptos::html::section()
            .child((
                leptos::html::div()
                    .child((
                        leptos::html::p().child(
                            #[allow(unused_braces)]
                            {
                                IntoRender::into_render({
                                    format!("{}{}", t("实例：", "Instance: "), text(&host, "name"))
                                })
                            },
                        ),
                        leptos::html::div()
                            .child((
                                leptos::html::label()
                                    .child(
                                        #[allow(unused_braces)]
                                        {
                                            IntoRender::into_render({
                                                t("日志日期范围（UTC）", "Log date range (UTC)")
                                            })
                                        },
                                    )
                                    .r#for("host-log-date-start-year"),
                                leptos::html::div()
                                    .child(
                                        #[allow(unused_braces)]
                                        {
                                            leptos::html::button()
                                                .child(
                                                    #[allow(unused_braces)]
                                                    {
                                                        IntoRender::into_render({
                                                            t("刷新日志", "Refresh logs")
                                                        })
                                                    },
                                                )
                                                .class("xcss-button")
                                                .on(leptos::ev::click, move |_| {
                                                    cursor.set(String::new());
                                                    local.update(|v| *v += 1)
                                                })
                                        },
                                    )
                                    .class("xcss-actions"),
                                {
                                    #[allow(unreachable_code)]
                                    #[allow(unused_mut)]
                                    #[allow(
                                        clippy::let_and_return,
                                        clippy::let_unit_value,
                                        clippy::unit_arg
                                    )]
                                    component_view(
                                        #[allow(clippy::needless_borrows_for_generic_args)]
                                        &DateRange,
                                        {
                                            let mut props = component_props_builder(&DateRange)
                                                .id(
                                                    #[allow(unused_braces)]
                                                    {
                                                        "host-log-date"
                                                    },
                                                )
                                                .value(
                                                    #[allow(unused_braces)]
                                                    {
                                                        range
                                                    },
                                                )
                                                .apply(
                                                    #[allow(unused_braces)]
                                                    {
                                                        Callback::new(move |r| {
                                                            range.set(r);
                                                            cursor.set(String::new());
                                                            local.update(|v| *v += 1);
                                                        })
                                                    },
                                                )
                                                .build();
                                            props
                                        },
                                    )
                                },
                            ))
                            .class("xcss-log-date-controls"),
                    ))
                    .class("xcss-content-panel xcss-content-stack host-logs-panel"),
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
                                        calendar_error
                                    },
                                )
                                .message(
                                    #[allow(unused_braces)]
                                    {
                                        t(
                                            "无法读取UTC 当天日期。",
                                            "Unable to load the current UTC date.",
                                        )
                                    },
                                )
                                .build();
                            props
                        },
                    )
                },
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
                                        error
                                    },
                                )
                                .message(
                                    #[allow(unused_braces)]
                                    {
                                        t("无法读取上报日志。", "Unable to load report logs.")
                                    },
                                )
                                .build();
                            props
                        },
                    )
                },
                IntoRender::into_render({
                    move || {
                        data.get().map(|d| {
                            View::new(
                                leptos::html::div()
                                    .child(
                                        #[allow(unused_braces)]
                                        {
                                            leptos::html::table()
                                                .child((
                                                    leptos::html::thead().child(
                                                        #[allow(unused_braces)]
                                                        {
                                                            leptos::html::tr()
                                        .child((
                                            leptos::html::th()
                                                .child(
                                                    #[allow(unused_braces)]
                                                    {
                                                        IntoRender::into_render({
                                                            t(
                                                                "服务端接收时间（UTC）",
                                                                "Server receipt time (UTC)",
                                                            )
                                                        })
                                                    },
                                                ),
                                            leptos::html::th()
                                                .child(
                                                    #[allow(unused_braces)]
                                                    {
                                                        IntoRender::into_render({
                                                            t(
                                                                "客户端采集时间（UTC）",
                                                                "Client collection time (UTC)",
                                                            )
                                                        })
                                                    },
                                                ),
                                            leptos::html::th()
                                                .child(
                                                    #[allow(unused_braces)]
                                                    {
                                                        IntoRender::into_render({
                                                            t("报告 ID", "Report ID")
                                                        })
                                                    },
                                                ),
                                        ))
                                                        },
                                                    ),
                                                    leptos::html::tbody().child(
                                                        #[allow(unused_braces)]
                                                        {
                                                            IntoRender::into_render({
                                                                rows(&d, "reports")
                                                                    .into_iter()
                                                                    .map(|r| {
                                                                        View::new(
    leptos::html::tr()
        .child((
            leptos::html::td()
                .child(
                    #[allow(unused_braces)]
                    {
                        IntoRender::into_render({
                            text(&r, "received_at")
                        })
                    },
                ),
            leptos::html::td()
                .child(
                    #[allow(unused_braces)]
                    {
                        IntoRender::into_render({
                            text(&r, "collected_at")
                        })
                    },
                ),
            leptos::html::td()
                .child(
                    #[allow(unused_braces)]
                    {
                        leptos::html::code()
                            .child(
                                #[allow(unused_braces)]
                                {
                                    IntoRender::into_render({
                                        text(&r, "report_id")
                                    })
                                },
                            )
                    },
                ),
        )),
)
                                                                    })
                                                                    .collect_view()
                                                            })
                                                        },
                                                    ),
                                                ))
                                                .class("xcss-table")
                                        },
                                    )
                                    .class("xcss-table-scroll"),
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
                        &Pagination,
                        {
                            let mut props = component_props_builder(&Pagination)
                                .data(
                                    #[allow(unused_braces)]
                                    {
                                        data
                                    },
                                )
                                .cursor(
                                    #[allow(unused_braces)]
                                    {
                                        cursor
                                    },
                                )
                                .revision(
                                    #[allow(unused_braces)]
                                    {
                                        local
                                    },
                                )
                                .label(
                                    #[allow(unused_braces)]
                                    {
                                        t("日志分页", "Log pages")
                                    },
                                )
                                .build();
                            props
                        },
                    )
                },
            ))
            .class("xcss-content-stack")
            .aria_label(::leptos::prelude::IntoAttributeValue::into_attribute_value(
                t("上报日志", "Report logs"),
            )),
    )
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
