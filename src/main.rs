// src/main.rs
mod studio;

use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use serde::{Deserialize, Serialize};
use std::fs;
use studio::{Material, Part, SavedGame, StudioState};
use chrono::Local; // <--- Used to get the actual local date of your PC

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
enum AppFlowState {
    #[default]
    Login,
    CreateAccount,
    Dashboard,
    Studio,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct User {
    username: String,
    password_hash: String,
    gender: String,
    birth_month: u32,
    birth_day: u32,
    birth_year: u32,
}

#[derive(PartialEq, Debug)]
enum DashboardTab {
    Home,
    VortoStudio,
    Friends,
    FriendsChat,
    Server,
    VortoTickets,
    Settings,
}

#[derive(Resource)]
struct VortoSystemState {
    users: Vec<User>,
    logged_in_user: Option<String>,
    recents: Vec<SavedGame>,

    // Fields
    username_input: String,
    password_input: String,
    retype_password: String,
    error_msg: String,

    gender_input: String,
    b_month: u32,
    b_day: u32,
    b_year: u32,

    captcha_solved: bool,
    captcha_question: String,
    captcha_answer: String,
    captcha_input: String,

    studio: StudioState,
    show_rename_dialog: bool,
    temp_rename_input: String,

    // Navigation and Panels
    current_tab: DashboardTab,
    search_query: String,
    settings_sub_tab: String, // "Privacy" or "Security"
    created_servers: Vec<String>,
    new_server_name: String,
}

impl Default for VortoSystemState {
    fn default() -> Self {
        let mut logged_in_user = None;
        let mut users = Vec::new();
        let mut recents = Vec::new();

        if let Ok(data) = fs::read_to_string("vorto_data.json") {
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&data) {
                users = serde_json::from_value(parsed["users"].clone()).unwrap_or_default();
                recents = serde_json::from_value(parsed["recents"].clone()).unwrap_or_default();
                if let Some(user) = parsed.get("logged_in_user").and_then(|u| u.as_str()) {
                    if !user.is_empty() {
                        logged_in_user = Some(user.to_string());
                    }
                }
            }
        }

        Self {
            users,
            logged_in_user,
            recents,
            username_input: String::new(),
            password_input: String::new(),
            retype_password: String::new(),
            error_msg: String::new(),
            gender_input: "Select Gender".to_string(),
            b_month: 1,
            b_day: 1,
            b_year: 2010,
            captcha_solved: false,
            captcha_question: "What is 7 + 8?".to_string(),
            captcha_answer: "15".to_string(),
            captcha_input: String::new(),
            studio: StudioState::default(),
            show_rename_dialog: false,
            temp_rename_input: String::new(),
            current_tab: DashboardTab::Home,
            search_query: String::new(),
            settings_sub_tab: "Privacy".to_string(),
            created_servers: vec!["Official Vorto Hub".to_string()],
            new_server_name: String::new(),
        }
    }
}

#[derive(Component)]
struct StudioCamera;

#[derive(Component)]
struct StudioBaseplate;

#[derive(Component)]
struct StudioLight;

#[derive(Component)]
struct StudioPartEntity {
    id: u32,
}

#[derive(Component)]
struct StudioHandle {
    axis_name: String,
}

#[derive(Resource)]
struct MeshCache {
    cube_mesh: Handle<Mesh>,
    torus_mesh: Handle<Mesh>,
    cylinder_mesh: Handle<Mesh>,
    handle_materials: std::collections::HashMap<String, Handle<StandardMaterial>>,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Vorto".into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(EguiPlugin)
        .init_state::<AppFlowState>()
        .insert_resource(VortoSystemState::default())
        .add_systems(Startup, (setup_3d_environment, evaluate_login_state))
        .add_systems(Update, (
            draw_2d_interface,
            sync_3d_scene.run_if(in_state(AppFlowState::Studio)),
            handle_studio_mouse_and_drag_inputs.run_if(in_state(AppFlowState::Studio)),
            handle_studio_hotkeys.run_if(in_state(AppFlowState::Studio)),
            draw_custom_gizmos.run_if(in_state(AppFlowState::Studio)),
            manage_handle_entities.run_if(in_state(AppFlowState::Studio)),
        ))
        .run();
}

fn evaluate_login_state(
    state: Res<VortoSystemState>,
    mut next_flow_state: ResMut<NextState<AppFlowState>>,
) {
    if state.logged_in_user.is_some() {
        next_flow_state.set(AppFlowState::Dashboard);
    }
}

fn setup_3d_environment(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let cube = meshes.add(Cuboid::new(1.0, 1.0, 1.0));
    let cylinder = meshes.add(Cylinder::new(0.04, 1.4));
    let torus = meshes.add(Torus::new(0.03, 1.8));

    let mut handle_mats = std::collections::HashMap::new();
    handle_mats.insert("red".to_string(), materials.add(StandardMaterial { base_color: Color::srgb(1.0, 0.1, 0.1), unlit: true, ..default() }));
    handle_mats.insert("green".to_string(), materials.add(StandardMaterial { base_color: Color::srgb(0.1, 1.0, 0.1), unlit: true, ..default() }));
    handle_mats.insert("blue".to_string(), materials.add(StandardMaterial { base_color: Color::srgb(0.1, 0.1, 1.0), unlit: true, ..default() }));
    handle_mats.insert("cyan".to_string(), materials.add(StandardMaterial { base_color: Color::srgb(0.1, 0.9, 0.9), unlit: true, ..default() }));
    handle_mats.insert("yellow".to_string(), materials.add(StandardMaterial { base_color: Color::srgb(0.9, 0.9, 0.1), unlit: true, ..default() }));
    handle_mats.insert("magenta".to_string(), materials.add(StandardMaterial { base_color: Color::srgb(0.9, 0.1, 0.9), unlit: true, ..default() }));

    commands.insert_resource(MeshCache {
        cube_mesh: cube,
        torus_mesh: torus,
        cylinder_mesh: cylinder,
        handle_materials: handle_mats,
    });

    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(0.0, 15.0, 25.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        StudioCamera,
    ));

    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Plane3d::default().mesh().size(100.0, 100.0)),
            material: materials.add(StandardMaterial {
                base_color: Color::srgb(0.3, 0.3, 0.3),
                ..default()
            }),
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            ..default()
        },
        StudioBaseplate,
    ));

    commands.spawn((
        DirectionalLightBundle {
            directional_light: DirectionalLight {
                shadows_enabled: true,
                ..default()
            },
            transform: Transform::from_xyz(10.0, 20.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        StudioLight,
    ));
}

fn draw_2d_interface(
    mut contexts: EguiContexts,
    mut state: ResMut<VortoSystemState>,
    current_flow_state: Res<State<AppFlowState>>,
    mut next_flow_state: ResMut<NextState<AppFlowState>>,
) {
    let ctx = contexts.ctx_mut();
    let mut visuals = egui::Visuals::dark();
    visuals.panel_fill = egui::Color32::from_rgb(11, 14, 18);
    visuals.window_fill = egui::Color32::from_rgb(18, 22, 28);
    ctx.set_visuals(visuals);

    match current_flow_state.get() {
        AppFlowState::Login => {
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                    ui.add_space(100.0);
                    ui.heading("VORTO");
                    ui.label("Interactive Virtual Experience Engine");
                    ui.add_space(20.0);

                    egui::Frame::window(ui.style()).show(ui, |ui| {
                        ui.set_width(320.0);
                        ui.vertical_centered(|ui| {
                            ui.heading("Log in");
                            ui.add_space(20.0);
                            ui.add(egui::TextEdit::singleline(&mut state.username_input).hint_text("Username"));
                            ui.add_space(10.0);
                            ui.add(egui::TextEdit::singleline(&mut state.password_input).password(true).hint_text("Password"));
                            ui.add_space(15.0);

                            if !state.error_msg.is_empty() {
                                ui.colored_label(egui::Color32::RED, &state.error_msg);
                            }

                            if ui.button("Log In").clicked() {
                                let found = state.users.iter().any(|u| {
                                    u.username.to_lowercase() == state.username_input.to_lowercase()
                                        && u.password_hash == state.password_input
                                });
                                if found {
                                    state.logged_in_user = Some(state.username_input.clone());
                                    state.error_msg.clear();
                                    state.password_input.clear();
                                    save_local_system(&state);
                                    next_flow_state.set(AppFlowState::Dashboard);
                                } else {
                                    state.error_msg = "Invalid credentials!".to_string();
                                }
                            }

                            ui.add_space(10.0);
                            if ui.link("Create Account").clicked() {
                                state.error_msg.clear();
                                next_flow_state.set(AppFlowState::CreateAccount);
                            }
                        });
                    });
                });
            });
        }

        AppFlowState::CreateAccount => {
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                    ui.add_space(50.0);
                    ui.heading("VORTO");
                    ui.add_space(15.0);

                    egui::Frame::window(ui.style()).show(ui, |ui| {
                        ui.set_width(350.0);
                        ui.vertical_centered(|ui| {
                            ui.heading("Create Account");
                            ui.add_space(15.0);
                            ui.add(egui::TextEdit::singleline(&mut state.username_input).hint_text("Username"));
                            ui.add_space(10.0);
                            ui.add(egui::TextEdit::singleline(&mut state.password_input).password(true).hint_text("Password"));
                            ui.add_space(10.0);
                            ui.add(egui::TextEdit::singleline(&mut state.retype_password).password(true).hint_text("Re-type Password"));
                            ui.add_space(15.0);

                            ui.horizontal(|ui| {
                                ui.label("Gender:");
                                egui::ComboBox::from_id_source("gender_combo")
                                    .selected_text(&state.gender_input)
                                    .show_ui(ui, |ui| {
                                        ui.selectable_value(&mut state.gender_input, "Male".to_string(), "Male");
                                        ui.selectable_value(&mut state.gender_input, "Female".to_string(), "Female");
                                        ui.selectable_value(&mut state.gender_input, "Other".to_string(), "Other");
                                    });
                            });

                            ui.horizontal(|ui| {
                                ui.label("Birthdate:");
                                egui::ComboBox::from_id_source("b_month")
                                    .selected_text(state.b_month.to_string())
                                    .show_ui(ui, |ui| {
                                        for m in 1..=12 { ui.selectable_value(&mut state.b_month, m, m.to_string()); }
                                    });
                                egui::ComboBox::from_id_source("b_day")
                                    .selected_text(state.b_day.to_string())
                                    .show_ui(ui, |ui| {
                                        for d in 1..=31 { ui.selectable_value(&mut state.b_day, d, d.to_string()); }
                                    });
                                egui::ComboBox::from_id_source("b_year")
                                    .selected_text(state.b_year.to_string())
                                    .show_ui(ui, |ui| {
                                        for y in (1950..=2026).rev() { ui.selectable_value(&mut state.b_year, y, y.to_string()); }
                                    });
                            });

                            ui.add_space(10.0);
                            ui.group(|ui| {
                                ui.label(format!("🛡 CAPTCHA: {}", state.captcha_question));
                                ui.add(egui::TextEdit::singleline(&mut state.captcha_input).hint_text("Your answer"));
                                if ui.button("Verify").clicked() {
                                    if state.captcha_input.trim() == state.captcha_answer {
                                        state.captcha_solved = true;
                                    } else {
                                        state.error_msg = "Incorrect answer!".to_string();
                                    }
                                }
                                if state.captcha_solved {
                                    ui.colored_label(egui::Color32::GREEN, "✔ Verified!");
                                }
                            });

                            if !state.error_msg.is_empty() {
                                ui.colored_label(egui::Color32::RED, &state.error_msg);
                            }

                            ui.add_space(15.0);
                            if ui.button("Register").clicked() {
                                if !state.captcha_solved {
                                    state.error_msg = "Please verify the CAPTCHA.".to_string();
                                } else if state.password_input != state.retype_password {
                                    state.error_msg = "Passwords do not match!".to_string();
                                } else {
                                    let new_user = User {
                                        username: state.username_input.clone(),
                                        password_hash: state.password_input.clone(),
                                        gender: state.gender_input.clone(),
                                        birth_month: state.b_month,
                                        birth_day: state.b_day,
                                        birth_year: state.b_year,
                                    };
                                    state.users.push(new_user);
                                    state.logged_in_user = Some(state.username_input.clone());
                                    save_local_system(&state);
                                    next_flow_state.set(AppFlowState::Dashboard);
                                }
                            }

                            ui.add_space(5.0);
                            if ui.link("Back to Login").clicked() {
                                next_flow_state.set(AppFlowState::Login);
                            }
                        });
                    });
                });
            });
        }

        AppFlowState::Dashboard => {
            egui::TopBottomPanel::top("vorto_dash_header").show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.strong("⚡ VORTO");
                    ui.add_space(ui.available_width() / 3.0);
                    ui.add(egui::TextEdit::singleline(&mut state.search_query).hint_text("🔍 Search Vorto..."));

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("Sign Out").clicked() {
                            state.logged_in_user = None;
                            save_local_system(&state);
                            next_flow_state.set(AppFlowState::Login);
                        }
                        ui.label(format!("👤 {}", state.logged_in_user.as_deref().unwrap_or("Guest")));
                    });
                });
            });

            egui::SidePanel::left("vorto_dash_side").show(ctx, |ui| {
                ui.add_space(15.0);
                if ui.selectable_label(state.current_tab == DashboardTab::Home, "🏠 Home").clicked() {
                    state.current_tab = DashboardTab::Home;
                }
                ui.add_space(8.0);
                if ui.selectable_label(state.current_tab == DashboardTab::VortoStudio, "🎬 VortoStudio").clicked() {
                    state.current_tab = DashboardTab::VortoStudio;
                }
                ui.add_space(8.0);
                if ui.selectable_label(state.current_tab == DashboardTab::Friends, "👥 Friends").clicked() {
                    state.current_tab = DashboardTab::Friends;
                }
                ui.add_space(8.0);
                if ui.selectable_label(state.current_tab == DashboardTab::FriendsChat, "💬 Friends Chat").clicked() {
                    state.current_tab = DashboardTab::FriendsChat;
                }
                ui.add_space(8.0);
                if ui.selectable_label(state.current_tab == DashboardTab::Server, "🌐 Server").clicked() {
                    state.current_tab = DashboardTab::Server;
                }
                ui.add_space(8.0);
                if ui.selectable_label(state.current_tab == DashboardTab::VortoTickets, "🎟️ VortoTickets").clicked() {
                    state.current_tab = DashboardTab::VortoTickets;
                }
                ui.add_space(8.0);
                if ui.selectable_label(state.current_tab == DashboardTab::Settings, "⚙️ Settings").clicked() {
                    state.current_tab = DashboardTab::Settings;
                }
            });

            egui::CentralPanel::default().show(ctx, |ui| {
                match state.current_tab {
                    DashboardTab::Home => {
                        ui.heading("Friends");
                        ui.group(|ui| {
                            ui.horizontal(|ui| {
                                ui.label("🟢 Online: PlayerX, StudioBuilder");
                                ui.label(" | 🔴 Offline: Friend_1");
                            });
                        });
                        ui.add_space(30.0);

                        ui.heading("Games");
                        ui.separator();
                        ui.horizontal(|ui| {
                            ui.group(|ui| {
                                ui.set_width(140.0);
                                ui.set_height(100.0);
                                ui.vertical_centered(|ui| {
                                    ui.label("🏁 Speed Obby");
                                    ui.add_space(15.0);
                                    if ui.button("▶ Play").clicked() {}
                                });
                            });
                            ui.group(|ui| {
                                ui.set_width(140.0);
                                ui.set_height(100.0);
                                ui.vertical_centered(|ui| {
                                    ui.label("🏗 Block Sandbox");
                                    ui.add_space(15.0);
                                    if ui.button("▶ Play").clicked() {}
                                });
                            });
                        });
                    }
                    DashboardTab::VortoStudio => {
                        ui.heading("VortoStudio Workspaces");
                        ui.add_space(15.0);

                        ui.label("Templates:");
                        ui.horizontal(|ui| {
                            ui.group(|ui| {
                                ui.set_width(160.0);
                                ui.set_height(120.0);
                                ui.vertical_centered(|ui| {
                                    ui.strong("Standard Grid Baseplate");
                                    ui.add_space(15.0);
                                    if ui.button("Launch Studio").clicked() {
                                        state.studio = StudioState::default();
                                        next_flow_state.set(AppFlowState::Studio);
                                    }
                                });
                            });
                        });

                        ui.add_space(30.0);
                        ui.heading("Recent Workspaces");
                        ui.separator();

                        if state.recents.is_empty() {
                            ui.label("No recent workspace experiences.");
                        } else {
                            let mut index_to_load = None;
                            for (idx, game) in state.recents.iter().enumerate() {
                                ui.horizontal(|ui| {
                                    ui.label(format!("📁 {} - {}", game.name, game.modified_date));
                                    if ui.button("Open").clicked() {
                                        index_to_load = Some(idx);
                                    }
                                });
                            }

                            if let Some(idx) = index_to_load {
                                let game = state.recents[idx].clone();
                                state.studio.current_game_name = game.name;
                                state.studio.parts = game.parts;
                                state.studio.baseplate_color = game.baseplate_color;
                                state.studio.baseplate_scale = game.baseplate_scale;
                                state.studio.next_id = state.studio.parts.iter().map(|p| p.id).max().unwrap_or(0) + 1;
                                next_flow_state.set(AppFlowState::Studio);
                            }
                        }
                    }
                    DashboardTab::Friends => {
                        ui.heading("Friends Center");
                        ui.separator();
                        ui.label("Enter a player username to add them as a friend:");
                        ui.text_edit_singleline(&mut String::new());
                        if ui.button("➕ Add Friend").clicked() {}
                    }
                    DashboardTab::FriendsChat => {
                        ui.heading("Friends Chat");
                        ui.separator();
                        ui.label("💬 Choose an active DM chain below:");
                        ui.group(|ui| {
                            ui.selectable_label(false, "💬 PlayerX: Let's test the map");
                        });
                    }
                    DashboardTab::Server => {
                        ui.heading("Group Creation & Server Portal");
                        ui.separator();
                        ui.horizontal(|ui| {
                            ui.add(egui::TextEdit::singleline(&mut state.new_server_name).hint_text("New group / server name..."));
                            if ui.button("➕ Create Group").clicked() {
                                if !state.new_server_name.trim().is_empty() {
                                    state.created_servers.push(state.new_server_name.clone());
                                    state.new_server_name.clear();
                                }
                            }
                        });
                        ui.add_space(15.0);
                        for server in &state.created_servers {
                            ui.label(format!("🌐 {}", server));
                        }
                    }
                    DashboardTab::VortoTickets => {
                        ui.heading("VortoTickets Support Board");
                        ui.separator();
                        ui.label("Open a ticket if you're experiencing bugs or platform issues:");
                        if ui.button("📨 Create New Ticket").clicked() {}
                    }
                    DashboardTab::Settings => {
                        ui.heading("Vorto Settings Panel");
                        ui.horizontal(|ui| {
                            if ui.selectable_label(state.settings_sub_tab == "Privacy", "🔒 Privacy").clicked() {
                                state.settings_sub_tab = "Privacy".to_string();
                            }
                            if ui.selectable_label(state.settings_sub_tab == "Security", "🛡️ Security").clicked() {
                                state.settings_sub_tab = "Security".to_string();
                            }
                        });
                        ui.separator();
                        if state.settings_sub_tab == "Privacy" {
                            ui.heading("Privacy Console");
                            ui.checkbox(&mut true, "Allow direct friend chat messages");
                            ui.checkbox(&mut false, "Show game status on profile");
                        } else {
                            ui.heading("Security Console");
                            ui.button("Reset Password");
                            ui.checkbox(&mut true, "Require CAPTCHA validations");
                        }
                    }
                }
            });
        }

        AppFlowState::Studio => {
            let mut exit_studio = false;
            let mut force_save_recents = false;
            egui::TopBottomPanel::top("studio_top_bar").show(ctx, |ui| {
                egui::menu::bar(ui, |ui| {
                    ui.menu_button("File", |ui| {
                        // Manual Quick Save (No popups, automatically gets current date)
                        if ui.button("💾 Quick Save (Ctrl+S)").clicked() {
                            manual_save_current_studio(&mut state);
                            ui.close_menu();
                        }
                        if ui.button("Save Workspace As...").clicked() {
                            state.temp_rename_input = state.studio.current_game_name.clone();
                            state.show_rename_dialog = true;
                            ui.close_menu();
                        }
                        if ui.button("Exit").clicked() {
                            exit_studio = true;
                            ui.close_menu();
                        }
                    });
                    ui.separator();
                    ui.label("Toolbox:");
                    ui.selectable_value(&mut state.studio.tool, "Select".to_string(), "🖐 Select");
                    ui.selectable_value(&mut state.studio.tool, "Move".to_string(), "🏹 Move");
                    ui.selectable_value(&mut state.studio.tool, "Scale".to_string(), "📦 Scale");
                    ui.selectable_value(&mut state.studio.tool, "Rotate".to_string(), "🔄 Rotate");
                });
            });

            if state.show_rename_dialog {
                egui::Window::new("Save Experience")
                    .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
                    .show(ctx, |ui| {
                        ui.text_edit_singleline(&mut state.temp_rename_input);
                        ui.horizontal(|ui| {
                            if ui.button("Confirm").clicked() {
                                if !state.temp_rename_input.trim().is_empty() {
                                    state.studio.current_game_name = state.temp_rename_input.trim().to_string();
                                }
                                force_save_recents = true;
                                state.show_rename_dialog = false;
                            }
                            if ui.button("Cancel").clicked() {
                                state.show_rename_dialog = false;
                            }
                        });
                    });
            }

            state.studio.render_ui(ctx, &mut exit_studio);

            if exit_studio || force_save_recents {
                let mut recents = state.recents.clone();
                // Get actual current calendar date dynamically using Chrono!
                let date = Local::now().format("%Y-%m-%d").to_string();
                let current_game = SavedGame {
                    name: state.studio.current_game_name.clone(),
                    modified_date: date,
                    parts: state.studio.parts.clone(),
                    baseplate_color: state.studio.baseplate_color,
                    baseplate_scale: state.studio.baseplate_scale,
                };
                recents.retain(|g| g.name != current_game.name);
                recents.insert(0, current_game);
                state.recents = recents;
                save_local_system(&state);
                if exit_studio {
                    next_flow_state.set(AppFlowState::Dashboard);
                }
            }
        }
    }
}

fn sync_3d_scene(
    mut commands: Commands,
    state: Res<VortoSystemState>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    cache: Res<MeshCache>,
    mut baseplate_query: Query<(&mut Transform, &mut Handle<StandardMaterial>), With<StudioBaseplate>>,
    mut part_entities: Query<(Entity, &mut Transform, &mut Handle<StandardMaterial>, &StudioPartEntity), (Without<StudioBaseplate>, Without<StudioHandle>)>,
) {
    let studio = &state.studio;
    if let Ok((mut transform, material_handle)) = baseplate_query.get_single_mut() {
        transform.scale = Vec3::from_slice(&studio.baseplate_scale);
        if let Some(mat) = materials.get_mut(material_handle.id()) {
            mat.base_color = Color::srgb(0.25, 0.28, 0.26);
        }
    }

    let mut current_ids = std::collections::HashSet::new();
    for part in &studio.parts {
        current_ids.insert(part.id);
        let mut found = false;
        for (_, mut transform, mat_handle, part_ent) in part_entities.iter_mut() {
            if part_ent.id == part.id {
                transform.translation = Vec3::from_slice(&part.position);
                transform.scale = Vec3::from_slice(&part.scale);
                transform.rotation = Quat::from_euler(EulerRot::XYZ, part.rotation[0], part.rotation[1], part.rotation[2]);
                if let Some(mat) = materials.get_mut(mat_handle.id()) {
                    mat.base_color = Color::srgb_u8(part.color[0], part.color[1], part.color[2]);
                    match part.material {
                        Material::Plastic => {
                            mat.perceptual_roughness = 0.5;
                            mat.emissive = LinearRgba::BLACK;
                        }
                        Material::SmoothPlastic => {
                            mat.perceptual_roughness = 0.1;
                            mat.emissive = LinearRgba::BLACK;
                        }
                        Material::Wood | Material::WoodPlanks => {
                            mat.perceptual_roughness = 0.9;
                            mat.emissive = LinearRgba::BLACK;
                        }
                        Material::Concrete => {
                            mat.perceptual_roughness = 0.8;
                            mat.emissive = LinearRgba::BLACK;
                        }
                        Material::Neon => {
                            mat.perceptual_roughness = 0.2;
                            mat.emissive = LinearRgba::rgb(
                                part.color[0] as f32 / 255.0,
                                part.color[1] as f32 / 255.0,
                                part.color[2] as f32 / 255.0
                            );
                        }
                    }
                }
                found = true;
                break;
            }
        }

        if !found {
            commands.spawn((
                PbrBundle {
                    mesh: cache.cube_mesh.clone(),
                    material: materials.add(StandardMaterial {
                        base_color: Color::srgb_u8(part.color[0], part.color[1], part.color[2]),
                        ..default()
                    }),
                    transform: Transform {
                        translation: Vec3::from_slice(&part.position),
                        scale: Vec3::from_slice(&part.scale),
                        rotation: Quat::from_euler(EulerRot::XYZ, part.rotation[0], part.rotation[1], part.rotation[2]),
                    },
                    ..default()
                },
                StudioPartEntity { id: part.id },
            ));
        }
    }

    for (entity, _, _, part_ent) in part_entities.iter() {
        if !current_ids.contains(&part_ent.id) {
            commands.entity(entity).despawn_recursive();
        }
    }
}

fn handle_studio_mouse_and_drag_inputs(
    mouse: Res<ButtonInput<MouseButton>>,
    mut state: ResMut<VortoSystemState>,
) {
    let studio = &mut state.studio;
    if mouse.pressed(MouseButton::Left) {
        studio.is_dragging_block_directly = true;
    } else {
        studio.active_drag_handle = None;
        studio.drag_start_mouse_pos = None;
        studio.is_dragging_block_directly = false;
    }
}

fn handle_studio_hotkeys(
    keys: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<VortoSystemState>,
) {
    let studio = &mut state.studio;
    let ctrl_held = keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight);

    // 1. DUPLICATE: Ctrl + D
    if ctrl_held && keys.just_pressed(KeyCode::KeyD) {
        if let Some(selected_id) = studio.selected_part_id {
            if let Some(part) = studio.parts.iter().find(|p| p.id == selected_id).cloned() {
                let duplicated_part = Part {
                    id: studio.next_id,
                    name: format!("{} (Clone)", part.name),
                    position: [part.position[0] + part.scale[0], part.position[1], part.position[2]],
                    scale: part.scale,
                    rotation: part.rotation,
                    color: part.color,
                    material: part.material,
                };
                studio.parts.push(duplicated_part);
                studio.selected_part_id = Some(studio.next_id);
                studio.next_id += 1;
            }
        }
    }

    // 2. QUICK SAVE: Ctrl + S
    if ctrl_held && keys.just_pressed(KeyCode::KeyS) {
        manual_save_current_studio(&mut state);
    }
}

// Manually save the currently open studio session instantly with real-world calendar date
fn manual_save_current_studio(state: &mut VortoSystemState) {
    // Dynamically fetches current local computer date on save!
    let date = Local::now().format("%Y-%m-%d").to_string();
    let current_game = SavedGame {
        name: state.studio.current_game_name.clone(),
        modified_date: date,
        parts: state.studio.parts.clone(),
        baseplate_color: state.studio.baseplate_color,
        baseplate_scale: state.studio.baseplate_scale,
    };

    let mut recents = state.recents.clone();
    recents.retain(|g| g.name != current_game.name);
    recents.insert(0, current_game);

    state.recents = recents;
    save_local_system(state);
}

fn save_local_system(state: &VortoSystemState) {
    let json_data = serde_json::json!({
        "users": state.users,
        "recents": state.recents,
        "logged_in_user": state.logged_in_user
    });
    let _ = fs::write("vorto_data.json", json_data.to_string());
}

fn draw_custom_gizmos() {}
fn manage_handle_entities() {}