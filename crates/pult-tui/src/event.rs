use crate::app::{App, Focus};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use pult_core::domain::GostPhase;
use tui_textarea::Input;

/// Ações originadas por interação de teclado na TUI.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TuiAction {
    Submit(String),
    Quit,
    FocusToggled,
    Scrolled,
    SpecApproved,
    PhaseAdvanced(GostPhase),
    FinishTriggered,
    MergeTriggered,
}

pub fn handle_key_event(app: &mut App, key: KeyEvent) -> Option<TuiAction> {
    if key.modifiers.contains(KeyModifiers::CONTROL) {
        match key.code {
            KeyCode::Char('q') | KeyCode::Char('c') => {
                app.quit();
                return Some(TuiAction::Quit);
            }
            KeyCode::Char('t') => {
                app.toggle_focus();
                return Some(TuiAction::FocusToggled);
            }
            KeyCode::Char('s') => {
                if let Some(content) = app.submit_input() {
                    return Some(TuiAction::Submit(content));
                }
                return None;
            }
            KeyCode::Char('p') => {
                if app.gost_phase == GostPhase::SpecContract {
                    return Some(TuiAction::SpecApproved);
                }
                return None;
            }
            KeyCode::Char('n') => {
                if let Ok(next) = app.advance_gost_phase() {
                    app.append_viewport_line(&format!(
                        "★ [GOST] Avanço para FASE {}: {}.",
                        next.number(),
                        next.name()
                    ));
                    return Some(TuiAction::PhaseAdvanced(next));
                }
                return None;
            }
            KeyCode::Char('g') => {
                return Some(TuiAction::FinishTriggered);
            }
            KeyCode::Char('m') => {
                return Some(TuiAction::MergeTriggered);
            }
            _ => {}
        }
    }

    match app.focus {
        Focus::Viewport => match key.code {
            KeyCode::Char('k') | KeyCode::Up => {
                app.scroll_up(1);
                Some(TuiAction::Scrolled)
            }
            KeyCode::Char('j') | KeyCode::Down => {
                app.scroll_down(1, 10);
                Some(TuiAction::Scrolled)
            }
            KeyCode::PageUp => {
                app.scroll_up(5);
                Some(TuiAction::Scrolled)
            }
            KeyCode::PageDown => {
                app.scroll_down(5, 10);
                Some(TuiAction::Scrolled)
            }
            KeyCode::Home => {
                app.scroll_to_top();
                Some(TuiAction::Scrolled)
            }
            KeyCode::End => {
                app.scroll_to_bottom(10);
                Some(TuiAction::Scrolled)
            }
            _ => None,
        },
        Focus::Editor => {
            app.textarea.input(Input::from(key));
            None
        }
    }
}
