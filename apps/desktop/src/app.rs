use gpui::{Context, Window, div, prelude::*, rgb};

use crate::projects_view::ProjectsView;
use crate::theme;

pub enum Screen {
    Projects,
    Editor { project_id: String, project_name: String },
}

pub struct OpenCutApp {
    pub screen: Screen,
    pub projects_view: gpui::Entity<ProjectsView>,
}

impl OpenCutApp {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let app = cx.weak_entity();
        let projects_view = cx.new(|_| ProjectsView::new(app));
        Self {
            screen: Screen::Projects,
            projects_view,
        }
    }

    pub fn open_project(&mut self, project_id: String, project_name: String, cx: &mut Context<Self>) {
        self.screen = Screen::Editor {
            project_id,
            project_name,
        };
        cx.notify();
    }
}

impl Render for OpenCutApp {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let content: gpui::AnyElement = match &self.screen {
            Screen::Projects => self.projects_view.clone().into_any_element(),
            Screen::Editor { project_name, project_id } => div()
                .size_full()
                .flex()
                .flex_col()
                .items_center()
                .justify_center()
                .gap_2()
                .child(format!("Editing: {project_name}"))
                .child(
                    div()
                        .text_sm()
                        .text_color(rgb(theme::TEXT_SECONDARY))
                        .child(format!("Editor UI lands here — preview, timeline, panels ({project_id})")),
                )
                .into_any_element(),
        };

        div()
            .size_full()
            .bg(rgb(theme::BACKGROUND))
            .text_color(rgb(theme::TEXT_PRIMARY))
            .child(content)
    }
}
