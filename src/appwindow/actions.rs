use std::time::Duration;

use gpui_kit::*;

use super::{AppWindow, Notice};
use crate::theme::{self, ThemePreset};
use crate::wg::{self, Snapshot, Tunnel};

impl AppWindow {
    pub(super) fn change_theme(&mut self, preset: ThemePreset, cx: &mut Context<Self>) {
        theme::apply(preset, cx);
        self.theme_preset = preset;
        self.theme_error = theme::save(preset).err();
        cx.notify();
    }

    pub(super) fn apply_snapshot(&mut self, snapshot: Snapshot, _cx: &mut Context<Self>) {
        self.tunnels = snapshot.tunnels;
        self.live = snapshot.live;
        self.privilege = snapshot.privilege;
        self.system_error = snapshot.system_error;
        self.system_requires_privilege = snapshot.system_requires_privilege;
        self.wg_available = snapshot.wg_available;
        self.wg_quick_available = snapshot.wg_quick_available;
        self.user_dir = snapshot.user_dir;
        self.last_refresh = wg::now_epoch();

        let selection_still_exists = self
            .selected
            .as_deref()
            .map(|name| self.tunnels.iter().any(|tunnel| tunnel.name == name))
            .unwrap_or(false);
        if !selection_still_exists {
            self.selected = self.tunnels.first().map(|tunnel| tunnel.name.clone());
        }
    }

    pub(super) fn start_refresh_loop(&mut self, cx: &mut Context<Self>) {
        cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor().timer(Duration::from_secs(5)).await;
                let Ok((system_configs, generation)) = this.update(cx, |this, _| {
                    (this.system_configs.clone(), this.system_configs_generation)
                }) else {
                    break;
                };
                let snapshot = cx
                    .background_spawn(async move { wg::snapshot(system_configs) })
                    .await;
                let alive = this
                    .update(cx, |this, cx| {
                        if this.system_configs_generation == generation {
                            this.apply_snapshot(snapshot, cx);
                            cx.notify();
                        }
                    })
                    .is_ok();
                if !alive {
                    break;
                }
            }
        })
        .detach();
    }

    pub(super) fn refresh_now(&mut self, cx: &mut Context<Self>) {
        let system_configs = self.system_configs.clone();
        let generation = self.system_configs_generation;
        cx.spawn(async move |this, cx| {
            let snapshot = cx
                .background_spawn(async move { wg::snapshot(system_configs) })
                .await;
            this.update(cx, |this, cx| {
                if this.system_configs_generation == generation {
                    this.apply_snapshot(snapshot, cx);
                    cx.notify();
                }
            })
            .ok();
        })
        .detach();
    }

    pub(super) fn unlock_system_configs(&mut self, cx: &mut Context<Self>) {
        if self.unlocking_system_configs {
            return;
        }
        self.unlocking_system_configs = true;
        self.system_auth_error = None;
        self.notice = Some(Notice::info("Requesting access to system configurations…"));
        cx.notify();

        cx.spawn(async move |this, cx| {
            let result = cx
                .background_spawn(async { wg::read_system_configs() })
                .await;
            this.update(cx, |this, cx| {
                this.unlocking_system_configs = false;
                match result {
                    Ok(tunnels) => {
                        this.system_configs = Some(tunnels);
                        this.system_configs_generation += 1;
                        this.notice = Some(Notice::success("System configurations loaded"));
                        this.refresh_now(cx);
                    }
                    Err(error) => {
                        this.system_auth_error = Some(error.clone());
                        this.notice = Some(Notice::error(error));
                    }
                }
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    pub(super) fn toggle(&mut self, name: &str, cx: &mut Context<Self>) {
        if self.busy.is_some() {
            return;
        }
        let Some(tunnel) = self
            .tunnels
            .iter()
            .find(|tunnel| tunnel.name == name)
            .cloned()
        else {
            return;
        };
        let up = !self.live.get(name).map(|live| live.up).unwrap_or(false);
        let privilege = self.privilege;

        self.busy = Some(name.to_owned());
        self.notice = Some(Notice::info(if up {
            format!("Connecting \"{name}\"…")
        } else {
            format!("Disconnecting \"{name}\"…")
        }));
        cx.notify();

        cx.spawn(async move |this, cx| {
            let result = cx
                .background_spawn(async move { wg::set_tunnel(&tunnel, up, privilege) })
                .await;
            this.update(cx, |this, cx| {
                this.busy = None;
                this.notice = Some(if result.ok {
                    Notice::success(result.message)
                } else {
                    Notice::error(result.message)
                });
                cx.notify();
                this.refresh_now(cx);
            })
            .ok();
        })
        .detach();
    }

    pub(super) fn import(&mut self, cx: &mut Context<Self>) {
        if self.busy.is_some() {
            return;
        }
        let raw = self.import_input.read(cx).value().to_string();
        let path = wg::expand_path(&raw);
        self.busy = Some("__import".into());
        self.notice = Some(Notice::info("Importing configuration…"));
        cx.notify();

        cx.spawn(async move |this, cx| {
            let result = cx
                .background_spawn(async move { wg::import_config(&path) })
                .await;
            this.update(cx, |this, cx| {
                this.busy = None;
                this.notice = Some(if result.ok {
                    Notice::success(result.message.clone())
                } else {
                    Notice::error(result.message.clone())
                });
                if let Some(name) = result.name {
                    this.selected = Some(name);
                }
                cx.notify();
                this.refresh_now(cx);
            })
            .ok();
        })
        .detach();
    }

    pub(super) fn remove(&mut self, name: &str, cx: &mut Context<Self>) {
        if self.busy.is_some() {
            return;
        }
        let Some(tunnel) = self
            .tunnels
            .iter()
            .find(|tunnel| tunnel.name == name)
            .cloned()
        else {
            return;
        };
        self.busy = Some(name.to_owned());
        self.notice = Some(Notice::info(format!("Removing \"{name}\"…")));
        cx.notify();

        cx.spawn(async move |this, cx| {
            let result = cx
                .background_spawn(async move { wg::delete_config(&tunnel) })
                .await;
            this.update(cx, |this, cx| {
                this.busy = None;
                this.notice = Some(if result.ok {
                    Notice::success(result.message)
                } else {
                    Notice::error(result.message)
                });
                if result.ok {
                    this.selected = None;
                }
                cx.notify();
                this.refresh_now(cx);
            })
            .ok();
        })
        .detach();
    }

    pub(super) fn selected_tunnel(&self) -> Option<&Tunnel> {
        let name = self.selected.as_deref()?;
        self.tunnels.iter().find(|tunnel| tunnel.name == name)
    }
}
