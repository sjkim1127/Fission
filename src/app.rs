//! Application state management for the Fission GUI.
//!
//! This module contains the main application struct that holds all UI state,
//! debugger state, and manages the egui rendering loop.

use eframe::egui;

/// Main application state container
pub struct FissionApp {
    /// Log buffer for the output console
    log_buffer: Vec<String>,

    /// Current command input in the integrated CLI
    cli_input: String,

    /// Currently loaded binary path (if any)
    loaded_binary: Option<String>,

    /// Debugger running state
    is_debugging: bool,
}

impl Default for FissionApp {
    fn default() -> Self {
        Self {
            log_buffer: vec![
                "╔══════════════════════════════════════════════════════════════╗".into(),
                "║  Fission - Next-Gen Dynamic Instrumentation Platform         ║".into(),
                "║  \"Split the Binary, Fuse the Power.\"                         ║".into(),
                "╚══════════════════════════════════════════════════════════════╝".into(),
                "".into(),
                "[*] Ready. Load a binary to begin analysis.".into(),
            ],
            cli_input: String::new(),
            loaded_binary: None,
            is_debugging: false,
        }
    }
}

impl FissionApp {
    /// Add a log message to the output buffer
    pub fn log(&mut self, message: impl Into<String>) {
        self.log_buffer.push(message.into());
    }

    /// Clear the log buffer
    pub fn clear_logs(&mut self) {
        self.log_buffer.clear();
    }

    /// Render the top menu bar
    fn render_menu_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Open Binary...").clicked() {
                        self.log("[*] Open file dialog (TODO)");
                        ui.close_menu();
                    }
                    if ui.button("Exit").clicked() {
                        std::process::exit(0);
                    }
                });

                ui.menu_button("Debug", |ui| {
                    let attach_text = if self.is_debugging {
                        "Detach"
                    } else {
                        "Attach"
                    };
                    if ui.button(attach_text).clicked() {
                        self.is_debugging = !self.is_debugging;
                        let status = if self.is_debugging {
                            "attached"
                        } else {
                            "detached"
                        };
                        self.log(format!("[*] Debugger {}", status));
                        ui.close_menu();
                    }
                });

                ui.menu_button("View", |ui| {
                    if ui.button("Clear Console").clicked() {
                        self.clear_logs();
                        self.log("[*] Console cleared");
                        ui.close_menu();
                    }
                });

                ui.menu_button("Help", |ui| {
                    if ui.button("About").clicked() {
                        self.log("[*] Fission v0.1.0 - Hybrid Analysis Platform");
                        ui.close_menu();
                    }
                });
            });
        });
    }

    /// Render the status bar at the bottom
    fn render_status_bar(&self, ctx: &egui::Context) {
        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                // Debugger status indicator
                let (status_color, status_text) = if self.is_debugging {
                    (egui::Color32::from_rgb(100, 200, 100), "● DEBUGGING")
                } else {
                    (egui::Color32::from_rgb(150, 150, 150), "○ IDLE")
                };
                ui.colored_label(status_color, status_text);

                ui.separator();

                // Loaded binary info
                if let Some(ref path) = self.loaded_binary {
                    ui.label(format!("📁 {}", path));
                } else {
                    ui.label("No binary loaded");
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label("Fission v0.1.0");
                });
            });
        });
    }

    /// Render the main content area
    fn render_main_content(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // Main split: Left panel (disasm) | Right panel (logs)
            egui::SidePanel::left("disasm_panel")
                .resizable(true)
                .default_width(500.0)
                .show_inside(ui, |ui| {
                    ui.heading("📋 Disassembly");
                    ui.separator();

                    egui::ScrollArea::vertical().show(ui, |ui| {
                        ui.monospace("Address    Bytes            Instruction");
                        ui.monospace("─────────────────────────────────────────");
                        ui.monospace("0x00401000 55               push rbp");
                        ui.monospace("0x00401001 48 89 E5         mov rbp, rsp");
                        ui.monospace("0x00401004 48 83 EC 20      sub rsp, 0x20");
                        ui.monospace("...");
                        ui.monospace("");
                        ui.label("(Load a binary to see actual disassembly)");
                    });
                });

            // Log/Console panel (remaining space)
            ui.heading("📜 Console");
            ui.separator();

            // Scrollable log area
            let text_style = egui::TextStyle::Monospace;
            let row_height = ui.text_style_height(&text_style);

            egui::ScrollArea::vertical()
                .auto_shrink([false; 2])
                .stick_to_bottom(true)
                .max_height(ui.available_height() - 30.0)
                .show_rows(ui, row_height, self.log_buffer.len(), |ui, row_range| {
                    for row in row_range {
                        if let Some(log) = self.log_buffer.get(row) {
                            ui.monospace(log);
                        }
                    }
                });

            ui.separator();

            // CLI input at the bottom
            ui.horizontal(|ui| {
                ui.label(">");
                let response = ui.add(
                    egui::TextEdit::singleline(&mut self.cli_input)
                        .desired_width(ui.available_width() - 60.0)
                        .font(egui::TextStyle::Monospace)
                        .hint_text("Enter command..."),
                );

                if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    let cmd = self.cli_input.trim().to_string();
                    if !cmd.is_empty() {
                        self.log(format!("> {}", cmd));
                        self.process_command(&cmd);
                        self.cli_input.clear();
                    }
                    response.request_focus();
                }

                if ui.button("Run").clicked() {
                    let cmd = self.cli_input.trim().to_string();
                    if !cmd.is_empty() {
                        self.log(format!("> {}", cmd));
                        self.process_command(&cmd);
                        self.cli_input.clear();
                    }
                }
            });
        });
    }

    /// Process a CLI command entered in the GUI
    fn process_command(&mut self, cmd: &str) {
        match cmd {
            "help" | "?" => {
                self.log("Available commands:");
                self.log("  load <path>  : Load a binary for analysis");
                self.log("  start        : Start debugging session");
                self.log("  stop         : Stop debugging session");
                self.log("  regs         : Show registers");
                self.log("  clear        : Clear console");
                self.log("  exit         : Quit Fission");
            }
            "clear" => {
                self.clear_logs();
                self.log("[*] Console cleared");
            }
            "exit" | "quit" | "q" => {
                std::process::exit(0);
            }
            "regs" => {
                self.log("Registers (stub):");
                self.log("  RAX: 0x0000000000000000");
                self.log("  RBX: 0x0000000000000000");
                self.log("  RCX: 0x0000000000000000");
                self.log("  RDX: 0x0000000000000000");
            }
            "start" => {
                if self.loaded_binary.is_some() {
                    self.is_debugging = true;
                    self.log("[*] Debugging session started");
                } else {
                    self.log("[!] No binary loaded. Use 'load <path>' first.");
                }
            }
            "stop" => {
                self.is_debugging = false;
                self.log("[*] Debugging session stopped");
            }
            _ if cmd.starts_with("load ") => {
                let path = cmd.strip_prefix("load ").unwrap().trim();
                self.loaded_binary = Some(path.to_string());
                self.log(format!("[*] Loaded: {}", path));
            }
            _ => {
                self.log(format!("[!] Unknown command: '{}'", cmd));
                self.log("    Type 'help' for available commands.");
            }
        }
    }
}

impl eframe::App for FissionApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.render_menu_bar(ctx);
        self.render_status_bar(ctx);
        self.render_main_content(ctx);
    }
}
