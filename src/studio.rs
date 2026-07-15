// src/studio.rs
use bevy_egui::egui;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub enum Material {
    Plastic,
    SmoothPlastic,
    Wood,
    WoodPlanks,
    Concrete,
    Neon,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Part {
    pub id: u32,
    pub name: String,
    pub position: [f32; 3],
    pub scale: [f32; 3],
    pub rotation: [f32; 3], // Euler angles (radians)
    pub color: [u8; 3],
    pub material: Material,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SavedGame {
    pub name: String,
    pub modified_date: String,
    pub parts: Vec<Part>,
    pub baseplate_color: [u8; 3],
    pub baseplate_scale: [f32; 3],
}

pub struct StudioState {
    pub current_game_name: String,
    pub parts: Vec<Part>,
    pub selected_part_id: Option<u32>,
    pub next_id: u32,
    pub baseplate_color: [u8; 3],
    pub baseplate_scale: [f32; 3],
    
    // Tools: "Select", "Move", "Scale", "Rotate"
    pub tool: String,

    // Interaction & Dragging States
    pub active_drag_handle: Option<String>, 
    pub drag_start_mouse_pos: Option<[f32; 2]>,
    pub drag_start_part_pos: Option<[f32; 3]>,
    pub drag_start_part_scale: Option<[f32; 3]>,
    pub drag_start_part_rotation: Option<[f32; 3]>,
    
    pub is_dragging_block_directly: bool,
    pub air_distance: f32,

    // Camera State
    pub camera_yaw: f32,
    pub camera_pitch: f32,
    pub camera_distance: f32,
    pub camera_target: [f32; 3],
}

impl Default for StudioState {
    fn default() -> Self {
        Self {
            current_game_name: "Unsaved Vorto Experience".to_string(),
            parts: Vec::new(),
            selected_part_id: None,
            next_id: 1,
            baseplate_color: [80, 80, 80],
            baseplate_scale: [100.0, 1.0, 100.0],
            tool: "Select".to_string(),
            active_drag_handle: None,
            drag_start_mouse_pos: None,
            drag_start_part_pos: None,
            drag_start_part_scale: None,
            drag_start_part_rotation: None,
            is_dragging_block_directly: false,
            air_distance: 15.0,
            camera_yaw: 0.0,
            camera_pitch: -0.4,
            camera_distance: 30.0,
            camera_target: [0.0, 0.0, 0.0],
        }
    }
}

impl StudioState {
    pub fn render_ui(&mut self, ctx: &egui::Context, exit_studio: &mut bool) {
        // --- Top Panel: Toolbar ---
        egui::TopBottomPanel::top("studio_top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading(&self.current_game_name);
                ui.separator();
                
                ui.label("Tools:");
                let tools = ["Select", "Move", "Scale", "Rotate"];
                for tool_name in tools {
                    let is_active = self.tool == tool_name;
                    if ui.selectable_label(is_active, tool_name).clicked() {
                        self.tool = tool_name.to_string();
                    }
                }
            });
        });

        // --- Left Sidebar: Explorer ---
        egui::SidePanel::left("studio_left_panel").show(ctx, |ui| {
            ui.heading("Explorer");
            ui.separator();

            if ui.button("➕ Insert Block").clicked() {
                let new_part = Part {
                    id: self.next_id,
                    name: format!("Part {}", self.next_id),
                    position: [0.0, 0.5, 0.0],
                    scale: [2.0, 2.0, 2.0],
                    rotation: [0.0, 0.0, 0.0],
                    color: [200, 200, 200],
                    material: Material::Plastic,
                };
                self.parts.push(new_part);
                self.selected_part_id = Some(self.next_id);
                self.next_id += 1;
            }

            ui.add_space(10.0);
            
            ui.horizontal(|ui| {
                let is_baseplate_selected = self.selected_part_id.is_none();
                if ui.selectable_label(is_baseplate_selected, "🏁 Baseplate").clicked() {
                    self.selected_part_id = None; // Deselect to focus baseplate
                }
            });

            ui.add_space(5.0);
            ui.label("Hierarchy:");
            
            egui::ScrollArea::vertical().show(ui, |ui| {
                let mut clicked_part_id = None;
                for part in &self.parts {
                    let is_selected = Some(part.id) == self.selected_part_id;
                    if ui.selectable_label(is_selected, format!("📦 {}", part.name)).clicked() {
                        clicked_part_id = Some(part.id);
                    }
                }
                if let Some(id) = clicked_part_id {
                    self.selected_part_id = Some(id);
                }
            });

            ui.add_space(20.0);
            if ui.button("🚪 Exit to Dashboard").clicked() {
                *exit_studio = true;
            }
        });

        // --- Right Sidebar: Properties ---
        egui::SidePanel::right("studio_right_panel").show(ctx, |ui| {
            ui.heading("Properties");
            ui.separator();

            if let Some(selected_id) = self.selected_part_id {
                let mut index_to_remove = None;
                if let Some((idx, part)) = self.parts.iter_mut().enumerate().find(|(_, p)| p.id == selected_id) {
                    
                    ui.horizontal(|ui| {
                        ui.label("Name:");
                        ui.text_edit_singleline(&mut part.name);
                    });

                    ui.add_space(8.0);
                    ui.label("Position (X, Y, Z):");
                    ui.horizontal(|ui| {
                        ui.add(egui::DragValue::new(&mut part.position[0]).speed(0.1).prefix("X: "));
                        ui.add(egui::DragValue::new(&mut part.position[1]).speed(0.1).prefix("Y: "));
                        ui.add(egui::DragValue::new(&mut part.position[2]).speed(0.1).prefix("Z: "));
                    });

                    ui.add_space(8.0);
                    ui.label("Size / Scale (Width, Height, Depth):");
                    ui.horizontal(|ui| {
                        // FIXED WARNINGS: Replaced obsolete `.clamp_range` with modern `.range`
                        ui.add(egui::DragValue::new(&mut part.scale[0]).speed(0.1).prefix("X: ").range(0.01..=f32::MAX));
                        ui.add(egui::DragValue::new(&mut part.scale[1]).speed(0.1).prefix("Y: ").range(0.01..=f32::MAX));
                        ui.add(egui::DragValue::new(&mut part.scale[2]).speed(0.1).prefix("Z: ").range(0.01..=f32::MAX));
                    });

                    ui.add_space(8.0);
                    ui.label("Rotation (Degrees):");
                    ui.horizontal(|ui| {
                        let mut rx = part.rotation[0].to_degrees();
                        let mut ry = part.rotation[1].to_degrees();
                        let mut rz = part.rotation[2].to_degrees();
                        if ui.add(egui::DragValue::new(&mut rx).speed(1.0).suffix("°")).changed() {
                            part.rotation[0] = rx.to_radians();
                        }
                        if ui.add(egui::DragValue::new(&mut ry).speed(1.0).suffix("°")).changed() {
                            part.rotation[1] = ry.to_radians();
                        }
                        if ui.add(egui::DragValue::new(&mut rz).speed(1.0).suffix("°")).changed() {
                            part.rotation[2] = rz.to_radians();
                        }
                    });

                    ui.add_space(8.0);
                    ui.label("Material:");
                    egui::ComboBox::from_id_source("material_combo")
                        .selected_text(format!("{:?}", part.material))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut part.material, Material::Plastic, "Plastic");
                            ui.selectable_value(&mut part.material, Material::SmoothPlastic, "Smooth Plastic");
                            ui.selectable_value(&mut part.material, Material::Wood, "Wood");
                            ui.selectable_value(&mut part.material, Material::WoodPlanks, "Wood Planks");
                            ui.selectable_value(&mut part.material, Material::Concrete, "Concrete");
                            ui.selectable_value(&mut part.material, Material::Neon, "Neon");
                        });

                    ui.add_space(8.0);
                    ui.label("Color:");
                    let mut color_f32 = [
                        part.color[0] as f32 / 255.0,
                        part.color[1] as f32 / 255.0,
                        part.color[2] as f32 / 255.0,
                    ];
                    if ui.color_edit_button_rgb(&mut color_f32).changed() {
                        part.color[0] = (color_f32[0] * 255.0) as u8;
                        part.color[1] = (color_f32[1] * 255.0) as u8;
                        part.color[2] = (color_f32[2] * 255.0) as u8;
                    }

                    ui.add_space(15.0);
                    if ui.button("🗑 Delete Part").clicked() {
                        index_to_remove = Some(idx);
                    }
                }

                if let Some(idx) = index_to_remove {
                    self.parts.remove(idx);
                    self.selected_part_id = None;
                }
            } else {
                // Baseplate Properties (Shown when nothing else is selected)
                ui.label("🏁 Baseplate Configuration");
                ui.separator();

                ui.add_space(8.0);
                ui.label("Baseplate Scale (Width, Height, Depth):");
                ui.horizontal(|ui| {
                    // FIXED WARNINGS: Replaced obsolete `.clamp_range` with modern `.range`
                    ui.add(egui::DragValue::new(&mut self.baseplate_scale[0]).speed(0.5).prefix("X: ").range(1.0..=f32::MAX));
                    ui.add(egui::DragValue::new(&mut self.baseplate_scale[1]).speed(0.1).prefix("Y: ").range(0.1..=f32::MAX));
                    ui.add(egui::DragValue::new(&mut self.baseplate_scale[2]).speed(0.5).prefix("Z: ").range(1.0..=f32::MAX));
                });

                ui.add_space(8.0);
                ui.label("Baseplate Color:");
                let mut bp_color_f32 = [
                    self.baseplate_color[0] as f32 / 255.0,
                    self.baseplate_color[1] as f32 / 255.0,
                    self.baseplate_color[2] as f32 / 255.0,
                ];
                if ui.color_edit_button_rgb(&mut bp_color_f32).changed() {
                    self.baseplate_color[0] = (bp_color_f32[0] * 255.0) as u8;
                    self.baseplate_color[1] = (bp_color_f32[1] * 255.0) as u8;
                    self.baseplate_color[2] = (bp_color_f32[2] * 255.0) as u8;
                }
            }
        });
    }
}
