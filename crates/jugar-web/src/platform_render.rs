//! Rendering and HUD logic for PongGame.
//!
//! Extracted from platform.rs for file health compliance.

use super::{
    ButtonRect, Color, GameMode, GameState, PongAI, PongGame, RenderFrame, SpeedMultiplier,
    TextAlign, TextBaseline,
};

impl PongGame {
    /// Handles mouse clicks on HUD buttons.
    pub(super) fn handle_hud_click(&mut self, mx: f32, my: f32) {
        // If info panel is visible, clicking anywhere dismisses it
        if self.show_model_info {
            self.show_model_info = false;
            return; // Consume the click
        }

        // Check mode buttons
        if self.hud_buttons.mode_demo.contains(mx, my) {
            self.game_mode = GameMode::Demo;
            self.reset_game();
            self.state = GameState::Playing;
            self.audio.on_game_start();
        } else if self.hud_buttons.mode_1p.contains(mx, my) {
            self.game_mode = GameMode::SinglePlayer;
            self.reset_game();
            self.state = GameState::Playing;
            self.audio.on_game_start();
        } else if self.hud_buttons.mode_2p.contains(mx, my) {
            self.game_mode = GameMode::TwoPlayer;
            self.reset_game();
            self.state = GameState::Playing;
            self.audio.on_game_start();
        }
        // Check speed buttons
        else if self.hud_buttons.speed_1x.contains(mx, my) {
            self.speed_multiplier = SpeedMultiplier::Normal;
        } else if self.hud_buttons.speed_5x.contains(mx, my) {
            self.speed_multiplier = SpeedMultiplier::Fast5x;
        } else if self.hud_buttons.speed_10x.contains(mx, my) {
            self.speed_multiplier = SpeedMultiplier::Fast10x;
        } else if self.hud_buttons.speed_50x.contains(mx, my) {
            self.speed_multiplier = SpeedMultiplier::Fast50x;
        } else if self.hud_buttons.speed_100x.contains(mx, my) {
            self.speed_multiplier = SpeedMultiplier::Fast100x;
        } else if self.hud_buttons.speed_1000x.contains(mx, my) {
            self.speed_multiplier = SpeedMultiplier::Fast1000x;
        }
        // Check AI difficulty buttons
        else if self.hud_buttons.ai_decrease.contains(mx, my) {
            if let Some(ref mut ai) = self.ai {
                let level = ai.difficulty();
                if level > 0 {
                    ai.set_difficulty(level - 1);
                }
            }
        } else if self.hud_buttons.ai_increase.contains(mx, my) {
            if let Some(ref mut ai) = self.ai {
                let level = ai.difficulty();
                if level < 9 {
                    ai.set_difficulty(level + 1);
                }
            }
        }
        // Check download button
        else if self.hud_buttons.download.contains(mx, my) {
            self.download_requested = true;
        }
        // Check model info button
        else if self.hud_buttons.model_info.contains(mx, my) {
            self.show_model_info = !self.show_model_info;
        }
        // Check sound toggle button
        else if self.hud_buttons.sound_toggle.contains(mx, my) {
            self.sound_enabled = !self.sound_enabled;
            // Play confirmation sound when enabling audio (provides immediate feedback)
            self.audio.on_sound_toggle(self.sound_enabled);
        }
    }

    /// Renders the HUD (mode buttons, speed buttons, AI info, download button).
    pub(super) fn render_hud(&mut self, frame: &mut RenderFrame) {
        self.render_hud_mode_buttons(frame);
        self.render_hud_speed_buttons(frame);
        self.render_hud_ai_difficulty(frame);
        self.render_hud_footer_buttons(frame);
        self.render_ai_explain_widget(frame);
        if self.show_model_info {
            self.render_model_info_panel(frame);
        }
        self.render_hud_footer_links(frame);
    }

    /// Render game mode buttons (Demo, 1P, 2P) in the top-left.
    fn render_hud_mode_buttons(&mut self, frame: &mut RenderFrame) {
        let hud_y = 10.0;
        let button_height = 28.0;
        let button_padding = 8.0;
        let font_size = "14px monospace";
        let small_font = "12px monospace";
        let mut mode_x = 10.0;

        let render_mode_button =
            |frame: &mut RenderFrame, x: f32, label: &str, is_selected: bool| -> ButtonRect {
                let bw = (label.len() as f32).mul_add(10.0, button_padding * 2.0);
                let bg_color = if is_selected {
                    Color::new(0.3, 0.6, 1.0, 0.9)
                } else {
                    Color::new(0.2, 0.2, 0.2, 0.8)
                };
                let border_color = if is_selected {
                    Color::WHITE
                } else {
                    Color::new(0.5, 0.5, 0.5, 1.0)
                };
                frame.fill_rect(x, hud_y, bw, button_height, bg_color);
                frame.stroke_rect(x, hud_y, bw, button_height, border_color, 1.0);
                frame.fill_text_aligned(
                    label,
                    x + bw / 2.0,
                    hud_y + button_height / 2.0,
                    font_size,
                    Color::WHITE,
                    TextAlign::Center,
                    TextBaseline::Middle,
                );
                ButtonRect::new(x, hud_y, bw, button_height)
            };

        self.hud_buttons.mode_demo =
            render_mode_button(frame, mode_x, "Demo", self.game_mode == GameMode::Demo);
        mode_x += self.hud_buttons.mode_demo.width + 5.0;

        self.hud_buttons.mode_1p = render_mode_button(
            frame,
            mode_x,
            "1P",
            self.game_mode == GameMode::SinglePlayer,
        );
        mode_x += self.hud_buttons.mode_1p.width + 5.0;

        self.hud_buttons.mode_2p =
            render_mode_button(frame, mode_x, "2P", self.game_mode == GameMode::TwoPlayer);
        mode_x += self.hud_buttons.mode_2p.width + 5.0;

        frame.fill_text_aligned(
            "[M]",
            mode_x + 5.0,
            hud_y + button_height / 2.0,
            small_font,
            Color::new(0.5, 0.5, 0.5, 0.8),
            TextAlign::Left,
            TextBaseline::Middle,
        );
    }

    /// Render speed multiplier buttons (1x..1000x) in the top-right.
    fn render_hud_speed_buttons(&mut self, frame: &mut RenderFrame) {
        let hud_y = 10.0;
        let button_height = 28.0;
        let button_padding = 8.0;
        let font_size = "14px monospace";

        let render_speed_button = |frame: &mut RenderFrame,
                                   x: f32,
                                   label: &str,
                                   key_hint: &str,
                                   is_selected: bool|
         -> ButtonRect {
            let bw = (label.len() as f32).mul_add(8.0, button_padding * 2.0);
            let bg_color = if is_selected {
                Color::new(1.0, 0.6, 0.2, 0.9)
            } else {
                Color::new(0.2, 0.2, 0.2, 0.8)
            };
            let border_color = if is_selected {
                Color::WHITE
            } else {
                Color::new(0.5, 0.5, 0.5, 1.0)
            };
            frame.fill_rect(x, hud_y, bw, button_height, bg_color);
            frame.stroke_rect(x, hud_y, bw, button_height, border_color, 1.0);
            frame.fill_text_aligned(
                label,
                x + bw / 2.0,
                hud_y + button_height / 2.0,
                font_size,
                Color::WHITE,
                TextAlign::Center,
                TextBaseline::Middle,
            );
            frame.fill_text_aligned(
                key_hint,
                x + bw / 2.0,
                hud_y + button_height + 3.0,
                "10px monospace",
                Color::new(0.4, 0.4, 0.4, 0.7),
                TextAlign::Center,
                TextBaseline::Top,
            );
            ButtonRect::new(x, hud_y, bw, button_height)
        };

        let w1x = ("1x".len() as f32).mul_add(8.0, button_padding * 2.0);
        let w5x = ("5x".len() as f32).mul_add(8.0, button_padding * 2.0);
        let w10x = ("10x".len() as f32).mul_add(8.0, button_padding * 2.0);
        let w50x = ("50x".len() as f32).mul_add(8.0, button_padding * 2.0);
        let w100x = ("100x".len() as f32).mul_add(8.0, button_padding * 2.0);
        let w1000x = ("1000x".len() as f32).mul_add(8.0, button_padding * 2.0);
        let total_width = w1x + w5x + w10x + w50x + w100x + w1000x + 25.0;

        let mut speed_x = self.width - total_width - 10.0;

        self.hud_buttons.speed_1x = render_speed_button(
            frame,
            speed_x,
            "1x",
            "1",
            self.speed_multiplier == SpeedMultiplier::Normal,
        );
        speed_x += w1x + 5.0;

        self.hud_buttons.speed_5x = render_speed_button(
            frame,
            speed_x,
            "5x",
            "2",
            self.speed_multiplier == SpeedMultiplier::Fast5x,
        );
        speed_x += w5x + 5.0;

        self.hud_buttons.speed_10x = render_speed_button(
            frame,
            speed_x,
            "10x",
            "3",
            self.speed_multiplier == SpeedMultiplier::Fast10x,
        );
        speed_x += w10x + 5.0;

        self.hud_buttons.speed_50x = render_speed_button(
            frame,
            speed_x,
            "50x",
            "4",
            self.speed_multiplier == SpeedMultiplier::Fast50x,
        );
        speed_x += w50x + 5.0;

        self.hud_buttons.speed_100x = render_speed_button(
            frame,
            speed_x,
            "100x",
            "5",
            self.speed_multiplier == SpeedMultiplier::Fast100x,
        );
        speed_x += w100x + 5.0;

        self.hud_buttons.speed_1000x = render_speed_button(
            frame,
            speed_x,
            "1000x",
            "6",
            self.speed_multiplier == SpeedMultiplier::Fast1000x,
        );
    }

    /// Render AI difficulty indicator with progress bar and +/- buttons.
    #[allow(clippy::too_many_lines)]
    fn render_hud_ai_difficulty(&mut self, frame: &mut RenderFrame) {
        let hud_y = 10.0;
        let button_height = 28.0;
        let font_size = "14px monospace";
        let small_font = "12px monospace";

        let ai_y = hud_y + button_height + 15.0;
        let ai_level = self.ai.as_ref().map_or(5, crate::ai::PongAI::difficulty);
        let ai_name = self
            .ai
            .as_ref()
            .map_or("Normal", crate::ai::PongAI::difficulty_name);

        frame.fill_text_aligned(
            "AI:",
            10.0,
            ai_y,
            font_size,
            Color::new(0.7, 0.7, 0.7, 1.0),
            TextAlign::Left,
            TextBaseline::Top,
        );

        let bar_x = 40.0;
        let bar_width = 100.0;
        let bar_height = 12.0;
        frame.fill_rect(
            bar_x,
            ai_y + 2.0,
            bar_width,
            bar_height,
            Color::new(0.2, 0.2, 0.2, 0.8),
        );

        let fill_width = (f32::from(ai_level) / 9.0) * bar_width;
        let fill_color = match ai_level {
            0..=2 => Color::new(0.2, 0.8, 0.2, 1.0),
            3..=5 => Color::new(0.8, 0.8, 0.2, 1.0),
            6..=7 => Color::new(0.8, 0.5, 0.2, 1.0),
            _ => Color::new(0.8, 0.2, 0.2, 1.0),
        };
        frame.fill_rect(bar_x, ai_y + 2.0, fill_width, bar_height, fill_color);
        frame.stroke_rect(
            bar_x,
            ai_y + 2.0,
            bar_width,
            bar_height,
            Color::new(0.5, 0.5, 0.5, 1.0),
            1.0,
        );

        let ai_text = format!("{ai_level}/9 {ai_name}");
        frame.fill_text_aligned(
            &ai_text,
            bar_x + bar_width + 10.0,
            ai_y,
            small_font,
            Color::new(0.7, 0.7, 0.7, 1.0),
            TextAlign::Left,
            TextBaseline::Top,
        );

        let ai_btn_size = 20.0;
        let ai_btn_y = ai_y - 2.0;
        let ai_btn_x = bar_x + bar_width + 100.0;

        frame.fill_rect(
            ai_btn_x,
            ai_btn_y,
            ai_btn_size,
            ai_btn_size,
            Color::new(0.3, 0.3, 0.3, 0.9),
        );
        frame.stroke_rect(
            ai_btn_x,
            ai_btn_y,
            ai_btn_size,
            ai_btn_size,
            Color::new(0.5, 0.5, 0.5, 1.0),
            1.0,
        );
        frame.fill_text_aligned(
            "-",
            ai_btn_x + ai_btn_size / 2.0,
            ai_btn_y + ai_btn_size / 2.0,
            font_size,
            Color::WHITE,
            TextAlign::Center,
            TextBaseline::Middle,
        );
        self.hud_buttons.ai_decrease =
            ButtonRect::new(ai_btn_x, ai_btn_y, ai_btn_size, ai_btn_size);

        let ai_plus_x = ai_btn_x + ai_btn_size + 5.0;
        frame.fill_rect(
            ai_plus_x,
            ai_btn_y,
            ai_btn_size,
            ai_btn_size,
            Color::new(0.3, 0.3, 0.3, 0.9),
        );
        frame.stroke_rect(
            ai_plus_x,
            ai_btn_y,
            ai_btn_size,
            ai_btn_size,
            Color::new(0.5, 0.5, 0.5, 1.0),
            1.0,
        );
        frame.fill_text_aligned(
            "+",
            ai_plus_x + ai_btn_size / 2.0,
            ai_btn_y + ai_btn_size / 2.0,
            font_size,
            Color::WHITE,
            TextAlign::Center,
            TextBaseline::Middle,
        );
        self.hud_buttons.ai_increase =
            ButtonRect::new(ai_plus_x, ai_btn_y, ai_btn_size, ai_btn_size);
    }

    /// Render bottom-left footer buttons (Download, Info, Sound).
    #[allow(clippy::too_many_lines)]
    fn render_hud_footer_buttons(&mut self, frame: &mut RenderFrame) {
        let button_height = 28.0;
        let button_padding = 8.0;
        let font_size = "14px monospace";

        let download_y = self.height - 45.0;
        let download_text = "Download .apr";
        let download_width = (download_text.len() as f32).mul_add(8.0, button_padding * 2.0);

        frame.fill_rect(
            10.0,
            download_y,
            download_width,
            button_height,
            Color::new(0.1, 0.4, 0.2, 0.8),
        );
        frame.stroke_rect(
            10.0,
            download_y,
            download_width,
            button_height,
            Color::new(0.3, 0.7, 0.4, 1.0),
            1.0,
        );
        self.hud_buttons.download =
            ButtonRect::new(10.0, download_y, download_width, button_height);
        frame.fill_text_aligned(
            download_text,
            10.0 + download_width / 2.0,
            download_y + button_height / 2.0,
            font_size,
            Color::WHITE,
            TextAlign::Center,
            TextBaseline::Middle,
        );

        let info_text = "Info";
        let info_width = (info_text.len() as f32).mul_add(8.0, button_padding * 2.0);
        let info_x = 10.0 + download_width + 5.0;

        let info_bg = if self.show_model_info {
            Color::new(0.3, 0.3, 0.7, 0.9)
        } else {
            Color::new(0.2, 0.2, 0.4, 0.8)
        };
        frame.fill_rect(info_x, download_y, info_width, button_height, info_bg);
        frame.stroke_rect(
            info_x,
            download_y,
            info_width,
            button_height,
            Color::new(0.4, 0.4, 0.8, 1.0),
            1.0,
        );
        self.hud_buttons.model_info =
            ButtonRect::new(info_x, download_y, info_width, button_height);
        frame.fill_text_aligned(
            info_text,
            info_x + info_width / 2.0,
            download_y + button_height / 2.0,
            font_size,
            Color::WHITE,
            TextAlign::Center,
            TextBaseline::Middle,
        );
        frame.fill_text_aligned(
            "[I]",
            info_x + info_width + 5.0,
            download_y + button_height / 2.0,
            "10px monospace",
            Color::new(0.4, 0.4, 0.4, 0.7),
            TextAlign::Left,
            TextBaseline::Middle,
        );

        let sound_text = if self.sound_enabled { "Sound" } else { "Muted" };
        let sound_width = 64.0;
        let sound_x = info_x + info_width + 25.0;

        let sound_bg = if self.sound_enabled {
            Color::new(0.2, 0.4, 0.2, 0.8)
        } else {
            Color::new(0.4, 0.2, 0.2, 0.8)
        };
        frame.fill_rect(sound_x, download_y, sound_width, button_height, sound_bg);

        let sound_border = if self.sound_enabled {
            Color::new(0.3, 0.7, 0.3, 1.0)
        } else {
            Color::new(0.7, 0.3, 0.3, 1.0)
        };
        frame.stroke_rect(
            sound_x,
            download_y,
            sound_width,
            button_height,
            sound_border,
            1.0,
        );
        self.hud_buttons.sound_toggle =
            ButtonRect::new(sound_x, download_y, sound_width, button_height);
        frame.fill_text_aligned(
            sound_text,
            sound_x + sound_width / 2.0,
            download_y + button_height / 2.0,
            font_size,
            Color::WHITE,
            TextAlign::Center,
            TextBaseline::Middle,
        );
    }

    /// Render footer links (bottom-right).
    fn render_hud_footer_links(&self, frame: &mut RenderFrame) {
        let small_font = "12px monospace";
        let footer_x = self.width - 10.0;
        let footer_y = self.height - 10.0;
        let link_color = Color::new(0.4, 0.4, 0.4, 0.7);
        let link_spacing = 14.0;

        frame.fill_text_aligned(
            "paiml.com",
            footer_x,
            footer_y - link_spacing * 2.0,
            small_font,
            link_color,
            TextAlign::Right,
            TextBaseline::Bottom,
        );
        frame.fill_text_aligned(
            "github.com/paiml/jugar",
            footer_x,
            footer_y - link_spacing,
            small_font,
            link_color,
            TextAlign::Right,
            TextBaseline::Bottom,
        );
        frame.fill_text_aligned(
            ".apr format: github.com/paiml/aprender",
            footer_x,
            footer_y,
            small_font,
            link_color,
            TextAlign::Right,
            TextBaseline::Bottom,
        );
    }

    /// Renders the model info panel showing .apr metadata.
    ///
    /// Panel displays:
    /// - Model name, version, author, license
    /// - Flow Theory parameters
    /// - Current AI difficulty and state
    /// - Difficulty curve visualization
    #[allow(clippy::too_many_lines)]
    fn render_model_info_panel(&self, frame: &mut RenderFrame) {
        // Panel dimensions - centered, semi-transparent overlay
        let panel_width = 320.0;
        let panel_height = 340.0;
        let panel_x = (self.width - panel_width) / 2.0;
        let panel_y = (self.height - panel_height) / 2.0;

        // Semi-transparent dark background
        frame.fill_rect(
            panel_x,
            panel_y,
            panel_width,
            panel_height,
            Color::new(0.05, 0.05, 0.1, 0.95),
        );

        // Panel border
        frame.stroke_rect(
            panel_x,
            panel_y,
            panel_width,
            panel_height,
            Color::new(0.3, 0.3, 0.8, 1.0),
            2.0,
        );

        let title_font = "16px monospace";
        let label_font = "12px monospace";
        let value_font = "14px monospace";

        let text_color = Color::WHITE;
        let label_color = Color::new(0.6, 0.6, 0.8, 1.0);
        let highlight_color = Color::new(0.4, 0.8, 1.0, 1.0);

        let mut y = panel_y + 25.0;
        let left_margin = panel_x + 15.0;
        let value_x = panel_x + 140.0;
        let line_height = 22.0;

        // Title
        frame.fill_text_aligned(
            ".apr Model Info",
            panel_x + panel_width / 2.0,
            y,
            title_font,
            highlight_color,
            TextAlign::Center,
            TextBaseline::Middle,
        );
        y += line_height + 8.0;

        // Separator line
        frame.fill_rect(
            panel_x + 10.0,
            y,
            panel_width - 20.0,
            1.0,
            Color::new(0.3, 0.3, 0.5, 0.8),
        );
        y += 12.0;

        // Get model info from AI (or use defaults)
        let ai = self.ai.as_ref();
        let default_model = crate::ai::PongAIModel::default();
        let model = ai.map_or(&default_model, crate::ai::PongAI::model);

        // === Metadata Section ===
        frame.fill_text_aligned(
            "METADATA",
            left_margin,
            y,
            label_font,
            label_color,
            TextAlign::Left,
            TextBaseline::Middle,
        );
        y += line_height;

        // Model name
        frame.fill_text_aligned(
            "Model:",
            left_margin,
            y,
            label_font,
            label_color,
            TextAlign::Left,
            TextBaseline::Middle,
        );
        frame.fill_text_aligned(
            &model.metadata.name,
            value_x,
            y,
            value_font,
            text_color,
            TextAlign::Left,
            TextBaseline::Middle,
        );
        y += line_height;

        // Version
        frame.fill_text_aligned(
            "Version:",
            left_margin,
            y,
            label_font,
            label_color,
            TextAlign::Left,
            TextBaseline::Middle,
        );
        frame.fill_text_aligned(
            &model.metadata.version,
            value_x,
            y,
            value_font,
            text_color,
            TextAlign::Left,
            TextBaseline::Middle,
        );
        y += line_height;

        // Author
        frame.fill_text_aligned(
            "Author:",
            left_margin,
            y,
            label_font,
            label_color,
            TextAlign::Left,
            TextBaseline::Middle,
        );
        frame.fill_text_aligned(
            &model.metadata.author,
            value_x,
            y,
            value_font,
            text_color,
            TextAlign::Left,
            TextBaseline::Middle,
        );
        y += line_height;

        // License
        frame.fill_text_aligned(
            "License:",
            left_margin,
            y,
            label_font,
            label_color,
            TextAlign::Left,
            TextBaseline::Middle,
        );
        frame.fill_text_aligned(
            &model.metadata.license,
            value_x,
            y,
            value_font,
            text_color,
            TextAlign::Left,
            TextBaseline::Middle,
        );
        y += line_height + 8.0;

        // === Flow Theory Section ===
        frame.fill_text_aligned(
            "FLOW THEORY (DDA)",
            left_margin,
            y,
            label_font,
            label_color,
            TextAlign::Left,
            TextBaseline::Middle,
        );
        y += line_height;

        // Adaptation rate
        frame.fill_text_aligned(
            "Adapt Rate:",
            left_margin,
            y,
            label_font,
            label_color,
            TextAlign::Left,
            TextBaseline::Middle,
        );
        let adapt_text = format!("{:.0}%", model.flow_theory.adaptation_rate * 100.0);
        frame.fill_text_aligned(
            &adapt_text,
            value_x,
            y,
            value_font,
            text_color,
            TextAlign::Left,
            TextBaseline::Middle,
        );
        y += line_height;

        // Target win rate
        frame.fill_text_aligned(
            "Target Win:",
            left_margin,
            y,
            label_font,
            label_color,
            TextAlign::Left,
            TextBaseline::Middle,
        );
        let target_text = format!("{:.0}%", model.flow_theory.target_win_rate * 100.0);
        frame.fill_text_aligned(
            &target_text,
            value_x,
            y,
            value_font,
            text_color,
            TextAlign::Left,
            TextBaseline::Middle,
        );
        y += line_height;

        // Flow thresholds
        frame.fill_text_aligned(
            "Flow Range:",
            left_margin,
            y,
            label_font,
            label_color,
            TextAlign::Left,
            TextBaseline::Middle,
        );
        let range_text = format!(
            "{:.0}%-{:.0}%",
            model.flow_theory.anxiety_threshold * 100.0,
            model.flow_theory.boredom_threshold * 100.0
        );
        frame.fill_text_aligned(
            &range_text,
            value_x,
            y,
            value_font,
            text_color,
            TextAlign::Left,
            TextBaseline::Middle,
        );
        y += line_height + 8.0;

        // === Current State Section ===
        frame.fill_text_aligned(
            "CURRENT STATE",
            left_margin,
            y,
            label_font,
            label_color,
            TextAlign::Left,
            TextBaseline::Middle,
        );
        y += line_height;

        // AI difficulty level
        let ai_level = ai.map_or(5, crate::ai::PongAI::difficulty);
        let ai_name = ai.map_or("Normal", crate::ai::PongAI::difficulty_name);
        frame.fill_text_aligned(
            "Difficulty:",
            left_margin,
            y,
            label_font,
            label_color,
            TextAlign::Left,
            TextBaseline::Middle,
        );
        let level_text = format!("{ai_level}/9 {ai_name}");
        frame.fill_text_aligned(
            &level_text,
            value_x,
            y,
            value_font,
            highlight_color,
            TextAlign::Left,
            TextBaseline::Middle,
        );
        y += line_height;

        // Difficulty bar visualization
        let bar_x = left_margin;
        let bar_width = panel_width - 30.0;
        let bar_height = 12.0;

        // Background
        frame.fill_rect(
            bar_x,
            y,
            bar_width,
            bar_height,
            Color::new(0.15, 0.15, 0.2, 1.0),
        );

        // Fill based on level
        let fill_width = (f32::from(ai_level) / 9.0) * bar_width;
        let fill_color = match ai_level {
            0..=2 => Color::new(0.2, 0.8, 0.2, 1.0), // Green (easy)
            3..=5 => Color::new(0.8, 0.8, 0.2, 1.0), // Yellow (medium)
            6..=7 => Color::new(0.8, 0.5, 0.2, 1.0), // Orange (hard)
            _ => Color::new(0.8, 0.2, 0.2, 1.0),     // Red (expert)
        };
        frame.fill_rect(bar_x, y, fill_width, bar_height, fill_color);

        // Bar border
        frame.stroke_rect(
            bar_x,
            y,
            bar_width,
            bar_height,
            Color::new(0.4, 0.4, 0.6, 1.0),
            1.0,
        );
        y += bar_height + 8.0;

        // File size
        frame.fill_text_aligned(
            "File Size:",
            left_margin,
            y,
            label_font,
            label_color,
            TextAlign::Left,
            TextBaseline::Middle,
        );
        let size_text = format!("{} bytes", model.serialized_size());
        frame.fill_text_aligned(
            &size_text,
            value_x,
            y,
            value_font,
            text_color,
            TextAlign::Left,
            TextBaseline::Middle,
        );

        // Close hint at bottom
        frame.fill_text_aligned(
            "Press [I] or click Info to close",
            panel_x + panel_width / 2.0,
            panel_y + panel_height - 15.0,
            "10px monospace",
            Color::new(0.5, 0.5, 0.5, 0.8),
            TextAlign::Center,
            TextBaseline::Middle,
        );
    }

    /// Renders the AI explainability widget in the upper-right corner.
    ///
    /// Shows real-time SHAP-style feature contributions from the `.apr` model:
    /// - Current decision state (IDLE, REACT, TRACK, READY)
    /// - Feature contribution bars
    /// - Decision rationale
    fn render_ai_explain_widget(&self, frame: &mut RenderFrame) {
        // Only show when AI is active (Demo or SinglePlayer mode)
        if !matches!(self.game_mode, GameMode::Demo | GameMode::SinglePlayer) {
            return;
        }

        // In Demo mode, show widgets for both AIs
        if self.game_mode == GameMode::Demo {
            // Left AI widget (upper-left)
            if let Some(left_ai) = self.left_ai.as_ref() {
                self.render_single_ai_widget(frame, left_ai, 10.0, "P1 .apr Model");
            }
            // Right AI widget (upper-right)
            if let Some(right_ai) = self.ai.as_ref() {
                self.render_single_ai_widget(frame, right_ai, self.width - 210.0, "P2 .apr Model");
            }
        } else {
            // SinglePlayer mode - show left AI widget (AI opponent is on the left)
            if let Some(left_ai) = self.left_ai.as_ref() {
                self.render_single_ai_widget(frame, left_ai, 10.0, ".apr ML Model");
            }
        }
    }

    /// Renders a single AI explanation widget at the given x position.
    #[allow(clippy::unused_self, clippy::too_many_lines)]
    fn render_single_ai_widget(
        &self,
        frame: &mut RenderFrame,
        ai: &PongAI,
        widget_x: f32,
        title: &str,
    ) {
        let explanation = ai.explanation();

        // Widget dimensions - compact
        let widget_width = 200.0;
        let widget_height = 180.0;
        let widget_y = 80.0; // Below mode buttons and AI difficulty row

        // Semi-transparent dark background
        frame.fill_rect(
            widget_x,
            widget_y,
            widget_width,
            widget_height,
            Color::new(0.02, 0.02, 0.08, 0.85),
        );

        // Cyan border (matches aprender branding)
        frame.stroke_rect(
            widget_x,
            widget_y,
            widget_width,
            widget_height,
            Color::new(0.2, 0.8, 0.9, 0.9),
            1.0,
        );

        let title_font = "12px monospace";
        let small_font = "10px monospace";
        let tiny_font = "9px monospace";

        let text_color = Color::WHITE;
        let label_color = Color::new(0.6, 0.7, 0.8, 1.0);
        let cyan = Color::new(0.2, 0.8, 0.9, 1.0);

        let mut y = widget_y + 15.0;
        let left_margin = widget_x + 8.0;
        let line_height = 14.0;

        // Title with state indicator
        let state_color = match explanation.state {
            crate::ai::DecisionState::Idle => Color::new(0.5, 0.5, 0.5, 1.0),
            crate::ai::DecisionState::Reacting => Color::new(1.0, 0.8, 0.2, 1.0),
            crate::ai::DecisionState::Tracking => Color::new(0.2, 1.0, 0.4, 1.0),
            crate::ai::DecisionState::Ready => Color::new(0.2, 0.6, 1.0, 1.0),
        };

        frame.fill_text_aligned(
            title,
            left_margin,
            y,
            title_font,
            cyan,
            TextAlign::Left,
            TextBaseline::Middle,
        );

        // State badge
        frame.fill_text_aligned(
            explanation.state.code(),
            widget_x + widget_width - 10.0,
            y,
            title_font,
            state_color,
            TextAlign::Right,
            TextBaseline::Middle,
        );
        y += line_height + 4.0;

        // Separator
        frame.fill_rect(
            left_margin,
            y,
            widget_width - 16.0,
            1.0,
            Color::new(0.3, 0.4, 0.5, 0.6),
        );
        y += 6.0;

        // Feature contributions with bars
        let bar_max_width = 80.0;
        let bar_height = 8.0;
        let bar_x = widget_x + widget_width - bar_max_width - 10.0;

        // Show top 4 contributions
        for contrib in explanation.contributions.iter().take(4) {
            // Feature name (truncated)
            let name = if contrib.name.len() > 12 {
                format!("{}...", &contrib.name[..9])
            } else {
                contrib.name.clone()
            };

            frame.fill_text_aligned(
                &name,
                left_margin,
                y + bar_height / 2.0,
                tiny_font,
                label_color,
                TextAlign::Left,
                TextBaseline::Middle,
            );

            // Contribution bar background
            frame.fill_rect(
                bar_x,
                y,
                bar_max_width,
                bar_height,
                Color::new(0.1, 0.1, 0.15, 1.0),
            );

            // Contribution bar fill
            let bar_width = contrib.importance * bar_max_width;
            let bar_color = if contrib.contribution >= 0.0 {
                Color::new(0.2, 0.7, 0.3, 0.9) // Green for positive
            } else {
                Color::new(0.7, 0.3, 0.2, 0.9) // Red for negative
            };
            frame.fill_rect(bar_x, y, bar_width, bar_height, bar_color);

            y += line_height;
        }

        y += 4.0;

        // Separator
        frame.fill_rect(
            left_margin,
            y,
            widget_width - 16.0,
            1.0,
            Color::new(0.3, 0.4, 0.5, 0.6),
        );
        y += 6.0;

        // Model params row
        let params_text = format!(
            "L{} | {:.0}ms | {:.0}%acc",
            explanation.difficulty_level,
            explanation.reaction_delay_ms,
            explanation.prediction_accuracy * 100.0
        );
        frame.fill_text_aligned(
            &params_text,
            left_margin,
            y,
            tiny_font,
            label_color,
            TextAlign::Left,
            TextBaseline::Middle,
        );
        y += line_height;

        // Decision rationale (wrapped if needed)
        let rationale = if explanation.rationale.len() > 30 {
            format!("{}...", &explanation.rationale[..27])
        } else {
            explanation.rationale.clone()
        };
        frame.fill_text_aligned(
            &rationale,
            left_margin,
            y,
            small_font,
            text_color,
            TextAlign::Left,
            TextBaseline::Middle,
        );
    }
}
